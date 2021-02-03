use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let number_of_matches = cs
        .count(&["tutorial"], "tok=/.*s.*/", QueryLanguage::AQL)
        .unwrap();
    println!("Number of matches: {}", number_of_matches);

    let matches = cs
        .find(
            &["tutorial"],
            "tok=/.*s.*/",
            QueryLanguage::AQL,
            0,
            Some(100),
            ResultOrder::Normal,
            None,
        )
        .unwrap();
    for (i, m) in matches.iter().enumerate() {
        println!("Match {}: {}", i, m);
    }
}
