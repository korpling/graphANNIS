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

#[ignore]
#[test]
fn exclude_optional_node_in_between() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: r#"entity="person" !_o_ q? & infstat="giv" & #1 _r_ #3"#,
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(1),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    // Only node #1 and #3 should be part of the output
    assert_eq!(vec!["ref::entity::GUM/GUM_interview_ants#referent_291 ref::infstat::GUM/GUM_interview_ants#referent_321"], result);
}

#[ignore]
#[test]
fn exclude_optional_node_at_end() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["GUM"],
        query: r#"entity="person" _r_ infstat="giv" &  q? & #1 !_o_ #3"#,
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs
        .find(
            q.clone(),
            0,
            Some(1),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    // Only node #1 and #2 should be part of the output
    assert_eq!(vec!["ref::entity::GUM/GUM_interview_ants#referent_291 ref::infstat::GUM/GUM_interview_ants#referent_321"], result);
}

#[ignore]
#[test]
fn token_search_loads_components_for_leaf_filter() {
    if let Some(cs) = CORPUS_STORAGE.as_ref() {
        for (query, expected_count) in [
            ("tok", 11),
            ("tok=\"example\"", 1),
            ("tok=/example/", 1),
            ("tok!=\"example\"", 10),
            ("tok!=/example/", 10),
        ] {
            let query = SearchQuery {
                corpus_names: &["subtok.demo"],
                query,
                query_language: QueryLanguage::AQL,
                timeout: None,
            };

            // Unload corpus to test that query loads components necessary to filter for leaves
            cs.unload("subtok.demo").unwrap();
            let count = cs.count(query).unwrap();

            assert_eq!(count, expected_count);
        }
    }
}

#[ignore]
#[test]
fn negative_token_search_applies_leaf_filter() {
    if let Some(cs) = CORPUS_STORAGE.as_ref() {
        // `node_name=/.*/` causes the token query to become the RHS of the join
        for (query, expected_count) in [
            ("node_name=/.*/ _ident_ tok!=\"example\"", 10),
            ("node_name=/.*/ _ident_ tok!=/example/", 10),
        ] {
            let query = SearchQuery {
                corpus_names: &["subtok.demo"],
                query,
                query_language: QueryLanguage::AQL,
                timeout: None,
            };

            let count = cs.count(query).unwrap();

            assert_eq!(count, expected_count);
        }
    }
}

#[ignore]
#[test]
fn no_error_on_large_token_distance() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let q = SearchQuery {
        corpus_names: &["subtok.demo"],
        query: r#"tok .100 tok"#,
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    // There should be an empty result, but no error
    let result = cs
        .find(
            q.clone(),
            0,
            Some(1),
            graphannis::corpusstorage::ResultOrder::Normal,
        )
        .unwrap();

    assert_eq!(0, result.len());
}

#[ignore]
#[test]
fn legacy_meta_query_with_multiple_alternatives() {
    let cs = CORPUS_STORAGE.as_ref().unwrap();

    // "buy" and "favorite" each appear exactly once in type="interview" and once in type="news"
    let query = SearchQuery {
        corpus_names: &["GUM"],
        query: "\"buy\" | \"favorite\" & meta::type=\"interview\"",
        query_language: QueryLanguage::AQLQuirksV3,
        timeout: None,
    };

    let count = cs.count(query).unwrap();

    assert_eq!(count, 2);
}
