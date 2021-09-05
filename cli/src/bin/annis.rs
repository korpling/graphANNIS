#[macro_use]
extern crate anyhow;

use clap::{App, Arg};
use graphannis::corpusstorage::FrequencyDefEntry;
use graphannis::corpusstorage::LoadStatus;
use graphannis::corpusstorage::QueryLanguage;
use graphannis::corpusstorage::ResultOrder;
use graphannis::corpusstorage::{CorpusInfo, SearchQuery};
use graphannis::corpusstorage::{ExportFormat, ImportFormat};
use graphannis::CorpusStorage;
use log::info;
use prettytable::Cell;
use prettytable::Row;
use prettytable::Table;
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use simplelog::{LevelFilter, SimpleLogger, TermLogger};
use std::path::{Path, PathBuf};
use std::{collections::BTreeSet, time::Duration};

use anyhow::Result;

#[derive(Helper, Hinter, Highlighter, Validator)]
struct ConsoleHelper {
    known_commands: BTreeSet<String>,
    filename_completer: FilenameCompleter,
    pub corpora: Vec<CorpusInfo>,
}

impl ConsoleHelper {
    pub fn new(corpora: Vec<CorpusInfo>) -> ConsoleHelper {
        let mut known_commands = BTreeSet::new();
        known_commands.insert("import".to_string());
        known_commands.insert("export".to_string());
        known_commands.insert("list".to_string());
        known_commands.insert("delete".to_string());
        known_commands.insert("corpus".to_string());
        known_commands.insert("set-offset".to_string());
        known_commands.insert("set-limit".to_string());
        known_commands.insert("set-timeout".to_string());
        known_commands.insert("preload".to_string());
        known_commands.insert("count".to_string());
        known_commands.insert("find".to_string());
        known_commands.insert("frequency".to_string());
        known_commands.insert("plan".to_string());
        known_commands.insert("re-optimize".to_string());
        known_commands.insert("set-disk-based".to_string());
        known_commands.insert("set-parallel-search".to_string());
        known_commands.insert("set-quirks-mode".to_string());
        known_commands.insert("info".to_string());

        known_commands.insert("quit".to_string());
        known_commands.insert("exit".to_string());

        ConsoleHelper {
            known_commands,
            filename_completer: FilenameCompleter::new(),
            corpora,
        }
    }
}

impl Completer for ConsoleHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context,
    ) -> std::result::Result<(usize, Vec<rustyline::completion::Pair>), ReadlineError> {
        // check for more specialized completers
        if line.starts_with("import ") || line.starts_with("export ") {
            return self.filename_completer.complete(line, pos, ctx);
        } else if line.starts_with("corpus ") || line.starts_with("delete ") {
            // auto-complete the corpus names
            if let Some(prefix_len) = line.rfind(' ') {
                let prefix_len = prefix_len + 1;
                let mut matching_corpora = vec![];
                let corpus_prefix = &line[prefix_len..];
                for c in self.corpora.iter() {
                    if c.name.starts_with(corpus_prefix) {
                        let p = rustyline::completion::Pair {
                            display: c.name.clone(),
                            replacement: c.name.clone(),
                        };
                        matching_corpora.push(p);
                    }
                }
                return Ok((pos - corpus_prefix.len(), matching_corpora));
            } else {
                return Ok((pos, vec![]));
            }
        }

        let mut cmds = Vec::new();

        // only check at end of line for initial command strings
        if pos == line.len() {
            // check alll commands if the current string is a valid suffix
            for candidate in self.known_commands.iter() {
                if candidate.starts_with(line) {
                    let p = rustyline::completion::Pair {
                        display: candidate.clone(),
                        replacement: candidate.clone(),
                    };
                    cmds.push(p);
                }
            }
        }
        Ok((0, cmds))
    }
}

struct AnnisRunner {
    storage: Option<CorpusStorage>,
    current_corpus: Vec<String>,
    offset: usize,
    limit: Option<usize>,
    data_dir: PathBuf,
    use_parallel_joins: bool,
    use_disk: bool,
    query_language: QueryLanguage,
    timeout: Option<Duration>,
}

impl AnnisRunner {
    pub fn new(data_dir: &Path) -> Result<AnnisRunner> {
        Ok(AnnisRunner {
            storage: Some(CorpusStorage::with_auto_cache_size(data_dir, true)?),
            current_corpus: vec![],
            data_dir: PathBuf::from(data_dir),
            use_parallel_joins: true,
            use_disk: false,
            query_language: QueryLanguage::AQL,
            offset: 0,
            limit: None,
            timeout: None,
        })
    }

    pub fn start_loop(&mut self) {
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
        if rl.load_history("annis_history.txt").is_err() {
            println!("No previous history.");
        }

        if let Some(ref storage) = self.storage {
            rl.set_helper(Some(ConsoleHelper::new(storage.list().unwrap_or_default())));
        }

        loop {
            let prompt = if self.current_corpus.is_empty() {
                String::from(">> ")
            } else {
                format!("{}> ", self.current_corpus.join(","))
            };
            let readline = rl.readline(&prompt);
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line.clone());
                    if !self.exec(&line) {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        rl.save_history("annis_history.txt").unwrap();
    }

    fn exec(&mut self, line: &str) -> bool {
        let line_splitted: Vec<&str> = line.splitn(2, ' ').collect();
        if !line_splitted.is_empty() {
            let cmd = line_splitted[0];
            let args = if line_splitted.len() > 1 {
                String::from(line_splitted[1])
            } else {
                String::from("")
            };
            let result = match cmd {
                "import" => self.import(&args),
                "export" => self.export_graphml(&args),
                "list" => self.list(),
                "delete" => self.delete(&args),
                "corpus" => self.corpus(&args),
                "set-offset" => self.set_offset(&args),
                "set-limit" => self.set_limit(&args),
                "set-timeout" => self.set_timeout(&args),
                "preload" => self.preload(),
                "plan" => self.plan(&args),
                "re-optimize" => self.reoptimize(),
                "count" => self.count(&args),
                "find" => self.find(&args),
                "frequency" => self.frequency(&args),
                "set-parallel-search" => self.use_parallel(&args),
                "set-disk-based" => self.use_disk(&args),
                "set-quirks-mode" => self.quirks_mode(&args),
                "info" => self.info(&args),
                "quit" | "exit" => return false,
                _ => Err(anyhow!("unknown command \"{}\"", cmd)),
            };
            if let Err(err) = result {
                println!("Error: {:?}", err)
            }
        }
        // stay in loop
        true
    }

    fn import(&mut self, args: &str) -> Result<()> {
        let args: Vec<&str> = args.split(' ').collect();
        if args.is_empty() {
            bail!("You need to location of the files to import and optionally a name as argument");
        }

        let overwritten_corpus_name = if args.len() >= 2 {
            Some(args[1].to_owned())
        } else {
            None
        };

        // Determine most likely input format based on the extension of the file
        let path = PathBuf::from(args[0]);

        if path.exists() {
            let file_ext_owned = path
                .extension()
                .map(|file_ext| file_ext.to_string_lossy().to_lowercase());
            let file_ext = file_ext_owned.as_deref();

            if file_ext == Some("zip") {
                let zip_file = std::fs::File::open(path)?;
                // Import  ZIP file with possible multiple corpora
                let t_before = std::time::SystemTime::now();
                let names = self
                    .storage
                    .as_ref()
                    .ok_or_else(|| anyhow!("No corpus storage location set"))?
                    .import_all_from_zip(zip_file, self.use_disk, true, |status| {
                        info!("{}", status)
                    })?;
                let load_time = t_before.elapsed();
                if let Ok(t) = load_time {
                    info! {"imported corpora {:?} in {} ms", names, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                }
            } else {
                // Import a single corpus
                let mut format = ImportFormat::RelANNIS;

                if file_ext == Some("graphml") || file_ext == Some("xml") {
                    format = ImportFormat::GraphML
                }

                let t_before = std::time::SystemTime::now();
                let name: String = self
                    .storage
                    .as_ref()
                    .ok_or_else(|| anyhow!("No corpus storage location set"))?
                    .import_from_fs(
                        &path,
                        format,
                        overwritten_corpus_name,
                        self.use_disk,
                        true,
                        |status| info!("{}", status),
                    )?;
                let load_time = t_before.elapsed();
                if let Ok(t) = load_time {
                    info! {"imported corpus {} in {} ms", name, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                }
            }
        }

        Ok(())
    }

    fn export_graphml(&mut self, args: &str) -> Result<()> {
        let args: Vec<&str> = args.split(' ').collect();
        if args.is_empty() {
            bail!("You need give the location of the output XML file as argument");
        }

        let path = PathBuf::from(args[0]);
        let mut format = ExportFormat::GraphML;
        if let Some(file_ext) = path.extension() {
            if file_ext.to_string_lossy().to_lowercase() == "zip" {
                format = ExportFormat::GraphMLZip;
            } else if file_ext.to_string_lossy() == ".graphml" && self.current_corpus.len() != 1 {
                bail!(
                    r##"You need to select a *single* corpus first with the \"corpus\" command when exporting to a GraphML file. 
                To export multiple corpora, select a directory as output or a ZIP file (ending with .zip)"##
                );
            }
        } else {
            format = ExportFormat::GraphMLDirectory;
        }

        let t_before = std::time::SystemTime::now();
        self.storage
            .as_ref()
            .ok_or_else(|| anyhow!("No corpus storage location set"))?
            .export_to_fs(&self.current_corpus, &path, format)?;
        let load_time = t_before.elapsed();
        if let Ok(t) = load_time {
            info! {"exported corpora {:?} in {} ms", &self.current_corpus, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
        }

        Ok(())
    }

    fn reoptimize(&self) -> Result<()> {
        for corpus in self.current_corpus.iter() {
            self.storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .reoptimize_implementation(corpus, self.use_disk)?
        }

        Ok(())
    }

    fn list(&self) -> Result<()> {
        let mut corpora = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow!("No corpus storage location set"))?
            .list()?;
        corpora.sort_unstable_by_key(|info| info.name.clone());
        for c in corpora {
            let desc = match c.load_status {
                LoadStatus::NotLoaded => String::from("not loaded"),
                LoadStatus::PartiallyLoaded(size) => format!(
                    "partially loaded, {:.2} MB",
                    size as f64 / (1024 * 1024) as f64
                ),
                LoadStatus::FullyLoaded(size) => format!(
                    "fully loaded, {:.2} MB ",
                    size as f64 / (1024 * 1024) as f64
                ),
            };
            println!("{} ({})", c.name, desc);
        }
        Ok(())
    }

    fn delete(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            bail!("You need the name as an argument");
        }
        let name = args;

        self.storage
            .as_ref()
            .ok_or_else(|| anyhow!("No corpus storage location set"))?
            .delete(name)?;
        info!("Deleted corpus {}.", name);

        Ok(())
    }

    fn corpus(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            self.current_corpus = vec![];
        } else {
            let corpora = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .list()?;
            let corpora: BTreeSet<_> = corpora.into_iter().map(|c| c.name).collect();
            let selected = args.split_ascii_whitespace();
            self.current_corpus = Vec::new();
            for s in selected {
                if corpora.contains(s) {
                    self.current_corpus.push(s.to_string());
                } else {
                    println!("Corpus {} does not exist. Uses the \"list\" command to get all available corpora", s);
                }
            }
        }
        Ok(())
    }

    fn set_offset(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            self.offset = 0;
        } else {
            self.offset = args.trim().parse::<usize>()?;
        }
        Ok(())
    }

    fn set_limit(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            self.limit = None;
        } else {
            self.limit = Some(args.trim().parse::<usize>()?);
        }
        Ok(())
    }

    fn set_timeout(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            self.timeout = None;
            println!("Timeout disabled");
        } else {
            let seconds = args.trim().parse::<u64>()?;
            println!("Timeout set to {} seconds", seconds);
            self.timeout = Some(Duration::from_secs(seconds));
        }
        Ok(())
    }

    fn info(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus for the \"info\" command");
        } else {
            for corpus in self.current_corpus.iter() {
                let cinfo = self
                    .storage
                    .as_ref()
                    .ok_or_else(|| anyhow!("No corpus storage location set"))?
                    .info(corpus)?;
                if args == "config" {
                    println!("{}", toml::to_string(&cinfo.config)?);
                } else {
                    println!("{}", cinfo);
                }
            }
        }
        Ok(())
    }

    fn preload(&mut self) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            for corpus in self.current_corpus.iter() {
                let t_before = std::time::SystemTime::now();
                self.storage
                    .as_ref()
                    .ok_or_else(|| anyhow!("No corpus storage location set"))?
                    .preload(corpus)?;
                let load_time = t_before.elapsed();
                if let Ok(t) = load_time {
                    info! {"Preloaded corpus in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                }
            }
        }
        Ok(())
    }

    fn plan(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            let t_before = std::time::SystemTime::now();
            let plan = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .plan(&self.current_corpus, args, self.query_language)?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Planned query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            println!("{}", plan);
        }
        Ok(())
    }

    fn create_query_from_args<'a>(&'a self, query: &'a str) -> SearchQuery<'a, String> {
        SearchQuery {
            corpus_names: &self.current_corpus,
            query_language: self.query_language,
            timeout: self.timeout,
            query,
        }
    }

    fn count(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            let t_before = std::time::SystemTime::now();
            let c = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .count_extra(self.create_query_from_args(args))?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Executed query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }
            println!(
                "result: {} matches in {} documents",
                c.match_count, c.document_count
            );
        }
        Ok(())
    }

    fn find(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            let t_before = std::time::SystemTime::now();
            let matches = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .find(
                    self.create_query_from_args(args),
                    self.offset,
                    self.limit,
                    ResultOrder::Normal,
                )?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Executed query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            for m in matches {
                println!("{}", m);
            }
        }
        Ok(())
    }

    fn frequency(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            let splitted_arg: Vec<&str> = args.splitn(2, ' ').collect();
            let table_def: Vec<FrequencyDefEntry> = if splitted_arg.len() == 2 {
                // split the second argument
                let defs = splitted_arg[0].split(',');
                defs.filter_map(|d| -> Option<FrequencyDefEntry> { d.parse().ok() })
                    .collect()
            } else {
                println!("You have to give the frequency definition as first argument and the AQL as second argument");
                return Ok(());
            };

            let mut out = Table::new();
            let mut header_row = Row::empty();
            for def in table_def.iter() {
                header_row.add_cell(Cell::from(&format!("{}#{}", def.node_ref, def.name)));
            }
            header_row.add_cell(Cell::from(&"count"));
            out.add_row(header_row);

            let t_before = std::time::SystemTime::now();
            let frequency_table = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow!("No corpus storage location set"))?
                .frequency(self.create_query_from_args(splitted_arg[1]), table_def)?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Executed query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            // map the resulting frequency table to an output

            // TODO: map header
            for row in frequency_table.into_iter() {
                let mut out_row = Row::empty();
                for att in row.values.iter() {
                    if att.trim().is_empty() {
                        // This is whitespace only, add some quotation marks to show to make it visible
                        let mut val = "'".to_owned();
                        val.push_str(att);
                        val.push('\'');
                        out_row.add_cell(Cell::from(&val));
                    } else {
                        out_row.add_cell(Cell::from(att));
                    }
                }
                // also add the count
                out_row.add_cell(Cell::from(&row.count));
                out.add_row(out_row);
            }
            out.printstd();

            // TODO output error if needed
        }

        Ok(())
    }

    fn use_parallel(&mut self, args: &str) -> Result<()> {
        let new_val = match args.trim().to_lowercase().as_str() {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => bail!("unknown argument \"{}\"", args),
        };

        if self.use_parallel_joins != new_val {
            // the old corpus storage instance should release the disk lock
            self.storage = None;

            // re-init the corpus storage
            self.storage = Some(CorpusStorage::with_auto_cache_size(
                &self.data_dir,
                new_val,
            )?);
            self.use_parallel_joins = new_val;
        }

        if self.use_parallel_joins {
            println!("Join parallization is enabled");
        } else {
            println!("Join parallization is disabled");
        }

        Ok(())
    }

    fn use_disk(&mut self, args: &str) -> Result<()> {
        let new_val = match args.trim().to_lowercase().as_str() {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => return Err(anyhow!("unknown argument \"{}\"", args)),
        };

        self.use_disk = new_val;
        Ok(())
    }

    fn quirks_mode(&mut self, args: &str) -> Result<()> {
        let use_quirks = match args.trim().to_lowercase().as_str() {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => return Err(anyhow!("unknown argument \"{}\"", args)),
        };

        self.query_language = if use_quirks {
            QueryLanguage::AQLQuirksV3
        } else {
            QueryLanguage::AQL
        };

        match self.query_language {
            QueryLanguage::AQLQuirksV3 => {
                println!("Quirks mode is enabled");
            }
            QueryLanguage::AQL => {
                println!("Quirks mode is disabled");
            }
        }

        Ok(())
    }
}

fn main() {
    let matches = App::new("graphANNIS CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Command line interface to the graphANNIS API.")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Enables debug output")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("cmd")
                .short("c")
                .long("cmd")
                .help("Executes command")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DATA_DIR")
                .help("directory containing the data")
                .required(true)
                .index(1),
        )
        .get_matches();

    let log_filter = if matches.is_present("debug") {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let log_config = simplelog::ConfigBuilder::new()
        .add_filter_ignore_str("rustyline")
        .build();

    if let Err(e) = TermLogger::init(
        log_filter,
        log_config.clone(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    ) {
        println!("Error, can't initialize the terminal log output: {}.\nWill degrade to a more simple logger", e);
        if let Err(e_simple) = SimpleLogger::init(log_filter, log_config) {
            println!("Simple logging failed too: {}", e_simple);
        }
    }

    let dir = std::path::PathBuf::from(matches.value_of("DATA_DIR").unwrap());
    if !dir.is_dir() {
        println!("Must give a valid directory as argument");
        std::process::exit(3);
    }

    let runner_result = AnnisRunner::new(&dir);
    match runner_result {
        Ok(mut runner) => {
            if let Some(commands) = matches.values_of("cmd") {
                // execute commands directly
                for single_command in commands {
                    runner.exec(single_command);
                }
            } else {
                runner.start_loop();
            }
        }
        Err(e) => println!("Can't start console because of loading error: {:?}", e),
    };

    println!("graphANNIS says good-bye!");
}
