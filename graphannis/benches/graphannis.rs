#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;
extern crate graphannis;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use fake::Fake;
use fake::faker::name::raw::*;
use fake::locales::*;
use graphannis::CorpusStorage;
use graphannis::corpusstorage::ResultOrder;
use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::update::{GraphUpdate, UpdateEvent};
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

lazy_static! {

static ref CORPUS_STORAGE : Option<CorpusStorage> = {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        }
        else if Path::new("data").is_dir() {
            String::from("data")
        } else {
            String::from("../data")
        });

        // only execute the test if the directory exists
        if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::with_auto_cache_size(&db_dir, false).ok()
        } else {
            None
        }
    };
}

const TOK_COUNT: usize = 100_000;

fn find_all_nouns_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }

    bench.bench_function("find_all_nouns_gum", move |b| {
        cs.preload("GUM").unwrap();
        b.iter(|| {
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "pos=\"NN\"",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            let f = cs.find(query, 0, None, ResultOrder::Normal);
            assert!(f.is_ok());
        })
    });
}

fn find_first_ten_nouns_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }

    bench.bench_function("find_first_ten_nouns_gum", move |b| {
        cs.preload("GUM").unwrap();
        b.iter(|| {
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "pos=\"NN\"",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            let f = cs.find(query, 0, Some(10), ResultOrder::Normal);
            assert!(f.is_ok());
        })
    });
}

fn count_extra_coref_anno(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if !corpora.contains("GUM") {
            return;
        }
    }

    bench.bench_function("count_extra_coref_anno", move |b| {
        cs.preload("GUM").unwrap();
        let query = SearchQuery {
            corpus_names: &["GUM"],
            query: "infstat=\"giv\" ->coref[type=\"coref\"] entity=\"person\"",
            query_language: QueryLanguage::AQL,
            timeout: None,
        };
        b.iter(move || {
            let r = cs.count_extra(query.clone());
            assert!(r.is_ok());
        })
    });
}

fn count_extra_node(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if !corpora.contains("GUM") {
            return;
        }
    }

    bench.bench_function("count_extra_node", move |b| {
        cs.preload("GUM").unwrap();
        let query = SearchQuery {
            corpus_names: &["GUM"],
            query: "node",
            query_language: QueryLanguage::AQL,
            timeout: None,
        };

        b.iter(move || {
            let r = cs.count_extra(query.clone());
            assert!(r.is_ok());
        })
    });
}

fn find_first_ten_token_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }

    bench.bench_function("find_first_ten_token_gum", move |b| {
        cs.preload("GUM").unwrap();
        b.iter(|| {
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "tok",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            let f = cs.find(query, 0, Some(10), ResultOrder::Normal);
            assert!(f.is_ok());
        })
    });
}

fn deserialize_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();
    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore if corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }

    bench.bench_function("deserialize_gum", move |b| {
        b.iter(|| {
            cs.unload("GUM").unwrap();
            cs.preload("GUM").unwrap();
        });
    });
}

/// Creates graph updates for adding some token (first result) and deleting them
/// again (second result item).
fn create_example_update() -> (GraphUpdate, GraphUpdate) {
    // Create a set of graph updates to apply
    let mut updates_add = GraphUpdate::default();

    // Generate a lot of tokens made of fake strings (using names)
    let mut token_names: Vec<String> = Vec::with_capacity(TOK_COUNT);
    let mut previous_token_name = None;
    for i in 0..TOK_COUNT {
        let node_name = format!("n{}", i);

        let t: &str = LastName(EN).fake();

        token_names.push(node_name.clone());

        // Create token node
        updates_add
            .add_event(UpdateEvent::AddNode {
                node_name: node_name.clone(),
                node_type: "node".to_string(),
            })
            .unwrap();
        updates_add
            .add_event(UpdateEvent::AddNodeLabel {
                node_name: node_name.clone(),
                anno_ns: "annis".to_string(),
                anno_name: "tok".to_string(),
                anno_value: t.to_string(),
            })
            .unwrap();

        // add the order relation
        if let Some(previous_token_name) = previous_token_name {
            updates_add
                .add_event(UpdateEvent::AddEdge {
                    source_node: previous_token_name,
                    target_node: node_name.clone(),
                    layer: "annis".to_string(),
                    component_type: "Ordering".to_string(),
                    component_name: "".to_string(),
                })
                .unwrap();
        }

        previous_token_name = Some(node_name);
    }

    // Add update events to delete the just added token
    let mut updates_delete = GraphUpdate::new();
    for t in token_names {
        updates_delete
            .add_event(UpdateEvent::DeleteNode {
                node_name: t.clone(),
            })
            .unwrap();
    }
    (updates_add, updates_delete)
}

fn apply_update_inmemory(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    bench.bench_function("apply_update_inmemory", move |b| {
        cs.create_empty_corpus("apply_update_test_corpus", false)
            .unwrap();
        let (mut u1, mut u2) = create_example_update();
        b.iter(|| {
            cs.apply_update("apply_update_test_corpus", &mut u1)
                .unwrap();
            cs.apply_update("apply_update_test_corpus", &mut u2)
                .unwrap()
        });
        cs.delete("apply_update_test_corpus").unwrap();
    });
}

fn apply_update_ondisk(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    bench.bench_function("apply_update_ondisk", move |b| {
        cs.create_empty_corpus("apply_update_test_corpus", true)
            .unwrap();
        let (mut u1, mut u2) = create_example_update();
        b.iter(|| {
            cs.apply_update("apply_update_test_corpus", &mut u1)
                .unwrap();
            cs.apply_update("apply_update_test_corpus", &mut u2)
                .unwrap()
        });
        cs.delete("apply_update_test_corpus").unwrap();
    });
}

criterion_group!(name=default; config= Criterion::default().sample_size(50); targets =
    deserialize_gum,
    find_first_ten_token_gum,
    find_first_ten_nouns_gum, 
    find_all_nouns_gum,
    count_extra_coref_anno, count_extra_node);
criterion_group!(name=apply_update; config= Criterion::default().sample_size(20); targets =
    apply_update_inmemory,
    apply_update_ondisk,
);
criterion_main!(default, apply_update);
