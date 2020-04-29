extern crate graphannis;
#[macro_use]
extern crate lazy_static;

use graphannis::corpusstorage::QueryLanguage;
use graphannis::CorpusStorage;

use std::path::PathBuf;
use std::sync::Mutex;

use std::collections::HashSet;

lazy_static! {
    static ref CORPUS_STORAGE : Option<Mutex<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        if db_dir.exists() && db_dir.is_dir() {
            // but fail if directory is blocked
            let cs = CorpusStorage::with_auto_cache_size(&db_dir, true).unwrap();
            return Some(Mutex::new(cs));
        }
        None
    };
}

include!(concat!(env!("OUT_DIR"), "/searchtest.rs"));

#[ignore]
#[test]
fn non_reflexivity_nodes() {
    if let Some(cs_mutex) = CORPUS_STORAGE.as_ref() {
        let corpora = {
            let cs = cs_mutex.lock().unwrap();
            cs.list()
        };
        if let Ok(corpora) = corpora {
            let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
            // ignore of corpus does not exist
            if corpora.contains("GUM") {
                let node_count = {
                    let cs = cs_mutex.lock().unwrap();
                    cs.count(&["GUM"], "node", QueryLanguage::AQL).unwrap_or(0)
                };

                let operators_to_test = vec![
                    ".", ".1,10", "^", "^1,10", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_",
                    "->dep", "->dep *",
                ];

                for o in operators_to_test.into_iter() {
                    let count = {
                        let cs = cs_mutex.lock().unwrap();
                        cs.count(&["GUM"], &format!("node {} node", o), QueryLanguage::AQL)
                            .unwrap_or(0)
                    };
                    assert_ne!(
                        node_count, count,
                        "\"{}\" operator should be non-reflexive for nodes",
                        o
                    );
                }
            }
        }
    }
}

#[ignore]
#[test]
fn non_reflexivity_tokens() {
    if let Some(cs_mutex) = CORPUS_STORAGE.as_ref() {
        let corpora = {
            let cs = cs_mutex.lock().unwrap();
            cs.list()
        };
        if let Ok(corpora) = corpora {
            let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
            // ignore of corpus does not exist
            if corpora.contains("GUM") {
                let tok_count = {
                    let cs = cs_mutex.lock().unwrap();
                    cs.count(&["GUM"], "tok", QueryLanguage::AQL).unwrap_or(0)
                };

                let operators_to_test = vec![
                    ".", ".1,10", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_", "->dep", "->dep *",
                ];

                for o in operators_to_test.into_iter() {
                    let count = {
                        let cs = cs_mutex.lock().unwrap();
                        cs.count(&["GUM"], &format!("tok {} tok", o), QueryLanguage::AQL)
                            .unwrap_or(0)
                    };
                    assert_ne!(
                        tok_count, count,
                        "\"{}\" operator should be non-reflexive for tokens",
                        o
                    );
                }
            }
        }
    }
}
