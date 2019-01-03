extern crate graphannis;

use graphannis::corpusstorage::QueryLanguage;
use graphannis::CorpusStorage;

use std::cell::RefCell;
use std::path::PathBuf;

use std::collections::HashSet;

thread_local! {
   pub static CORPUS_STORAGE : RefCell<Option<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::with_auto_cache_size(&db_dir, false).ok()
        } else {
            None
        };
        return RefCell::new(cs);
    };
    pub static CORPUS_STORAGE_PARALLEL : RefCell<Option<CorpusStorage>> = {
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

include!(concat!(env!("OUT_DIR"), "/searchtest.rs"));

#[ignore]
#[test]
fn non_reflexivity_nodes() {
    CORPUS_STORAGE.with(|cs| {
        if let Some(ref cs) = *cs.borrow() {
            if let Ok(corpora) = cs.list() {
                let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
                // ignore of corpus does not exist
                if corpora.contains("GUM") {
                    let node_count = cs.count("GUM", "node", QueryLanguage::AQL).unwrap_or(0);

                    let operators_to_test = vec![
                        ".", ".*", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_", "->dep", "->dep *",
                    ];

                    for o in operators_to_test.into_iter() {
                        let count = cs
                            .count("GUM", &format!("node {} node", o), QueryLanguage::AQL)
                            .unwrap_or(0);
                        assert_ne!(
                            node_count, count,
                            "\"{}\" operator should be non-reflexive for nodes",
                            o
                        );
                    }
                }
            }
        }
    });
}


#[ignore]
#[test]
fn non_reflexivity_tokens() {
    CORPUS_STORAGE.with(|cs| {
        if let Some(ref cs) = *cs.borrow() {
            if let Ok(corpora) = cs.list() {
                let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
                // ignore of corpus does not exist
                if corpora.contains("GUM") {
                    let tok_count = cs.count("GUM", "tok", QueryLanguage::AQL).unwrap_or(0);

                    let operators_to_test = vec![
                        ".", ".*", ">", ">*", "_=_", "_i_", "_o_", "_l_", "_r_", "->dep", "->dep *",
                    ];

                    for o in operators_to_test.into_iter() {
                        let count = cs
                            .count("GUM", &format!("tok {} tok", o), QueryLanguage::AQL)
                            .unwrap_or(0);
                        assert_ne!(
                            tok_count, count,
                            "\"{}\" operator should be non-reflexive for tokens",
                            o
                        );
                    }
                }
            }
        }
    });
}
