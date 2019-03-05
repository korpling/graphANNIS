use graphannis::CorpusStorage;
use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let number_of_matches = cs.count("tutorial", "tok=/.*s.*/", QueryLanguage::AQL).unwrap();
    println!("Number of matches: {}", number_of_matches);

    let matches = cs.find("tutorial", "tok=/.*s.*/", QueryLanguage::AQL, 0, 100, ResultOrder::Normal).unwrap();
    for i in 0..matches.len() {
        println!("Match {}: {}", i, matches[i]);
    }
}