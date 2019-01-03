extern crate graphannis;

use graphannis::corpusstorage::QueryLanguage;
use graphannis::util;
use graphannis::CorpusStorage;

use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
   pub static CORPUS_STORAGE : RefCell<Option<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::with_auto_cache_size(&db_dir, true).ok()
        } else {
            None
        };
        return RefCell::new(cs);
       };
}

fn get_query_file() -> PathBuf {
    let query_file = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("queries/tests.csv")
    });
    query_file
}

#[ignore]
#[test]
fn all_from_csv_parallel() {
    let queries_file = get_query_file();
    CORPUS_STORAGE.with(|cs| {
        if let Some(ref cs) = *cs.borrow() {
            for def in util::get_queries_from_csv(&queries_file, true) {
                let mut count = 0;
                for c in def.corpus.iter() {
                    count += cs.count(c, &def.aql, QueryLanguage::AQL).unwrap_or(0)
                }
                assert_eq!(
                    def.count, count,
                    "Query '{}' ({}) on corpus {:?} should have had count {} but was {}.",
                    def.aql, def.name, def.corpus, def.count, count
                );
            }
        }
    });
}
