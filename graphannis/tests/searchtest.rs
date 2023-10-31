extern crate graphannis;
#[macro_use]
extern crate lazy_static;

use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::CorpusStorage;

use std::path::PathBuf;

lazy_static! {
    static ref CORPUS_STORAGE: Option<CorpusStorage> = {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("../data")
        });
        let cs = CorpusStorage::with_auto_cache_size(&db_dir, true).unwrap();
        Some(cs)
    };
}

include!(concat!(env!("OUT_DIR"), "/searchtest.rs"));

#[ignore]
#[test]
fn non_reflexivity_nodes() {
    if let Some(cs) = CORPUS_STORAGE.as_ref() {
        let node_count = {
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
    if let Some(cs) = CORPUS_STORAGE.as_ref() {
        let tok_count = {
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
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "pos=/V.*/ _=_ tok !->dep tok? & #1 _o_ s",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs.count(q);
    assert_eq!(true, result.is_ok());
}

#[ignore]
#[test]
fn find_order() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    // This is defacto a token search, but not presorted
    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "pos",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    assert_eq!(
        vec![
            "GUM::pos::GUM/GUM_interview_ants#tok_1",
            "GUM::pos::GUM/GUM_interview_ants#tok_2",
            "GUM::pos::GUM/GUM_interview_ants#tok_3",
            "GUM::pos::GUM/GUM_interview_ants#tok_4",
            "GUM::pos::GUM/GUM_interview_ants#tok_5",
        ],
        result
    );

    let result = cs
        .find(
            q,
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Inverted,
        )
        .unwrap();

    assert_eq!(
        vec![
            "GUM::pos::GUM/GUM_whow_skittles#tok_954",
            "GUM::pos::GUM/GUM_whow_skittles#tok_953",
            "GUM::pos::GUM/GUM_whow_skittles#tok_952",
            "GUM::pos::GUM/GUM_whow_skittles#tok_951",
            "GUM::pos::GUM/GUM_whow_skittles#tok_950",
        ],
        result
    );
}

#[ignore]
#[test]
fn find_order_presorted() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "tok",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    assert_eq!(
        vec![
            "GUM/GUM_interview_ants#tok_1",
            "GUM/GUM_interview_ants#tok_2",
            "GUM/GUM_interview_ants#tok_3",
            "GUM/GUM_interview_ants#tok_4",
            "GUM/GUM_interview_ants#tok_5",
        ],
        result
    );

    let result = cs
        .find(
            q,
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Inverted,
        )
        .unwrap();

    assert_eq!(
        vec![
            "GUM/GUM_whow_skittles#tok_954",
            "GUM/GUM_whow_skittles#tok_953",
            "GUM/GUM_whow_skittles#tok_952",
            "GUM/GUM_whow_skittles#tok_951",
            "GUM/GUM_whow_skittles#tok_950",
        ],
        result
    );
}

#[ignore]
#[test]
fn meta_node_output_quirks() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "\"researching\" & meta::type=\"interview\"",
        query_language: QueryLanguage::AQLQuirksV3,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    // Only the token node should be part of the output
    assert_eq!(vec!["GUM/GUM_interview_ants#tok_309",], result);
}

#[ignore]
#[test]
fn meta_node_output_standard() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: "\"researching\" @* type=\"interview\"",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(5),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    // Both nodes should be part of the output
    assert_eq!(
        vec!["GUM/GUM_interview_ants#tok_309 type::GUM/GUM_interview_ants",],
        result
    );
}
