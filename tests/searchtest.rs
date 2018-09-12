extern crate graphannis;

use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::util;

use std::path::{PathBuf};
use std::cell::RefCell;

use std::collections::HashSet;


thread_local!{
   pub static CORPUS_STORAGE : RefCell<Option<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::new_auto_cache_size(&db_dir, false).ok()
        } else {
            None
        };
        return RefCell::new(cs);
       };
}

fn get_query_dir() -> PathBuf {
    let query_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("queries")
    });
    query_dir
}


fn search_test_base(corpus : &str, panic_on_invalid : bool) {
    CORPUS_STORAGE.with(|cs| {
        if let Some(ref cs) = *cs.borrow() {
            if let Ok(corpora) = cs.list() {
                let corpora : HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
                // ignore of corpus does not exist
                if corpora.contains(corpus) {
                    let mut d = get_query_dir();
                    d.push(corpus);
                    for def in util::get_queries_from_folder(&d, panic_on_invalid) {
                        let count = cs.count(corpus, &def.aql).unwrap_or(0);
                        assert_eq!(
                            def.count, count,
                            "Query '{}' ({}) on corpus {} should have had count {} but was {}.",
                            def.aql, def.name, corpus, def.count, count
                        );
                            
                    }
                }
            }
        }
    });
}

#[ignore]
#[test]
fn all_from_folder() {
    let queries_dir = get_query_dir();
    // each folder is one corpus
    if let Ok(paths) = std::fs::read_dir(queries_dir) {
        for p in paths {
            if let Ok(p) = p {
                if let Ok(ftype) = p.file_type() {
                    if ftype.is_dir() {
                        if let Ok(corpus_name) = p.file_name().into_string() {
                           search_test_base(&corpus_name, true);
                        }
                    }
                }
            }
        }
    }
}


