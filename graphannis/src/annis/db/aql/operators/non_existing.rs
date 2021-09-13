use std::fmt::{Debug, Display};

use graphannis_core::{annostorage::AnnotationStorage, types::NodeID};

use crate::{
    annis::{
        db::exec::{nodesearch::NodeSearchSpec, MatchValueFilterFunc},
        operator::{
            BinaryOperator, BinaryOperatorIndex, BinaryOperatorSpec, UnaryOperator,
            UnaryOperatorSpec,
        },
        types::LineColumnRange,
    },
    AnnotationGraph,
};

pub struct NonExistingUnaryOperatorSpec {
    pub node_search: NodeSearchSpec,
    pub node_location: Option<LineColumnRange>,
    pub negated_op: Box<dyn BinaryOperatorSpec>,
}

impl Debug for NonExistingUnaryOperatorSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonExistingUnaryOperatorSpec")
            .field("negated_op", &self.negated_op)
            .finish()
    }
}

impl UnaryOperatorSpec for NonExistingUnaryOperatorSpec {
    fn necessary_components(
        &self,
        g: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        self.negated_op.necessary_components(g)
    }

    fn create_operator<'b>(
        &'b self,
        g: &'b AnnotationGraph,
    ) -> Option<Box<dyn crate::annis::operator::UnaryOperator + 'b>> {
        let value_filter = self
            .node_search
            .get_value_filter(g, self.node_location.clone())
            .ok()?;
        let node_search_qname = self.node_search.get_anno_qname();
        match self.negated_op.create_operator(g)? {
            BinaryOperator::Base(_) => None,
            BinaryOperator::Index(negated_op) => Some(Box::new(NonExistingUnaryOperator {
                negated_op,
                node_annos: g.get_node_annos(),
                node_search_qname,
                value_filter,
            })),
        }
    }
}

struct NonExistingUnaryOperator<'a> {
    negated_op: Box<dyn BinaryOperatorIndex + 'a>,
    node_annos: &'a dyn AnnotationStorage<NodeID>,
    node_search_qname: (Option<String>, Option<String>),
    value_filter: Vec<MatchValueFilterFunc>,
}

impl<'a> Display for NonExistingUnaryOperator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!",)?;
        self.negated_op.fmt(f)?;
        Ok(())
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperator<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        // Extract the annotation keys for the matches
        let it = self.negated_op.retrieve_matches(&m).map(|m| m.node);
        let candidates = self.node_annos.get_keys_for_iterator(
            self.node_search_qname.0.as_deref(),
            self.node_search_qname.1.as_deref(),
            Box::new(it),
        );

        // Only return true of no match was found which matches the operator and node value filter
        !candidates
            .into_iter()
            .any(|m| self.value_filter.iter().all(|f| f(&m, self.node_annos)))
    }
}
