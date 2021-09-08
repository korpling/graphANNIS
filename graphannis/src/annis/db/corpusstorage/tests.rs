extern crate log;
extern crate tempfile;

use std::path::PathBuf;

use crate::annis::db::corpusstorage::get_read_or_error;
use crate::annis::db::{aql::model::AnnotationComponentType, example_generator};
use crate::corpusstorage::{ImportFormat, QueryLanguage};
use crate::update::{GraphUpdate, UpdateEvent};
use crate::CorpusStorage;
use graphannis_core::annostorage::ValueSearch;
use graphannis_core::{graph::DEFAULT_NS, types::NodeID};
use itertools::Itertools;

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
            node_name: node_name,
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

    let segl0_id = graph.get_node_id_from_name("root/doc1#seg0").unwrap();
    let seg0_out: Vec<NodeID> = gs_cov.get_outgoing_edges(segl0_id).collect();
    assert_eq!(3, seg0_out.len());

    let seg1_id = graph.get_node_id_from_name("root/doc1#seg1").unwrap();
    let seg1_out: Vec<NodeID> = gs_cov.get_outgoing_edges(seg1_id).collect();
    assert_eq!(2, seg1_out.len());

    let seg2_id = graph.get_node_id_from_name("root/doc1#seg2").unwrap();
    let seg2_out: Vec<NodeID> = gs_cov.get_outgoing_edges(seg2_id).collect();
    assert_eq!(5, seg2_out.len());

    assert_eq!(None, graph.get_node_id_from_name("root/doc1#seg3"));
}

#[test]
fn import_salt_sample() {
    let tmp = tempfile::tempdir().unwrap();
    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    // Import both the GraphML and the relANNIS files as corpus
    cs.import_from_fs(
        &PathBuf::from("tests/SaltSampleCorpus"),
        ImportFormat::RelANNIS,
        Some("test-relannis".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();
    cs.import_from_fs(
        &PathBuf::from("tests/SaltSampleCorpus.graphml"),
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

    // Check all nodes and node annotations exist in both corpora
    let nodes1: Vec<String> = db1
        .get_node_annos()
        .exact_anno_search(Some("annis"), "node_name", ValueSearch::Any)
        .filter_map(|m| m.extract_annotation(db1.get_node_annos()))
        .map(|a| a.val.into())
        .sorted()
        .collect();
    let nodes2: Vec<String> = db2
        .get_node_annos()
        .exact_anno_search(Some("annis"), "node_name", ValueSearch::Any)
        .filter_map(|m| m.extract_annotation(db1.get_node_annos()))
        .map(|a| a.val.into())
        .sorted()
        .collect();
    assert_eq!(&nodes1, &nodes2);
    for n in &nodes1 {
        let id1 = db1.get_node_id_from_name(n).unwrap();
        let id2 = db2.get_node_id_from_name(n).unwrap();
        let mut annos1 = db1.get_node_annos().get_annotations_for_item(&id1);
        annos1.sort();
        let mut annos2 = db2.get_node_annos().get_annotations_for_item(&id2);
        annos2.sort();
        assert_eq!(annos1, annos2);
    }
}
