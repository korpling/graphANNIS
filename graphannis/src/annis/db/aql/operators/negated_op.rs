use std::fmt::Display;

use crate::{
    annis::{
        db::exec::nodesearch::NodeSearchSpec,
        operator::{BinaryOperator, BinaryOperatorSpec},
    },
    AnnotationGraph,
};
use graphannis_core::{
    annostorage::{AnnotationStorage, Match, ValueSearch},
    graph::NODE_TYPE_KEY,
    types::NodeID,
};

#[derive(Debug)]
pub struct NegatedOpSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
    pub negated_op: Box<dyn BinaryOperatorSpec>,
}

impl BinaryOperatorSpec for NegatedOpSpec {
    fn is_binding(&self) -> bool {
        false
    }

    fn necessary_components(
        &self,
        db: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        self.negated_op.necessary_components(db)
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<Box<dyn BinaryOperator + 'a>> {
        if let Some(negated_op) = self.negated_op.create_operator(db) {
            let op = NegatedOp {
                negated_op,
                node_annos: db.get_node_annos(),
            };
            Some(Box::new(op))
        } else {
            None
        }
    }
}

pub struct NegatedOp<'a> {
    negated_op: Box<dyn BinaryOperator + 'a>,
    node_annos: &'a dyn AnnotationStorage<NodeID>,
}

impl<'a> Display for NegatedOp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!",)?;
        self.negated_op.fmt(f)?;
        Ok(())
    }
}

impl<'a> BinaryOperator for NegatedOp<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        // Brute force search for all possible matches and filter using the wrapped operator
        let matched: Vec<_> = self
            .node_annos
            .exact_anno_search(
                Some(&NODE_TYPE_KEY.ns),
                &NODE_TYPE_KEY.name,
                ValueSearch::Some("node"),
            )
            .filter(|rhs| !self.negated_op.filter_match(lhs, rhs))
            .collect();
        Box::new(matched.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        // Invert the filtered logic by the actual operator
        !self.negated_op.filter_match(lhs, rhs)
    }
}
