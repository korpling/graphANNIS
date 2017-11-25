extern crate rustyline;

extern crate graphannis;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use graphannis::graphdb::GraphDB;
use graphannis::relannis;

fn import_relannis(path : &str) {
    let t_before = std::time::SystemTime::now();
    let res = relannis::load(path);
    let load_time = t_before.elapsed();
    match res {
        Ok(db) => {
            if let Ok(t) = load_time {
                println!{"Loaded in {} ms", (t.as_secs() * 1000 + t.subsec_nanos() as u64 / 1_000_000)};
            }
        },
        Err(err) => {
            println!("Can't import relANNIS from {}, error:\n{:?}", path, err);
        }
    }
}

fn exec(line :&str) {
    let line_splitted : Vec<&str> = line.splitn(2, ' ').collect();
    if line_splitted.len() > 0 {
        let cmd = line_splitted[0];
        match cmd {
            "import" => {
                if line_splitted.len() > 1 {
                    import_relannis(&line_splitted[1]);
                } else {
                    println!("You need to give the location of the relANNIS files as argument");
                }
            }, 
            "quit" | "exit" => {
                println!("Goodbye!");
                std::process::exit(0);
            },
            _ => {
                // do nothing
                println!("unknown command \"{}\"", cmd);
            }
        }
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("annis_history.txt") {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                exec(&line);
                rl.add_history_entry(&line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("annis_history.txt").unwrap();
}