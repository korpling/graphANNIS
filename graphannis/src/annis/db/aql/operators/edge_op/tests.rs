use graphannis_core::graph::{
    ANNIS_NS,
    update::{GraphUpdate, UpdateEvent},
};
use std::assert_matches;

use crate::{
    AnnotationGraph,
    annis::{
        db::{
            aql::{
                ast::RangeSpec,
                operators::{PartOfSubCorpusSpec, PointingSpec},
            },
            example_generator,
            exec::CostEstimate,
        },
        operator::{BinaryOperatorBase, BinaryOperatorSpec, EstimationType},
    },
};

/// Tests that if you invert a @* operator, the cost estimate stays the same.
#[test]
fn inverted_partof_has_same_estimate() {
    // Create a simple annotation graph a chain of PartOf edges, so that the
    // fan-out and inverse fan-out of the PartOf component are both 1.
    // It has the following nodes, connected by a PartOf edge each
    // - root
    // - root/c
    // - root/c/c
    // - root/c/c/c
    // - ...
    // - root/c/c/c/c/c/c/c/c/c/c
    let mut update = GraphUpdate::new();
    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    for i in 1..10 {
        let source_path = format!("root{}", "/c".repeat(i));
        let target_path = format!("root{}", "/c".repeat(i - 1));
        update
            .add_event(UpdateEvent::AddNode {
                node_name: source_path.to_string(),
                node_type: "corpus".to_string(),
            })
            .unwrap();

        update
            .add_event(UpdateEvent::AddEdge {
                source_node: source_path.to_string(),
                target_node: target_path.to_string(),
                layer: ANNIS_NS.to_string(),
                component_type: "PartOf".to_string(),
                component_name: "".to_string(),
            })
            .unwrap();
    }

    let mut g = AnnotationGraph::with_default_graphstorages(false).unwrap();
    g.apply_update(&mut update, |_| {}).unwrap();

    // Define an operator and a realistic cost estimate for LHS and RHS
    let spec = PartOfSubCorpusSpec {
        dist: RangeSpec::Unbound,
    };
    let cost_estimate_lhs = CostEstimate {
        output: 1,
        intermediate_sum: 0,
        processed_in_step: 0,
    };
    let cost_estimate_rhs = CostEstimate {
        output: 1,
        intermediate_sum: 0,
        processed_in_step: 0,
    };

    let operator = spec
        .create_operator(&g, Some((&cost_estimate_lhs, &cost_estimate_rhs)))
        .unwrap();

    let orig_estimate = operator.estimation_type().unwrap();

    let inverted_operator = operator.get_inverse_operator(&g).unwrap();
    assert_eq!(true, inverted_operator.is_some());
    let inverted_operator = inverted_operator.unwrap();

    let inverted_estimate = inverted_operator.estimation_type().unwrap();

    assert_eq!(orig_estimate, inverted_estimate);
}

/// Test the execution plan of a graph component was cycles
#[test]
fn cycle_component_estimation() {
    let mut update = GraphUpdate::new();

    example_generator::create_corpus_structure_simple(&mut update);
    example_generator::create_tokens(&mut update, Some("root/doc1"), Some("root/doc1"));

    for t in 0..10 {
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: format!("root/doc1#tok{t}"),
                target_node: format!("root/doc1#tok{}", t + 1),
                layer: "default_ns".to_string(),
                component_type: "Pointing".to_string(),
                component_name: "dep".to_string(),
            })
            .unwrap();
    }
    // Add a dependency edge from the last to the first token, completing the cycle
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: format!("root/doc1#tok10"),
            target_node: format!("root/doc1#tok0"),
            layer: "default_ns".to_string(),
            component_type: "Pointing".to_string(),
            component_name: "dep".to_string(),
        })
        .unwrap();

    let mut g = AnnotationGraph::with_default_graphstorages(false).unwrap();
    g.apply_update(&mut update, |_| {}).unwrap();

    // Define an operator that operates on the generated dep component and a realistic cost estimate for LHS and RHS
    let unbound_spec = PointingSpec {
        name: "dep".to_string(),
        edge_anno: None,
        dist: RangeSpec::Unbound,
    };
    let direct_spec1 = PointingSpec {
        name: "dep".to_string(),
        edge_anno: None,
        dist: RangeSpec::Bound {
            min_dist: 1,
            max_dist: 1,
        },
    };
    let direct_spec2 = PointingSpec {
        name: "dep".to_string(),
        edge_anno: None,
        dist: RangeSpec::Bound {
            min_dist: 2,
            max_dist: 2,
        },
    };
    let cost_estimate_lhs = CostEstimate {
        output: 10,
        intermediate_sum: 0,
        processed_in_step: 0,
    };
    let cost_estimate_rhs = CostEstimate {
        output: 5,
        intermediate_sum: 0,
        processed_in_step: 0,
    };

    let unbound_op = unbound_spec
        .create_operator(&g, Some((&cost_estimate_lhs, &cost_estimate_rhs)))
        .unwrap();
    assert_eq!(
        unbound_op.estimation_type().unwrap(),
        EstimationType::Selectivity(1.0),
    );

    let direct_op1 = direct_spec1
        .create_operator(&g, Some((&cost_estimate_lhs, &cost_estimate_rhs)))
        .unwrap();
    assert_matches!(
        direct_op1.estimation_type().unwrap(),
        EstimationType::Selectivity(v) if v > 0.0 && v < 1.0
    );

    let direct_op2 = direct_spec2
        .create_operator(&g, Some((&cost_estimate_lhs, &cost_estimate_rhs)))
        .unwrap();
    assert_eq!(
        direct_op1.estimation_type().unwrap(),
        direct_op2.estimation_type().unwrap()
    );
}
