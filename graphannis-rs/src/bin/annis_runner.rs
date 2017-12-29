#[macro_use]
extern crate log;

extern crate graphannis;
extern crate rustyline;
extern crate simplelog;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use simplelog::{LogLevelFilter, TermLogger};
use graphannis::relannis;
use std::env;
use std::path::{Path, PathBuf};
use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::api::corpusstorage::Error;

struct AnnisRunner {
    storage: CorpusStorage,
}

impl AnnisRunner {
    pub fn new(data_dir: &Path) -> Result<AnnisRunner, Error> {
        Ok(AnnisRunner {
            storage: CorpusStorage::new(data_dir, None)?,
        })
    }

    pub fn start_loop(&mut self) {
        let mut rl = Editor::<()>::new();
        if let Err(_) = rl.load_history("annis_history.txt") {
            println!("No previous history.");
        }
        loop {
            let readline = rl.readline(">> ");
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
                line_splitted[1].split(' ').collect()
            } else {
                vec![]
            };
            match cmd {
                "import" => if args.len() >= 2 {
                    self.import_relannis(&args[0], &args[1]);
                } else {
                    println!("You need to give the name of the corpus and the location of the relANNIS files and  as argument");
                },
                "quit" | "exit" => {
                    return false;
                }
                _ => {
                    // do nothing
                    println!("unknown command \"{}\"", cmd);
                }
            }
        }
        // stay in loop
        return true;
    }

    fn import_relannis(&mut self, name: &str, path: &str) {
        let t_before = std::time::SystemTime::now();
        let res = relannis::load(&PathBuf::from(path));
        let load_time = t_before.elapsed();
        match res {
            Ok(db) => if let Ok(t) = load_time {
                info!{"Loaded corpus {} in {} ms", name, (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
                info!("Saving imported corpus to disk");
                self.storage.import(name, db);
                info!("Finsished saving corpus {} to disk", name);
            },
            Err(err) => {
                println!("Can't import relANNIS from {}, error:\n{:?}", path, err);
            }
        }
    }
}




fn main() {
    if let Err(e) = TermLogger::init(LogLevelFilter::Info, simplelog::Config::default()) {
        println!("Error, can't initialize the terminal log output: {}", e)
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
