extern crate graphannis;

use graphannis::api::corpusstorage::CorpusStorage;
use graphannis::util;

use std::path::{PathBuf};
use std::cell::RefCell;

use std::collections::HashSet;


thread_local!{
   pub static CORPUS_STORAGE: RefCell<Option<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::new(&db_dir).ok()
        } else {
            None
        };
       return RefCell::new(cs)
       };
}

fn get_query_dir() -> PathBuf {
    let query_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("../queries")
    });
    query_dir
}

fn get_corpus_storage() -> Option<CorpusStorage> {
    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("../data")
    });

    // only execute the test if the directory exists
    let cs = if db_dir.exists() && db_dir.is_dir() {
        CorpusStorage::new(&db_dir).ok()
    } else {
        None
    };
    
    return cs;
}

fn search_test_base(corpus : &str, query_set : &str, panic_on_invalid : bool) {
    let cs = get_corpus_storage();

    if let Some(cs) = cs {
        if let Ok(corpora) = cs.list() {
            let corpora : HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
            // ignore of corpus does not exist
            if corpora.contains(corpus) {
                let mut d = get_query_dir();
                d.push(query_set);
                for def in util::get_queries_from_folder(&d, panic_on_invalid) {
                    let count = cs.count(corpus, &def.json).unwrap_or(0);
                    assert_eq!(
                        def.count, count,
                        "Query '{}' ({}) on corpus {} should have had count {} but was {}.",
                        def.aql, def.name, corpus, def.count, count
                    );
                        
                }
            }
        }
    };
}

#[test]
fn count_gum() {
    search_test_base("GUM", "SearchTestGUM", true);
}

#[test]
fn count_pcc2() {
    search_test_base("pcc2", "SearchTestPcc2", true);
}

#[test]
fn count_parlament() {
    search_test_base("parlament", "SearchTestParlament", true);
}

#[test]
fn count_tiger() {
    search_test_base("tiger2", "SearchTestTiger", true);
}

#[test]
fn count_ridges() {
    search_test_base("ridges7", "SearchTestRidges", true);
}


