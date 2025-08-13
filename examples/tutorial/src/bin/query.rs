use graphannis::CorpusStorage;
use graphannis::corpusstorage::{QueryLanguage, ResultOrder, SearchQuery};
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let search_query = SearchQuery {
        corpus_names: &["tutorial"],
        query: "tok=/.*s.*/",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };

    let number_of_matches = cs.count(search_query.clone()).unwrap();
    println!("Number of matches: {}", number_of_matches);

    let matches = cs
        .find(search_query, 0, Some(100), ResultOrder::Normal)
        .unwrap();
    for (i, m) in matches.iter().enumerate() {
        println!("Match {}: {}", i, m);
    }
}
