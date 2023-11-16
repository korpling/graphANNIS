extern crate log;
extern crate tempfile;

use same_file::is_same_file;
use std::path::{Path, PathBuf};
use std::vec;

use crate::annis::db::corpusstorage::get_read_or_error;
use crate::annis::db::{aql::model::AnnotationComponentType, example_generator};
use crate::annis::errors::GraphAnnisError;
use crate::corpusstorage::{ImportFormat, QueryLanguage, ResultOrder};
use crate::errors::Result;
use crate::update::{GraphUpdate, UpdateEvent};
use crate::{AnnotationGraph, CorpusStorage};
use graphannis_core::annostorage::{EdgeAnnotationStorage, NodeAnnotationStorage, ValueSearch};
use graphannis_core::graph::{ANNIS_NS, NODE_NAME_KEY};
use graphannis_core::types::{Component, Edge};
use graphannis_core::{graph::DEFAULT_NS, types::NodeID};
use itertools::Itertools;
use pretty_assertions::assert_eq;

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
    example_generator::create_tokens(
        &mut g,
        Some("root/subCorpus1/doc1"),
        Some("root/subCorpus1/doc1"),
    );
    example_generator::create_tokens(
        &mut g,
        Some("root/subCorpus1/doc2"),
        Some("root/subCorpus1/doc2"),
    );

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

fn create_simple_graph(cs: &mut CorpusStorage) {
    let mut complete_graph_def = GraphUpdate::new();
    // Add corpus structure
    example_generator::create_corpus_structure_simple(&mut complete_graph_def);
    // Use the default tokenization as minimal tokens
    example_generator::create_tokens(
        &mut complete_graph_def,
        Some("root/doc1"),
        Some("root/doc1#text1"),
    );

    // Add some spans
    example_generator::make_span(
        &mut complete_graph_def,
        "root/doc1#span1",
        &["root/doc1#tok1", "root/doc1#tok2"],
        true,
    );

    complete_graph_def
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1#span1".to_string(),
            target_node: "root/doc1#text1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    example_generator::make_span(
        &mut complete_graph_def,
        "root/doc1#span2",
        &["root/doc1#tok3", "root/doc1#tok4", "root/doc1#tok5"],
        true,
    );
    complete_graph_def
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1#span2".to_string(),
            target_node: "root/doc1#text1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    example_generator::make_span(
        &mut complete_graph_def,
        "root/doc1#span3",
        &["root/doc1#tok5", "root/doc1#tok6", "root/doc1#tok7"],
        true,
    );
    complete_graph_def
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1#span3".to_string(),
            target_node: "root/doc1#text1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    example_generator::make_span(
        &mut complete_graph_def,
        "root/doc1#span4",
        &["root/doc1#tok9", "root/doc1#tok10"],
        true,
    );
    complete_graph_def
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1#span4".to_string(),
            target_node: "root/doc1#text1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    cs.apply_update("root", &mut complete_graph_def).unwrap();
}

#[test]
fn subgraphs_simple() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

    create_simple_graph(&mut cs);

    // get the subgraph for a token ("complicated")
    // This should return the following token and their covering spans
    // example[tok2] more[tok3] complicated[tok4] than[tok5] it[tok6] appears[tok7] to[tok8]
    let graph = cs
        .subgraph("root", vec!["root/doc1#tok4".to_string()], 2, 4, None)
        .unwrap();

    let cov_components = graph.get_all_components(Some(AnnotationComponentType::Coverage), None);
    assert_eq!(1, cov_components.len());
    let gs_cov = graph.get_graphstorage(&cov_components[0]).unwrap();

    let ordering_components =
        graph.get_all_components(Some(AnnotationComponentType::Ordering), Some(""));
    assert_eq!(1, ordering_components.len());
    let gs_ordering = graph.get_graphstorage(&ordering_components[0]).unwrap();

    // Check that all token exist and are connected
    for i in 2..8 {
        let t = format!("root/doc1#tok{}", i);
        let t_id = graph.get_node_annos().get_node_id_from_name(&t).unwrap();
        assert_eq!(true, t_id.is_some());
        let next_token = format!("root/doc1#tok{}", i + 1);
        let next_token_id = graph
            .get_node_annos()
            .get_node_id_from_name(&next_token)
            .unwrap();
        assert_eq!(true, next_token_id.is_some());
        assert_eq!(
            true,
            gs_ordering
                .is_connected(
                    t_id.unwrap(),
                    next_token_id.unwrap(),
                    1,
                    std::ops::Bound::Included(1)
                )
                .unwrap()
        );
    }
    // Also check, that the token outside the context do not exists
    for t in 0..2 {
        let t = format!("root/doc1#tok{}", t);
        assert_eq!(
            false,
            graph
                .get_node_annos()
                .get_node_id_from_name(&t)
                .unwrap()
                .is_some()
        );
    }
    for t in 9..10 {
        let t = format!("root/doc1#tok{}", t);
        assert_eq!(
            false,
            graph
                .get_node_annos()
                .get_node_id_from_name(&t)
                .unwrap()
                .is_some()
        );
    }

    // Check the (non-) existance of the spans
    let span1 = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#span1")
        .unwrap()
        .unwrap();
    let span2 = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#span2")
        .unwrap()
        .unwrap();
    let span3 = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#span3")
        .unwrap()
        .unwrap();

    assert_eq!(
        false,
        graph
            .get_node_annos()
            .get_node_id_from_name("root/doc1#span4")
            .unwrap()
            .is_some()
    );

    assert_eq!(1, gs_cov.get_outgoing_edges(span1).count());
    assert_eq!(3, gs_cov.get_outgoing_edges(span2).count());
    assert_eq!(3, gs_cov.get_outgoing_edges(span3).count());

    // Check that the corpus structure for the matched node is included
    let corpus_nodes: graphannis_core::errors::Result<Vec<_>> = graph
        .get_node_annos()
        .exact_anno_search(Some(ANNIS_NS), "node_type", ValueSearch::Some("corpus"))
        .collect();
    let corpus_nodes = corpus_nodes.unwrap();
    assert_eq!(2, corpus_nodes.len());
    let ds_nodes: graphannis_core::errors::Result<Vec<_>> = graph
        .get_node_annos()
        .exact_anno_search(Some(ANNIS_NS), "node_type", ValueSearch::Some("datasource"))
        .collect();
    let ds_nodes = ds_nodes.unwrap();
    assert_eq!(1, ds_nodes.len());

    let text_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#text1")
        .unwrap();
    assert_eq!(true, text_id.is_some());

    let doc_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1")
        .unwrap();
    assert_eq!(true, doc_id.is_some());

    let toplevel_id = graph
        .get_node_annos()
        .get_node_id_from_name("root")
        .unwrap();
    assert_eq!(true, toplevel_id.is_some());

    let part_of_components = graph.get_all_components(Some(AnnotationComponentType::PartOf), None);
    assert_eq!(1, part_of_components.len());
    let gs_partof = graph.get_graphstorage(&part_of_components[0]).unwrap();

    assert_eq!(
        doc_id.unwrap(),
        gs_partof
            .get_outgoing_edges(text_id.unwrap())
            .next()
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        toplevel_id.unwrap(),
        gs_partof
            .get_outgoing_edges(doc_id.unwrap())
            .next()
            .unwrap()
            .unwrap()
    );
}

#[test]
fn subgraphs_non_overlapping_regions() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

    create_simple_graph(&mut cs);

    // get the subgraph for a token ("example" and "it")
    // This should return the following token and their covering spans
    // this[tok1] example[tok2] more[tok3] ... than[tok5] it[tok6] appears[tok7]
    let graph = cs
        .subgraph(
            "root",
            vec!["root/doc1#tok2".to_string(), "root/doc1#tok6".to_string()],
            1,
            1,
            None,
        )
        .unwrap();

    // Check that all token exist and are connected
    let t1_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok1")
        .unwrap();
    assert_eq!(true, t1_id.is_some());
    let t2_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok2")
        .unwrap();
    assert_eq!(true, t2_id.is_some());
    let t3_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok3")
        .unwrap();
    assert_eq!(true, t3_id.is_some());

    let t5_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok5")
        .unwrap();
    assert_eq!(true, t5_id.is_some());
    let t6_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok6")
        .unwrap();
    assert_eq!(true, t6_id.is_some());
    let t7_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#tok7")
        .unwrap();
    assert_eq!(true, t7_id.is_some());

    let ordering_components =
        graph.get_all_components(Some(AnnotationComponentType::Ordering), Some(""));
    assert_eq!(1, ordering_components.len());
    let gs_ordering = graph.get_graphstorage(&ordering_components[0]).unwrap();

    assert_eq!(
        true,
        gs_ordering
            .is_connected(
                t1_id.unwrap(),
                t2_id.unwrap(),
                1,
                std::ops::Bound::Included(1)
            )
            .unwrap()
    );
    assert_eq!(
        true,
        gs_ordering
            .is_connected(
                t2_id.unwrap(),
                t3_id.unwrap(),
                1,
                std::ops::Bound::Included(1)
            )
            .unwrap()
    );
    assert_eq!(
        false,
        gs_ordering
            .is_connected(
                t3_id.unwrap(),
                t5_id.unwrap(),
                1,
                std::ops::Bound::Included(1)
            )
            .unwrap()
    );
    assert_eq!(
        true,
        gs_ordering
            .is_connected(
                t5_id.unwrap(),
                t6_id.unwrap(),
                1,
                std::ops::Bound::Included(1)
            )
            .unwrap()
    );
    assert_eq!(
        true,
        gs_ordering
            .is_connected(
                t6_id.unwrap(),
                t7_id.unwrap(),
                1,
                std::ops::Bound::Included(1)
            )
            .unwrap()
    );

    // The last and first node of the context region should be connected by a special ordering edge
    let gs_ds_ordering = graph
        .get_graphstorage(&Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "datasource-gap".into(),
        ))
        .unwrap();
    let out: graphannis_core::errors::Result<Vec<_>> =
        gs_ds_ordering.get_outgoing_edges(t3_id.unwrap()).collect();
    let out = out.unwrap();
    assert_eq!(vec![t5_id.unwrap()], out);
}

#[test]
fn subgraph_with_segmentation() {
    let tmp = tempfile::tempdir().unwrap();
    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

    let mut g = GraphUpdate::new();
    // Add corpus structure
    example_generator::create_corpus_structure_simple(&mut g);
    // Use the default tokenization as minimal tokens
    example_generator::create_tokens(&mut g, Some("root/doc1"), Some("root/doc1#text1"));

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
        false,
    );
    example_generator::make_span(
        &mut g,
        "root/doc1#seg1",
        &["root/doc1#tok3", "root/doc1#tok4"],
        false,
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
        false,
    );
    example_generator::make_span(&mut g, "root/doc1#seg3", &["root/doc1#tok10"], false);

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
        .get_node_annos()
        .get_node_id_from_name("root/doc1#seg0")
        .unwrap()
        .unwrap();
    let seg0_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(segl0_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(3, seg0_out.unwrap().len());

    let seg1_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#seg1")
        .unwrap()
        .unwrap();
    let seg1_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(seg1_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(2, seg1_out.unwrap().len());

    let seg2_id = graph
        .get_node_annos()
        .get_node_id_from_name("root/doc1#seg2")
        .unwrap()
        .unwrap();
    let seg2_out: Result<Vec<_>> = gs_cov
        .get_outgoing_edges(seg2_id)
        .map(|e| e.map_err(GraphAnnisError::from))
        .collect();
    assert_eq!(5, seg2_out.unwrap().len());

    assert_eq!(
        None,
        graph
            .get_node_annos()
            .get_node_id_from_name("root/doc1#seg3")
            .unwrap()
    );
}

/// Test that context generation works with a corpus that has segmentations
/// and gaps in the segments.
#[test]
fn subgraph_with_segmentation_and_gap() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    let corpus_name = cs
        .import_from_fs(
            &cargo_dir.join("tests/SegmentationWithGaps.graphml"),
            ImportFormat::GraphML,
            None,
            false,
            true,
            |_| {},
        )
        .unwrap();

    // Use the norm="Gaps" node as match which is an existing segmentation
    let m = vec!["SegmentationWithGaps/doc01#norm12".to_string()];

    // Get the context using tokens
    let g = cs.subgraph(&corpus_name, m.clone(), 1, 2, None).unwrap();
    // Check that all token and the page are included, including the token
    // that is not covered by a segmentation node.
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_11")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_12")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_13")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_14")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#page2")
        .unwrap()
        .is_some());

    // Get the context for the norm node using the norm segmentation
    let g = cs
        .subgraph(&corpus_name, m, 1, 1, Some("norm".to_string()))
        .unwrap();
    // Check that all token and the page are included
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_11")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_12")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_13")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_14")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#page2")
        .unwrap()
        .is_some());

    // Get the context for the token using the norm segmentation
    let g = cs
        .subgraph(
            &corpus_name,
            vec!["SegmentationWithGaps/doc01#tok_12".to_string()],
            1,
            1,
            Some("norm".to_string()),
        )
        .unwrap();
    // Check that all token and the page are included
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_11")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_12")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_13")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#tok_14")
        .unwrap()
        .is_some());
    assert!(g
        .get_node_annos()
        .get_node_id_from_name("SegmentationWithGaps/doc01#page2")
        .unwrap()
        .is_some());
}

fn compare_edge_annos(
    annos1: &dyn EdgeAnnotationStorage,
    annos2: &dyn EdgeAnnotationStorage,
    items1: &[Edge],
    items2: &[Edge],
) {
    assert_eq!(items1.len(), items2.len());
    for i in 0..items1.len() {
        let mut annos1 = annos1.get_annotations_for_item(&items1[i]).unwrap();
        annos1.sort();
        let mut annos2 = annos2.get_annotations_for_item(&items2[i]).unwrap();
        annos2.sort();
        assert_eq!(annos1, annos2);
    }
}

fn compare_node_annos(
    annos1: &dyn NodeAnnotationStorage,
    annos2: &dyn NodeAnnotationStorage,
    items1: &[NodeID],
    items2: &[NodeID],
) {
    assert_eq!(items1.len(), items2.len());
    for i in 0..items1.len() {
        let mut annos1 = annos1.get_annotations_for_item(&items1[i]).unwrap();
        annos1.sort();
        let mut annos2 = annos2.get_annotations_for_item(&items2[i]).unwrap();
        annos2.sort();
        assert_eq!(annos1, annos2);
    }
}

fn compare_corpora(g1: &AnnotationGraph, g2: &AnnotationGraph) {
    // Check all nodes and node annotations exist in both corpora
    let nodes1: Vec<String> = g1
        .get_node_annos()
        .exact_anno_search(Some(ANNIS_NS), "node_name", ValueSearch::Any)
        .filter_map(|m| m.unwrap().extract_annotation(g1.get_node_annos()).unwrap())
        .map(|a| a.val.into())
        .sorted()
        .collect();
    let nodes2: Vec<String> = g2
        .get_node_annos()
        .exact_anno_search(Some(ANNIS_NS), "node_name", ValueSearch::Any)
        .filter_map(|m| m.unwrap().extract_annotation(g1.get_node_annos()).unwrap())
        .map(|a| a.val.into())
        .sorted()
        .collect();
    assert_eq!(&nodes1, &nodes2);

    let nodes1: Vec<NodeID> = nodes1
        .into_iter()
        .filter_map(|n| g1.get_node_annos().get_node_id_from_name(&n).unwrap())
        .collect();
    let nodes2: Vec<NodeID> = nodes2
        .into_iter()
        .filter_map(|n| g2.get_node_annos().get_node_id_from_name(&n).unwrap())
        .collect();
    compare_node_annos(g1.get_node_annos(), g2.get_node_annos(), &nodes1, &nodes2);

    // Check that the graphs have the same edges
    let mut components1 = g1.get_all_components(None, None);
    components1.sort();
    let mut components2 = g2.get_all_components(None, None);
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
                    target: g1
                        .get_node_annos()
                        .get_node_id_from_name(t)
                        .unwrap()
                        .unwrap(),
                })
                .collect();
            let edges2: Vec<Edge> = targets2
                .iter()
                .map(|t| Edge {
                    source: start2,
                    target: g2
                        .get_node_annos()
                        .get_node_id_from_name(t)
                        .unwrap()
                        .unwrap(),
                })
                .collect();
            compare_edge_annos(
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
        &cargo_dir.join("tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("test-graphml".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    cs.import_from_fs(
        &cargo_dir.join("tests/SaltSampleCorpus"),
        ImportFormat::RelANNIS,
        Some("test-relannis".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    // compare both corpora, they should be exactly equal
    let entry_graphml = cs.get_fully_loaded_entry("test-graphml").unwrap();
    let lock_graphml = entry_graphml.read().unwrap();
    let db_graphml = get_read_or_error(&lock_graphml).unwrap();

    let entry_relannis = cs.get_fully_loaded_entry("test-relannis").unwrap();
    let lock_relannis = entry_relannis.read().unwrap();
    let db_relannis = get_read_or_error(&lock_relannis).unwrap();

    compare_corpora(db_graphml, db_relannis);
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
fn import_relative_corpus_with_linked_file() {
    let tmp = tempfile::tempdir().unwrap();

    // Set the relative path so that the corpus file is in the current folder
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(cargo_dir.join("tests")).unwrap();

    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    let corpus_name = cs
        .import_from_fs(
            Path::new("CorpusWithLinkedFile.graphml"),
            ImportFormat::GraphML,
            None,
            false,
            true,
            |_| {},
        )
        .unwrap();
    assert_eq!("CorpusWithLinkedFile", &corpus_name);
    // Check that the linked file was copied
    let entry = cs
        .get_loaded_entry("CorpusWithLinkedFile", false, false)
        .unwrap();
    let lock = entry.read().unwrap();
    let g: &AnnotationGraph = get_read_or_error(&lock).unwrap();

    let files = cs.get_linked_files("CorpusWithLinkedFile", &g).unwrap();
    let mut files = files.unwrap();
    let first_file = files.next().unwrap().unwrap();
    assert_eq!("linked_file.txt", first_file.0);
    assert!(is_same_file(
        tmp.path()
            .join("CorpusWithLinkedFile/files/linked_file.txt"),
        &first_file.1
    )
    .unwrap());
    let file_content = std::fs::read_to_string(first_file.1).unwrap();
    assert_eq!("The content of this file is not important.", file_content);
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

    compare_corpora(db1, db2);
}

/// This is a regression test for
/// https://github.com/korpling/graphANNIS/issues/267
///
/// It test that having an optional node (for negation) in the first position of
/// the query does not affect extracting the correct node name in the "find"
/// query.
#[test]
fn optional_node_first_in_query() {
    let tmp = tempfile::tempdir().unwrap();
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let cs = CorpusStorage::with_auto_cache_size(tmp.path(), true).unwrap();
    // Import both the GraphML and the relANNIS files as corpus
    cs.import_from_fs(
        &cargo_dir.join("tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        None,
        false,
        true,
        |_| {},
    )
    .unwrap();

    // Execute a "find" search for a query that has an optional node
    let q = SearchQuery {
        corpus_names: &["SaltSampleCorpus"],
        query: "node? !> Inf-Struct=\"contrast-focus\"",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs.find(q, 0, None, ResultOrder::Normal).unwrap();
    assert_eq!(4, result.len());
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus1/doc1#IS_span1",
        result[0]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus1/doc2#IS_span1",
        result[1]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus2/doc3#IS_span1",
        result[2]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus2/doc4#IS_span1",
        result[3]
    );

    // Execute a "find" search but reverse the order of the nodes, this should
    // give the same result
    let q = SearchQuery {
        corpus_names: &["SaltSampleCorpus"],
        query: "Inf-Struct=\"contrast-focus\" & node? & #2 !> #1",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };
    let result = cs.find(q, 0, None, ResultOrder::Normal).unwrap();
    assert_eq!(4, result.len());
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus1/doc1#IS_span1",
        result[0]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus1/doc2#IS_span1",
        result[1]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus2/doc3#IS_span1",
        result[2]
    );
    assert_eq!(
        "default_ns::Inf-Struct::rootCorpus/subCorpus2/doc4#IS_span1",
        result[3]
    );
}
