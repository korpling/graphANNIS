use graphannis_core::graph::{
    ANNIS_NS,
    update::{GraphUpdate, UpdateEvent},
};

use crate::{
    AnnotationGraph,
    annis::{
        db::{
            aql::{ast::RangeSpec, operators::PartOfSubCorpusSpec},
            exec::CostEstimate,
        },
        operator::{BinaryOperatorBase, BinaryOperatorSpec},
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
