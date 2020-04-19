use clap::{App, Arg};
use graphannis::corpusstorage::CorpusInfo;
use graphannis::corpusstorage::FrequencyDefEntry;
use graphannis::corpusstorage::ImportFormat;
use graphannis::corpusstorage::LoadStatus;
use graphannis::corpusstorage::QueryLanguage;
use graphannis::corpusstorage::ResultOrder;
use graphannis::errors::*;
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
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

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
        known_commands.insert("list".to_string());
        known_commands.insert("delete".to_string());
        known_commands.insert("corpus".to_string());
        known_commands.insert("set-offset".to_string());
        known_commands.insert("set-limit".to_string());
        known_commands.insert("preload".to_string());
        known_commands.insert("update_statistics".to_string());
        known_commands.insert("count".to_string());
        known_commands.insert("find".to_string());
        known_commands.insert("frequency".to_string());
        known_commands.insert("plan".to_string());
        known_commands.insert("use_disk".to_string());
        known_commands.insert("use_parallel".to_string());
        known_commands.insert("quirks_mode".to_string());
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
        if line.starts_with("import ") {
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
        })
    }

    pub fn start_loop(&mut self) {
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
        if let Err(_) = rl.load_history("annis_history.txt") {
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
                    if self.exec(&line) == false {
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
        if line_splitted.len() > 0 {
            let cmd = line_splitted[0];
            let args = if line_splitted.len() > 1 {
                String::from(line_splitted[1])
            } else {
                String::from("")
            };
            let result = match cmd {
                "import" => self.import_relannis(&args),
                "list" => self.list(),
                "delete" => self.delete(&args),
                "corpus" => self.corpus(&args),
                "set-offset" => self.set_offset(&args),
                "set-limit" => self.set_limit(&args),
                "preload" => self.preload(),
                "update_statistics" => self.update_statistics(),
                "plan" => self.plan(&args),
                "count" => self.count(&args),
                "find" => self.find(&args),
                "frequency" => self.frequency(&args),
                "use_parallel" => self.use_parallel(&args),
                "use_disk" => self.use_disk(&args),
                "quirks_mode" => self.quirks_mode(&args),
                "info" => self.info(),
                "quit" | "exit" => return false,
                _ => Err(format!("unknown command \"{}\"", cmd).into()),
            };
            if let Err(err) = result {
                match err {
                    Error::Generic { msg, .. } => println!("Error: {}", msg),
                    _ => println!("Error: {:?}", err),
                };
            }
        }
        // stay in loop
        return true;
    }

    fn import_relannis(&mut self, args: &str) -> Result<()> {
        let args: Vec<&str> = args.split(' ').collect();
        if args.is_empty() {
            return Err(
                "You need to location of the relANNIS files and optionally a name as argument"
                    .into(),
            );
        }

        let overwritten_corpus_name = if args.len() >= 2 {
            Some(args[1].to_owned())
        } else {
            None
        };

        let path = args[0];

        let t_before = std::time::SystemTime::now();
        let name: String = self
            .storage
            .as_ref()
            .ok_or("No corpus storage location set")?
            .import_from_fs(
                &PathBuf::from(path),
                ImportFormat::RelANNIS,
                overwritten_corpus_name,
                self.use_disk,
            )?;
        let load_time = t_before.elapsed();
        if let Ok(t) = load_time {
            info! {"imported corpus {} in {} ms", name, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
        }

        Ok(())
    }

    fn list(&self) -> Result<()> {
        let mut corpora = self
            .storage
            .as_ref()
            .ok_or("No corpus storage location set")?
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
            return Err("You need the name as an argument".into());
        }
        let name = args;

        self.storage
            .as_ref()
            .ok_or("No corpus storage location set")?
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
                .ok_or("No corpus storage location set")?
                .list()?;
            let corpora = BTreeSet::from_iter(corpora.into_iter().map(|c| c.name));
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
            self.offset = usize::from_str_radix(args.trim(), 10)?;
        }
        Ok(())
    }

    fn set_limit(&mut self, args: &str) -> Result<()> {
        if args.is_empty() {
            self.limit = None;
        } else {
            self.limit = Some(usize::from_str_radix(args.trim(), 10)?);
        }
        Ok(())
    }

    fn info(&self) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus for the \"info\" command");
        } else {
            for corpus in self.current_corpus.iter() {
                let cinfo = self
                    .storage
                    .as_ref()
                    .ok_or("No corpus storage location set")?
                    .info(corpus)?;
                println!("{}", cinfo);
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
                    .ok_or("No corpus storage location set")?
                    .preload(corpus)?;
                let load_time = t_before.elapsed();
                if let Ok(t) = load_time {
                    info! {"Preloaded corpus in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                }
            }
        }
        Ok(())
    }

    fn update_statistics(&mut self) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            for corpus in self.current_corpus.iter() {
                let t_before = std::time::SystemTime::now();
                self.storage
                    .as_ref()
                    .ok_or("No corpus storage location set")?
                    .update_statistics(corpus)?;
                let load_time = t_before.elapsed();
                if let Ok(t) = load_time {
                    info! {"Updated statistics for corpus in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
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
                .ok_or("No corpus storage location set")?
                .plan(&self.current_corpus, args, self.query_language)?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Planned query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            println!("{}", plan);
        }
        Ok(())
    }

    fn count(&self, args: &str) -> Result<()> {
        if self.current_corpus.is_empty() {
            println!("You need to select a corpus first with the \"corpus\" command");
        } else {
            let t_before = std::time::SystemTime::now();
            let c = self
                .storage
                .as_ref()
                .ok_or("No corpus storage location set")?
                .count(&self.current_corpus, args, self.query_language)?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Executed query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }
            println!("result: {} matches", c);
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
                .ok_or("No corpus storage location set")?
                .find(
                    &self.current_corpus[..],
                    args,
                    self.query_language,
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
                .ok_or("No corpus storage location set")?
                .frequency(
                    &self.current_corpus,
                    splitted_arg[1],
                    self.query_language,
                    table_def,
                )?;
            let load_time = t_before.elapsed();
            if let Ok(t) = load_time {
                info! {"Executed query in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            // map the resulting frequency table to an output

            // TODO: map header
            for row in frequency_table.into_iter() {
                let mut out_row = Row::empty();
                for att in row.0.iter() {
                    out_row.add_cell(Cell::from(att));
                }
                // also add the count
                out_row.add_cell(Cell::from(&row.1));
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
            _ => return Err(format!("unknown argument \"{}\"", args).into()),
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
            _ => return Err(format!("unknown argument \"{}\"", args).into()),
        };

        self.use_disk = new_val;
        Ok(())
    }

    fn quirks_mode(&mut self, args: &str) -> Result<()> {
        let use_quirks = match args.trim().to_lowercase().as_str() {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => return Err(format!("unknown argument \"{}\"", args).into()),
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
        .add_filter_ignore_str("rustyline:")
        .build();

    if let Err(e) = TermLogger::init(
        log_filter,
        log_config.clone(),
        simplelog::TerminalMode::Mixed,
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
            if let Some(cmd) = matches.value_of("cmd") {
                // execute command directly
                runner.exec(cmd);
            } else {
                runner.start_loop();
            }
        }
        Err(e) => println!("Can't start console because of loading error: {:?}", e),
    };

    println!("graphANNIS says good-bye!");
}