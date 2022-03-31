extern crate log;
extern crate tempfile;

use std::path::PathBuf;

use crate::annis::db::corpusstorage::get_read_or_error;
use crate::annis::db::{aql::model::AnnotationComponentType, example_generator};
use crate::annis::errors::GraphAnnisError;
use crate::corpusstorage::{ImportFormat, QueryLanguage, ResultOrder};
use crate::errors::Result;
use crate::update::{GraphUpdate, UpdateEvent};
use crate::{AnnotationGraph, CorpusStorage};
use graphannis_core::annostorage::{AnnotationStorage, ValueSearch};
use graphannis_core::graph::NODE_NAME_KEY;
use graphannis_core::types::Edge;
use graphannis_core::{graph::DEFAULT_NS, types::NodeID};
use itertools::Itertools;
use malloc_size_of::MallocSizeOf;

use super::SearchQuery;

#[test]
fn delete() {
    let tmp = tempfile::tempdir().unwrap();
    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
    // fully load a corpus
    let mut g = GraphUpdate::new();
    g.add_event(UpdateEvent::AddNode {
        node_name: "test".to_string(),
        node_type: "node".to_string(),
    })
    .unwrap();

    cs.apply_update("testcorpus", &mut g).unwrap();
    cs.preload("testcorpus").unwrap();
    cs.delete("testcorpus").unwrap();
}

#[test]
fn load_cs_twice() {
    let tmp = tempfile::tempdir().unwrap();
    {
        let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
        let mut g = GraphUpdate::new();
        g.add_event(UpdateEvent::AddNode {
            node_name: "test".to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();

        cs.apply_update("testcorpus", &mut g).unwrap();
    }

    {
        let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
        let mut g = GraphUpdate::new();
        g.add_event(UpdateEvent::AddNode {
            node_name: "test".to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();

        cs.apply_update("testcorpus", &mut g).unwrap();
    }
}

#[test]
fn apply_update_add_and_delete_nodes() {
    let tmp = tempfile::tempdir().unwrap();
    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

    let mut g = GraphUpdate::new();
    example_generator::create_corpus_structure(&mut g);
    example_generator::create_tokens(&mut g, Some("root/subCorpus1/doc1"));
    example_generator::create_tokens(&mut g, Some("root/subCorpus1/doc2"));

    g.add_event(UpdateEvent::AddEdge {
        source_node: "root/subCorpus1/doc1#tok1".to_owned(),
        target_node: "root/subCorpus1/doc1#tok2".to_owned(),
        layer: "dep".to_owned(),
        component_type: "Pointing".to_owned(),
        component_name: "dep".to_owned(),
    })
    .unwrap();

    cs.apply_update("root", &mut g).unwrap();

    let node_query = SearchQuery {
        corpus_names: &["root"],
        query: "node",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };

    let node_count = cs.count(node_query.clone()).unwrap();
    assert_eq!(22, node_count);

    let dep_query = SearchQuery {
        corpus_names: &["root"],
        query: "node ->dep node",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let edge_count = cs.count(dep_query.clone()).unwrap();
    assert_eq!(1, edge_count);

    // delete one of the tokens
    let mut g = GraphUpdate::new();
    g.add_event(UpdateEvent::DeleteNode {
        node_name: "root/subCorpus1/doc1#tok2".to_string(),
    })
    .unwrap();
    cs.apply_update("root", &mut g).unwrap();

    let node_count = cs.count(node_query).unwrap();
    assert_eq!(21, node_count);
    let edge_count = cs.count(dep_query).unwrap();
    assert_eq!(0, edge_count);
}

#[test]
fn subgraph_with_segmentation() {
    let tmp = tempfile::tempdir().unwrap();
    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

    let mut g = GraphUpdate::new();
    // Add corpus structure
    example_generator::create_corpus_structure_simple(&mut g);
    // Use the default tokenization as minimal tokens
    example_generator::create_tokens(&mut g, Some("root/doc1"));

    // Add first segmentation
    let seg_tokens = vec![
        "Is this example",
        "more complicated",
        "than it appears to be",
        "?",
    ];
    for (i, t) in seg_tokens.iter().enumerate() {
        let node_name = format!("root/doc1#seg{}", i);
        example_generator::create_token_node(&mut g, &node_name, t, Some("root/doc1"));
        g.add_event(UpdateEvent::AddNodeLabel {
            node_name,
            anno_ns: "default_ns".to_string(),
            anno_name: "seg".to_string(),
            anno_value: t.to_string(),
        })
        .unwrap();
    }
    for i in 0..seg_tokens.len() {
        g.add_event(UpdateEvent::AddEdge {
            source_node: format!("root/doc1#seg{}", i),
            target_node: format!("root/doc1#seg{}", i + 1),
            layer: DEFAULT_NS.to_string(),
            component_type: "Ordering".to_string(),
            component_name: "seg".to_string(),
        })
        .unwrap();
    }
    // add coverage for seg
    example_generator::make_span(
        &mut g,
        "root/doc1#seg0",
        &["root/doc1#tok0", "root/doc1#tok1", "root/doc1#tok2"],
    );
    example_generator::make_span(
        &mut g,
        "root/doc1#seg1",
        &["root/doc1#tok3", "root/doc1#tok4"],
    );
    example_generator::make_span(
        &mut g,
        "root/doc1#seg2",
        &[
            "root/doc1#tok5",
            "root/doc1#tok6",
            "root/doc1#tok7",
            "root/doc1#tok8",
            "root/doc1#tok9",
        ],
    );
    example_generator::make_span(&mut g, "root/doc1#seg3", &["root/doc1#tok10"]);

    cs.apply_update("root", &mut g).unwrap();

    let query = SearchQuery {
        corpus_names: &["root"],
        query: "node .seg,1,2 node",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };

    assert_eq!(5, cs.count(query).unwrap());

    // get the subgraph with context 1 on dipl
    let graph = cs
        .subgraph(
            "root",
            vec!["root/doc1#seg1".to_string()],
            1,
            1,
            Some("seg".to_string()),
        )
        .unwrap();

    let cov_components = graph.get_all_components(Some(AnnotationComponentType::Coverage), None);
    assert_eq!(1, cov_components.len());

    let gs_cov = graph.get_graphstorage(&cov_components[0]).unwrap();

    let segl0_id = graph
        .get_node_id_from_name("root/doc1#seg0")
        .unwrap()
        .unwrap();
    let seg0_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(segl0_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(3, seg0_out.unwrap().len());

    let seg1_id = graph
        .get_node_id_from_name("root/doc1#seg1")
        .unwrap()
        .unwrap();
    let seg1_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(seg1_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(2, seg1_out.unwrap().len());

    let seg2_id = graph
        .get_node_id_from_name("root/doc1#seg2")
        .unwrap()
        .unwrap();
    let seg2_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(seg2_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(5, seg2_out.unwrap().len());

    assert_eq!(None, graph.get_node_id_from_name("root/doc1#seg3").unwrap());
}

fn compare_annos<T>(
    annos1: &dyn AnnotationStorage<T>,
    annos2: &dyn AnnotationStorage<T>,
    items1: &[T],
    items2: &[T],
) where
    T: Send + Sync + MallocSizeOf,
{
    assert_eq!(items1.len(), items2.len());
    for i in 0..items1.len() {
        let mut annos1 = annos1.get_annotations_for_item(&items1[i]).unwrap();
        annos1.sort();
        let mut annos2 = annos2.get_annotations_for_item(&items2[i]).unwrap();
        annos2.sort();
        assert_eq!(annos1, annos2);
    }
}

fn compare_corpora(g1: &AnnotationGraph, g2: &AnnotationGraph, rhs_remove_annis_coverage: bool) {
    // Check all nodes and node annotations exist in both corpora
    let nodes1: Vec<String> = g1
        .get_node_annos()
        .exact_anno_search(Some("annis"), "node_name", ValueSearch::Any)
        .filter_map(|m| m.unwrap().extract_annotation(g1.get_node_annos()).unwrap())
        .map(|a| a.val.into())
        .sorted()
        .collect();
    let nodes2: Vec<String> = g2
        .get_node_annos()
        .exact_anno_search(Some("annis"), "node_name", ValueSearch::Any)
        .filter_map(|m| m.unwrap().extract_annotation(g1.get_node_annos()).unwrap())
        .map(|a| a.val.into())
        .sorted()
        .collect();
    assert_eq!(&nodes1, &nodes2);

    let nodes1: Vec<NodeID> = nodes1
        .into_iter()
        .filter_map(|n| g1.get_node_id_from_name(&n).unwrap())
        .collect();
    let nodes2: Vec<NodeID> = nodes2
        .into_iter()
        .filter_map(|n| g2.get_node_id_from_name(&n).unwrap())
        .collect();
    compare_annos(g1.get_node_annos(), g2.get_node_annos(), &nodes1, &nodes2);

    // Check that the graphs have the same edges
    let mut components1 = g1.get_all_components(None, None);
    components1.sort();
    let mut components2 = g2.get_all_components(None, None);
    if rhs_remove_annis_coverage {
        // Remove the special annis coverage component created during relANNIS import
        components2.retain(|c| {
            c.get_type() != AnnotationComponentType::Coverage
                || !c.name.is_empty()
                || c.layer != "annis"
        });
    }
    components2.sort();
    assert_eq!(components1, components2);

    for c in components1 {
        let gs1 = g1.get_graphstorage_as_ref(&c).unwrap();
        let gs2 = g2.get_graphstorage_as_ref(&c).unwrap();

        for i in 0..nodes1.len() {
            let start1 = nodes1[i];
            let start2 = nodes2[i];

            // Check all connected nodes for this edge
            let targets1: Result<Vec<String>> = gs1
                .get_outgoing_edges(start1)
                .filter_map_ok(|target| {
                    g1.get_node_annos()
                        .get_value_for_item(&target, &NODE_NAME_KEY)
                        .unwrap()
                })
                .map_ok(|n| n.into())
                .map(|n| n.map_err(GraphAnnisError::from))
                .collect();
            let mut targets1 = targets1.unwrap();
            targets1.sort();

            let targets2: Result<Vec<String>> = gs2
                .get_outgoing_edges(start2)
                .filter_map_ok(|target| {
                    g2.get_node_annos()
                        .get_value_for_item(&target, &NODE_NAME_KEY)
                        .unwrap()
                })
                .map(|n| n.map_err(GraphAnnisError::from))
                .map_ok(|n| n.to_string())
                .collect();
            let mut targets2 = targets2.unwrap();
            targets2.sort();
            assert_eq!(targets1, targets2);

            // Check the edge annotations for each edge
            let edges1: Vec<Edge> = targets1
                .iter()
                .map(|t| Edge {
                    source: start1,
                    target: g1.get_node_id_from_name(t).unwrap().unwrap(),
                })
                .collect();
            let edges2: Vec<Edge> = targets2
                .iter()
                .map(|t| Edge {
                    source: start2,
                    target: g2.get_node_id_from_name(t).unwrap().unwrap(),
                })
                .collect();
            compare_annos(
                gs1.get_anno_storage(),
                gs2.get_anno_storage(),
                &edges1,
                &edges2,
            );
        }
    }
}

#[test]
fn import_salt_sample() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    // Import both the GraphML and the relANNIS files as corpus
    cs.import_from_fs(
        &cargo_dir.join("tests/SaltSampleCorpus"),
        ImportFormat::RelANNIS,
        Some("test-relannis".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();
    cs.import_from_fs(
        &cargo_dir.join("tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("test-graphml".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    // compare both corpora, they should be exactly equal
    let e1 = cs.get_fully_loaded_entry("test-graphml").unwrap();
    let lock1 = e1.read().unwrap();
    let db1 = get_read_or_error(&lock1).unwrap();

    let e2 = cs.get_fully_loaded_entry("test-relannis").unwrap();
    let lock2 = e2.read().unwrap();
    let db2 = get_read_or_error(&lock2).unwrap();

    compare_corpora(db1, db2, true);
}

#[test]
fn import_special_character_corpus_name() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    let corpus_name = cs
        .import_from_fs(
            &cargo_dir.join("tests/SpecialCharCorpusName"),
            ImportFormat::RelANNIS,
            None,
            false,
            true,
            |_| {},
        )
        .unwrap();
    assert_eq!("Root:: Cörp/u%s", &corpus_name);

    // Check that the special corpus name can be queried
    let q = SearchQuery {
        corpus_names: &vec![&corpus_name],
        query: "lemma",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let token = cs.count_extra(q.clone()).unwrap();
    assert_eq!(44, token.match_count);
    assert_eq!(4, token.document_count);

    let matches = cs.find(q, 0, Some(1), ResultOrder::Normal).unwrap();
    assert_eq!(1, matches.len());
    assert_eq!(
        "salt::lemma::Root%3A%3A%20C%C3%B6rp%2Fu%25s/subCorpus1/doc1#sTok1",
        matches[0]
    );

    let q_quirks = SearchQuery {
        corpus_names: &vec![&corpus_name],
        query: "lemma",
        query_language: QueryLanguage::AQLQuirksV3,
        timeout: None,
    };
    let matches_quirks = cs.find(q_quirks, 0, Some(1), ResultOrder::Normal).unwrap();
    assert_eq!(1, matches_quirks.len());
    assert_eq!(
        "salt::lemma::Root::%20C%C3%B6rp%2Fu%25s/subCorpus1/doc1#sTok1",
        matches_quirks[0]
    );
}

#[test]
fn load_legacy_binary_corpus() {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let data_dir = cargo_dir.join("tests/data");

    let cs = CorpusStorage::with_auto_cache_size(&data_dir, true).unwrap();

    // load both legacy corpora and compare them
    let e1 = cs.get_fully_loaded_entry("sample-disk-based").unwrap();
    let lock1 = e1.read().unwrap();
    let db1 = get_read_or_error(&lock1).unwrap();

    let e2 = cs.get_fully_loaded_entry("sample-memory-based").unwrap();
    let lock2 = e2.read().unwrap();
    let db2 = get_read_or_error(&lock2).unwrap();

    compare_corpora(db1, db2, false);
}
