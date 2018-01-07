#[macro_use]
extern crate log;

extern crate graphannis;
extern crate rustyline;
extern crate simplelog;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{Completer, FilenameCompleter};
use simplelog::{LogLevelFilter, TermLogger, SimpleLogger};
use graphannis::relannis;
use std::env;
use std::path::{Path, PathBuf};
use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::api::corpusstorage::Error;
use std::collections::BTreeSet;
use std::iter::FromIterator;

struct CommandCompleter {
    known_commands : BTreeSet<String>,
    filename_completer : FilenameCompleter,
}

impl CommandCompleter {
    pub fn new() -> CommandCompleter {
        let mut known_commands = BTreeSet::new();
        known_commands.insert("import".to_string());
        known_commands.insert("list".to_string());
        known_commands.insert("corpus".to_string());
        known_commands.insert("count".to_string());
        known_commands.insert("quit".to_string());
        known_commands.insert("exit".to_string());
        
        CommandCompleter {
            known_commands,
            filename_completer: FilenameCompleter::new(),
        }
    }
}

impl Completer for CommandCompleter {
    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<String>), ReadlineError> {
        let mut cmds = Vec::new();

        // check for more specialized completers
        if line.starts_with("import ") {
            return self.filename_completer.complete(line, pos);
        }

        // only check at end of line for initial command strings
        if pos == line.len() {
            // check alll commands if the current string is a valid suffix
            for candidate in self.known_commands.iter() {
                if candidate.starts_with(line) {
                    cmds.push(candidate.clone());
                }
            }
        }
        Ok((0, cmds))
    }
}
struct AnnisRunner {
    storage: CorpusStorage,
    current_corpus : Option<String>,
}

impl AnnisRunner {
    pub fn new(data_dir: &Path) -> Result<AnnisRunner, Error> {
        Ok(AnnisRunner {
            storage: CorpusStorage::new(data_dir)?,
            current_corpus: None,
        })
    }

    pub fn start_loop(&mut self) {
        let mut rl = Editor::<CommandCompleter>::new();
        if let Err(_) = rl.load_history("annis_history.txt") {
            println!("No previous history.");
        }
        rl.set_completer(Some(CommandCompleter::new()));
    

        loop {
            let prompt = if let Some(ref c) = self.current_corpus {
                format!("{}> ", c)
            } else {
                String::from(">> ")
            };
            let readline = rl.readline(&prompt);
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
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
            match cmd {
                "import" =>  self.import_relannis(&args),
                "list" => self.list(),
                "corpus" => self.corpus(&args),
                "count" => self.count(&args),
                "quit" | "exit" => return false,
                _ => println!("unknown command \"{}\"", cmd),
            };
        }
        // stay in loop
        return true;
    }

    fn import_relannis(&mut self, args : &str) {

        let args : Vec<&str> = args.split(' ').collect();
        if args.len() < 2 {
            println!("You need to give the name of the corpus and the location of the relANNIS files and  as argument");
            return;
        }

        let name = args[0];
        let path = args[1];

        let t_before = std::time::SystemTime::now();
        let res = relannis::load(&PathBuf::from(path));
        let load_time = t_before.elapsed();
        match res {
            Ok(db) => if let Ok(t) = load_time {
                info!{"Loaded corpus {} in {} ms", name, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                info!("Saving imported corpus to disk");
                self.storage.import(name, db);
                info!("Finished saving corpus {} to disk", name);
            },
            Err(err) => {
                println!("Can't import relANNIS from {}, error:\n{:?}", path, err);
            }
        }
    }

    fn list(&self) {
        let corpora = self.storage.list();
        for c in corpora {
            println!("{}", c);
        }
    }

    fn corpus(&mut self, args : &str) {
        if args.is_empty() {
            self.current_corpus = None;
        } else {
            let corpora = BTreeSet::from_iter(self.storage.list().into_iter());
            let selected = String::from(args);
            if corpora.contains(&selected) {
                self.current_corpus = Some(String::from(args));    
            } else {
                println!("Corpus {} does not exist. Uses the \"list\" command to get all available corpora", selected);
            }
        }
    }

    fn count(&self, args : &str) {

        if let Some(ref corpus) = self.current_corpus {
            let t_before = std::time::SystemTime::now();
            let c = self.storage.count(corpus, args);
            let load_time = t_before.elapsed();

            if let Ok(t) = load_time {
                info!{"Executed query in in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }

            if let Ok(c) = c {                
                println!("result: {} matches", c);
            } else {
                println!("Error when executing query: {:?}", c);
            }
            
        } else {
            println!("You need to select a corpus first with the \"corpus\" command");
        }

       
    }
}




fn main() {
    if let Err(e) = TermLogger::init(LogLevelFilter::Info, simplelog::Config::default()) {
        println!("Error, can't initialize the terminal log output: {}.\nWill degrade to a more simple logger", e);
        if let Err(e_simple) = SimpleLogger::init(LogLevelFilter::Debug, simplelog::Config::default()) {
            println!("Simple logging failed too: {}", e_simple);
        }
     }

    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("Please give the data directory as argument.");
            std::process::exit(1);
        }
        2 => {
            let dir = std::path::PathBuf::from(&args[1]);
            if !dir.is_dir() {
                println!("Must give a valid directory as argument");
                std::process::exit(3);
            }

            let runner_result = AnnisRunner::new(&dir);
            match runner_result {
                 Ok(mut runner) =>  runner.start_loop(),
                 Err(e) => println!("Can't start console because of loading error: {:?}", e)
            };

        }
        _ => {
            println!("Too many arguments given, only give the data directory as argument");
            std::process::exit(2)
        }
    };
    println!("graphANNIS says good-bye!");
}
