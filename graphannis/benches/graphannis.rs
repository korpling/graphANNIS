#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;
extern crate graphannis;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use graphannis::corpusstorage::ResultOrder;
use graphannis::corpusstorage::{QueryLanguage, SearchQuery};
use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::collections::HashSet;
use std::path::PathBuf;

lazy_static! {

static ref CORPUS_STORAGE : Option<CorpusStorage> = {
        let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
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

fn find_all_nouns_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    let corpora = cs.list();
    if let Ok(corpora) = corpora {
        let corpora: HashSet<String> = corpora.into_iter().map(|c| c.name).collect();
        // ignore of corpus does not exist
        if corpora.contains("GUM") {
            cs.preload("GUM").unwrap();
        } else {
            return;
        }
    }

    bench.bench_function("find_all_nouns_gum", move |b| {
        b.iter(|| {
            let query = SearchQuery {
                corpus_names: &["GUM"],
                query: "pos=\"NN\"",
                query_language: QueryLanguage::AQL,
                timeout: None,
            };
            let f = cs.find(query, usize::min_value(), None, ResultOrder::Normal);
            assert!(f.is_ok());
        })
    });
}

fn deserialize_gum(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    bench.bench_function("deserialize_gum", move |b| {
        b.iter(|| {
            cs.unload("GUM");
            cs.preload("GUM").unwrap();
        });
    });
}

fn apply_update(bench: &mut Criterion) {
    if CORPUS_STORAGE.is_none() {
        return;
    }

    let cs = CORPUS_STORAGE.as_ref().unwrap();

    // Create a set of graph updates to apply
    let mut u = GraphUpdate::default();

    // Generate a lot of tokens made of fake strings (using names)
    const TOK_COUNT: usize = 2000;
    let mut token_names: Vec<String> = Vec::with_capacity(TOK_COUNT);
    let mut previous_token_name = None;
    for i in 0..TOK_COUNT {
        let node_name = format!("n{}", i);

        let t: &str = LastName(EN).fake();

        token_names.push(node_name.clone());

        // Create token node
        u.add_event(UpdateEvent::AddNode {
            node_name: node_name.clone(),
            node_type: "node".to_string(),
        })
        .unwrap();
        u.add_event(UpdateEvent::AddNodeLabel {
            node_name: node_name.clone(),
            anno_ns: "annis".to_string(),
            anno_name: "tok".to_string(),
            anno_value: t.to_string(),
        })
        .unwrap();

        // add the order relation
        if let Some(previous_token_name) = previous_token_name {
            u.add_event(UpdateEvent::AddEdge {
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

    bench.bench_function("apply_update", move |b| {
        b.iter(|| cs.apply_update("apply_update_test_corpus", &mut u).unwrap());
    });

    cs.delete("apply_update_test_corpus").unwrap();
}

criterion_group!(name=long_running; config= Criterion::default().sample_size(10); targets = find_all_nouns_gum);
criterion_group!(name=short_running; config= Criterion::default().sample_size(50); targets = apply_update, deserialize_gum);
criterion_main!(long_running, short_running);
