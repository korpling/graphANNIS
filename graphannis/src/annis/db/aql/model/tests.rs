use std::{fs::File, path::PathBuf};

use crate::{
    annis::db::{aql::model::CorpusSize, example_generator},
    model::AnnotationComponent,
    AnnotationGraph,
};
use assert_matches::assert_matches;
use graphannis_core::graph::{
    serialization::graphml,
    storage::GraphStorage,
    update::{GraphUpdate, UpdateEvent},
    NODE_NAME_KEY,
};
use insta::assert_snapshot;
use itertools::Itertools;

use super::AnnotationComponentType::{self, Coverage};

#[test]
fn global_stats_token_count() {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_file = File::open(&cargo_dir.join("tests/SegmentationWithGaps.graphml")).unwrap();
    let (graph, _config_str): (AnnotationGraph, _) =
        graphannis_core::graph::serialization::graphml::import(input_file, false, |_status| {})
            .unwrap();

    assert_eq!(true, graph.global_statistics.is_some());
    let global_stats = graph.global_statistics.unwrap();
    assert_eq!(true, global_stats.all_token_in_order_component);
    assert_matches!(global_stats.corpus_size, 
        CorpusSize::Token { base_token_count, segmentation_count }
        if base_token_count == 16
            && segmentation_count.len() == 2
            && *segmentation_count.get("diplomatic").unwrap() == 11 
            && *segmentation_count.get("norm").unwrap() == 13);
}

#[test]
fn inherited_cov_edges_simple_tokenization() {
    // Ad a simple dominance node structure above the example sentence.
    let mut u = GraphUpdate::new();
    example_generator::create_corpus_structure_simple(&mut u);
    example_generator::create_tokens(&mut u, Some("root/doc1"), Some("root/doc1"));
    example_generator::make_span(
        &mut u,
        "root/doc1#span1",
        &["root/doc1#tok1", "root/doc1#tok2", "root/doc1#tok3"],
        true,
    );
    example_generator::make_span(
        &mut u,
        "root/doc1#span2",
        &["root/doc1#tok4", "root/doc1#tok5"],
        true,
    );
    u.add_event(UpdateEvent::AddNode {
        node_name: "root/doc1#struct1".to_string(),
        node_type: "node".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddNodeLabel {
        node_name: "root/doc1#struct1".to_string(),
        anno_ns: "test".to_string(),
        anno_name: "cat".to_string(),
        anno_value: "P".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddEdge {
        source_node: "root/doc1#struct1".to_string(),
        target_node: "root/doc1#span1".to_string(),
        layer: "test".to_string(),
        component_type: "Dominance".to_string(),
        component_name: "edge".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddEdge {
        source_node: "root/doc1#struct1".to_string(),
        target_node: "root/doc1#span2".to_string(),
        layer: "test".to_string(),
        component_type: "Dominance".to_string(),
        component_name: "edge".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddNode {
        node_name: "root/doc1#struct2".to_string(),
        node_type: "node".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddNodeLabel {
        node_name: "root/doc1#struct2".to_string(),
        anno_ns: "test".to_string(),
        anno_name: "cat".to_string(),
        anno_value: "ROOT".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddEdge {
        source_node: "root/doc1#struct2".to_string(),
        target_node: "root/doc1#struct1".to_string(),
        layer: "test".to_string(),
        component_type: "Dominance".to_string(),
        component_name: "edge".to_string(),
    })
    .unwrap();

    let mut g = AnnotationGraph::with_default_graphstorages(false).unwrap();
    g.apply_update(&mut u, |_| {}).unwrap();

    // Check that the inherited coverage edges have been created
    let gs = g
        .get_graphstorage_as_ref(&AnnotationComponent::new(
            Coverage,
            "annis".into(),
            "inherited-coverage".into(),
        ))
        .unwrap();
    let sources: Vec<_> = gs
        .source_nodes()
        .map(|n| {
            g.get_node_annos()
                .get_value_for_item(&n.unwrap(), &NODE_NAME_KEY)
                .unwrap()
                .unwrap()
                .to_string()
        })
        .sorted()
        .collect();
    assert_eq!(sources, vec!["root/doc1#struct1", "root/doc1#struct2"]);

    // Also check that the edges target the right token
    assert_out_edges(
        &g,
        gs,
        "root/doc1#struct1",
        &[
            "root/doc1#tok1",
            "root/doc1#tok2",
            "root/doc1#tok3",
            "root/doc1#tok4",
            "root/doc1#tok5",
        ],
    );
    assert_out_edges(
        &g,
        gs,
        "root/doc1#struct2",
        &[
            "root/doc1#tok1",
            "root/doc1#tok2",
            "root/doc1#tok3",
            "root/doc1#tok4",
            "root/doc1#tok5",
        ],
    );
}

#[test]
fn inherited_cov_edges_multiple_segmentation() {
    let mut u = GraphUpdate::new();
    example_generator::create_corpus_structure_simple(&mut u);
    example_generator::create_multiple_segmentations(&mut u, "root/doc1");
    // Add a simple dominance node structure above the "a" segmentation
    example_generator::make_span(
        &mut u,
        "root/doc1#span1",
        &["root/doc1#a1", "root/doc1#a2", "root/doc1#a3"],
        true,
    );
    example_generator::make_span(&mut u, "root/doc1#span2", &["root/doc1#a4"], true);
    u.add_event(UpdateEvent::AddNode {
        node_name: "root/doc1#struct1".to_string(),
        node_type: "node".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddNodeLabel {
        node_name: "root/doc1#struct1".to_string(),
        anno_ns: "test".to_string(),
        anno_name: "cat".to_string(),
        anno_value: "ROOT".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddEdge {
        source_node: "root/doc1#struct1".to_string(),
        target_node: "root/doc1#span1".to_string(),
        layer: "test".to_string(),
        component_type: "Dominance".to_string(),
        component_name: "edge".to_string(),
    })
    .unwrap();
    u.add_event(UpdateEvent::AddEdge {
        source_node: "root/doc1#struct1".to_string(),
        target_node: "root/doc1#span2".to_string(),
        layer: "test".to_string(),
        component_type: "Dominance".to_string(),
        component_name: "edge".to_string(),
    })
    .unwrap();

    let mut g = AnnotationGraph::with_default_graphstorages(false).unwrap();
    g.apply_update(&mut u, |_| {}).unwrap();

    // TODO Check that the inherited coverage edges have been created
    let gs = g
        .get_graphstorage_as_ref(&AnnotationComponent::new(
            Coverage,
            "annis".into(),
            "inherited-coverage".into(),
        ))
        .unwrap();

    let sources: Vec<_> = gs
        .source_nodes()
        .map(|n| {
            g.get_node_annos()
                .get_value_for_item(&n.unwrap(), &NODE_NAME_KEY)
                .unwrap()
                .unwrap()
                .to_string()
        })
        .sorted()
        .collect();
    assert_eq!(
        sources,
        vec!["root/doc1#span1", "root/doc1#span2", "root/doc1#struct1"]
    );

    // Also check that the edges target the right timeline items (and not the segmentation nodes)
    assert_out_edges(
        &g,
        gs,
        "root/doc1#span1",
        &[
            "root/doc1#tli1",
            "root/doc1#tli2",
            "root/doc1#tli3",
            "root/doc1#tli4",
        ],
    );
    assert_out_edges(&g, gs, "root/doc1#span2", &["root/doc1#tli5"]);
    assert_out_edges(
        &g,
        gs,
        "root/doc1#struct1",
        &[
            "root/doc1#tli1",
            "root/doc1#tli2",
            "root/doc1#tli3",
            "root/doc1#tli4",
            "root/doc1#tli5",
        ],
    );
}

fn assert_out_edges(
    graph: &AnnotationGraph,
    gs: &dyn GraphStorage,
    source: &str,
    expected: &[&str],
) {
    let out: Vec<_> = gs
        .get_outgoing_edges(
            graph
                .get_node_annos()
                .get_node_id_from_name(source)
                .unwrap()
                .unwrap(),
        )
        .map(|t| {
            graph
                .get_node_annos()
                .get_value_for_item(&t.unwrap(), &NODE_NAME_KEY)
                .unwrap()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(out, expected);
}
#[test]
fn add_token_to_single_sentence() {
    let content = &include_bytes!("../../../../../tests/single_sentence.graphml")[..];
    let (mut graph, _config) =
        graphml::import::<AnnotationComponentType, _, _>(content, false, |_| {}).unwrap();
    // Create updates that add a new token
    let mut updates = GraphUpdate::new();
    updates
        .add_event(UpdateEvent::AddNode {
            node_name: "single_sentence/zossen#newToken".into(),
            node_type: "node".into(),
        })
        .unwrap();
    updates
        .add_event(UpdateEvent::AddNodeLabel {
            node_name: "single_sentence/zossen#newToken".into(),
            anno_ns: "annis".into(),
            anno_name: "tok".into(),
            anno_value: "".into(),
        })
        .unwrap();
    updates
        .add_event(UpdateEvent::AddEdge {
            source_node: "single_sentence/zossen#newToken".into(),
            target_node: "single_sentence/zossen#text".into(),
            layer: "annis".into(),
            component_type: "PartOf".into(),
            component_name: "".into(),
        })
        .unwrap();
    updates
        .add_event(UpdateEvent::DeleteEdge {
            source_node: "single_sentence/zossen#t4".into(),
            target_node: "single_sentence/zossen#t5".into(),
            layer: "annis".into(),
            component_type: "Ordering".into(),
            component_name: "".into(),
        })
        .unwrap();
    updates
        .add_event(UpdateEvent::AddEdge {
            source_node: "single_sentence/zossen#t4".into(),
            target_node: "single_sentence/zossen#newToken".into(),
            layer: "annis".into(),
            component_type: "Ordering".into(),
            component_name: "".into(),
        })
        .unwrap();
    updates
        .add_event(UpdateEvent::AddEdge {
            source_node: "single_sentence/zossen#newToken".into(),
            target_node: "single_sentence/zossen#t5".into(),
            layer: "annis".into(),
            component_type: "Ordering".into(),
            component_name: "".into(),
        })
        .unwrap();
    graph.apply_update(&mut updates, |_| {}).unwrap();

    let mut output = Vec::<u8>::default();
    graphannis_core::graph::serialization::graphml::export_stable_order(
        &graph,
        None,
        &mut output,
        |_| {},
    )
    .unwrap();
    assert_snapshot!(String::from_utf8_lossy(&output));
}
