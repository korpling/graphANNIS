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

struct AnnisRunner {
    storage: CorpusStorage,
}

impl AnnisRunner {
    pub fn new(data_dir: &Path) -> AnnisRunner {
        AnnisRunner {
            storage: CorpusStorage::new(data_dir, None),
        }
    }

    pub fn start_loop(&self) {
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

    fn exec(&self, line: &str) -> bool {
        let line_splitted: Vec<&str> = line.splitn(2, ' ').collect();
        if line_splitted.len() > 0 {
            let cmd = line_splitted[0];
            match cmd {
                "import" => if line_splitted.len() > 1 {
                    self.import_relannis(&line_splitted[1]);
                } else {
                    println!("You need to give the location of the relANNIS files as argument");
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

    fn import_relannis(&self, path: &str) {
        let t_before = std::time::SystemTime::now();
        let res = relannis::load(path);
        let load_time = t_before.elapsed();
        match res {
            Ok(_) => if let Ok(t) = load_time {
                println!{"Loaded in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            },
            Err(err) => {
                println!("Can't import relANNIS from {}, error:\n{:?}", path, err);
            }
        }
    }
}




fn main() {
    TermLogger::init(LogLevelFilter::Info, simplelog::Config::default()).unwrap();

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

            let runner = AnnisRunner::new(&dir);
            runner.start_loop();
        }
        _ => {
            println!("Too many arguments given, only give the data directory as argument");
            std::process::exit(2)
        }
    };
    println!("graphANNIS says good-bye!");
}
