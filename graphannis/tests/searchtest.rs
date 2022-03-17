extern crate graphannis;
#[macro_use]
extern crate lazy_static;

use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::CorpusStorage;

use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    static ref CORPUS_STORAGE: Option<Mutex<CorpusStorage>> = {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("../data")
        });
        let cs = CorpusStorage::with_auto_cache_size(&db_dir, true).unwrap();
        Some(Mutex::new(cs))
    };
}

include!(concat!(env!("OUT_DIR"), "/searchtest.rs"));

#[ignore]
#[test]
fn non_reflexivity_nodes() {
    if let Some(cs_mutex) = CORPUS_STORAGE.as_ref() {
        let node_count = {
            let cs = cs_mutex.lock().unwrap();
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "node",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            cs.count(query).unwrap_or(0)
        };

        let operators_to_test = vec![
            ".", ".1,10", "^", "^1,10", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_", "->dep",
            "->dep *",
        ];

        for o in operators_to_test.into_iter() {
            let count = {
                let cs = cs_mutex.lock().unwrap();
                let query = SearchQuery {
                    corpus_names: &["GUM"],
                    query: "node {} node",
                    query_language: QueryLanguage::AQL,
                    timeout: None,
                };
                cs.count(query).unwrap_or(0)
            };
            assert_ne!(
                node_count, count,
                "\"{}\" operator should be non-reflexive for nodes",
                o
            );
        }
    }
}

#[ignore]
#[test]
fn non_reflexivity_tokens() {
    if let Some(cs_mutex) = CORPUS_STORAGE.as_ref() {
        let tok_count = {
            let cs = cs_mutex.lock().unwrap();
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "tok",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            cs.count(query).unwrap_or(0)
        };

        let operators_to_test = vec![
            ".", ".1,10", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_", "->dep", "->dep *",
        ];

        for o in operators_to_test.into_iter() {
            let count = {
                let cs = cs_mutex.lock().unwrap();
                let query = SearchQuery {
                    corpus_names: &["GUM"],
                    query: &format!("tok {} tok", o),
                    query_language: QueryLanguage::AQL,
                    timeout: None,
                };
                cs.count(query).unwrap_or(0)
            };
            assert_ne!(
                tok_count, count,
                "\"{}\" operator should be non-reflexive for tokens",
                o
            );
        }
    }
}

#[ignore]
#[test]
fn reorder_and_negation() {
    let cs = CORPUS_STORAGE.as_ref().unwrap().lock().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "pos=/V.*/ _=_ tok !->dep tok? & #1 _o_ s",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs.count(q);
    assert_eq!(true, result.is_ok());
}
