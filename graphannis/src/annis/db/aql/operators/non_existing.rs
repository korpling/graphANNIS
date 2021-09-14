use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    sync::Arc,
};

use graphannis_core::graph::{ANNIS_NS, NODE_NAME};

use crate::{
    annis::{
        db::{
            aql::conjunction::Conjunction,
            exec::nodesearch::{NodeSearch, NodeSearchSpec},
        },
        errors::Result,
        operator::{
            BinaryOperator, BinaryOperatorBase, BinaryOperatorIndex, BinaryOperatorSpec,
            EstimationType, UnaryOperator, UnaryOperatorSpec,
        },
    },
    AnnotationGraph,
};

#[derive(Debug, Clone)]
pub struct NonExistingUnaryOperatorSpec {
    pub op: Arc<dyn BinaryOperatorSpec>,
    pub target: NodeSearchSpec,
    pub target_left: bool,
}

impl NonExistingUnaryOperatorSpec {
    fn create_subquery_conjunction(&self, node_name: String) -> Result<Conjunction> {
        let mut subquery = Conjunction::new();
        let var_flex = subquery.add_node(
            NodeSearchSpec::ExactValue {
                ns: Some(ANNIS_NS.into()),
                name: NODE_NAME.into(),
                val: Some(node_name),
                is_meta: false,
            },
            None,
        );
        let var_target = subquery.add_node(self.target.clone(), None);
        if self.target_left {
            subquery.add_operator(self.op.clone(), &var_target, &var_flex, true)?;
        } else {
            subquery.add_operator(self.op.clone(), &var_flex, &var_target, true)?;
        }

        Ok(subquery)
    }
}

impl UnaryOperatorSpec for NonExistingUnaryOperatorSpec {
    fn necessary_components(
        &self,
        g: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        if let Ok(subquery) = self.create_subquery_conjunction(String::default()) {
            subquery.necessary_components(g)
        } else {
            HashSet::default()
        }
    }

    fn create_operator<'b>(
        &'b self,
        g: &'b AnnotationGraph,
    ) -> Option<Box<dyn crate::annis::operator::UnaryOperator + 'b>> {
        let mut query_fragment = "<unknown>".into();
        let mut op_estimation = EstimationType::Min;
        if let Some(op) = self.op.create_operator(g) {
            query_fragment = if self.target_left {
                format!("{} {} x", self.target, op)
            } else {
                format!("x {} {}", op, self.target)
            };
            op_estimation = op.estimation_type();
        }

        let orig_op = self.op.create_operator(g)?;
        let mut negated_op: Box<dyn crate::annis::operator::UnaryOperator + 'b> =
            Box::new(NonExistingUnaryOperatorFilter {
                spec: self.clone(),
                graph: g,
                query_fragment: query_fragment.clone(),
                op_estimation: op_estimation.clone(),
                negated_op: orig_op,
            });
        if !self.target_left {
            if let BinaryOperator::Index(orig_op) = self.op.create_operator(g)? {
                negated_op = Box::new(NonExistingUnaryOperatorIndex {
                    spec: self.clone(),
                    graph: g,
                    query_fragment,
                    op_estimation,
                    negated_op: orig_op,
                })
            }
        }
        Some(negated_op)
    }
}

struct NonExistingUnaryOperatorIndex<'a> {
    spec: NonExistingUnaryOperatorSpec,
    negated_op: Box<dyn BinaryOperatorIndex + 'a>,
    graph: &'a AnnotationGraph,
    query_fragment: String,
    op_estimation: EstimationType,
}

impl<'a> Display for NonExistingUnaryOperatorIndex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " !({})", self.query_fragment)
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperatorIndex<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        // Extract the annotation keys for the matches
        let it = self.negated_op.retrieve_matches(&m).map(|m| m.node);
        let qname = self.spec.target.get_anno_qname();
        let candidates = self.graph.get_node_annos().get_keys_for_iterator(
            qname.0.as_deref(),
            qname.1.as_deref(),
            Box::new(it),
        );

        let value_filter = self
            .spec
            .target
            .get_value_filter(self.graph, None)
            .unwrap_or_default();

        // Only return true of no match was found which matches the operator and node value filter
        !candidates.into_iter().any(|m| {
            value_filter
                .iter()
                .all(|f| f(&m, self.graph.get_node_annos()))
        })
    }

    fn estimation_type(&self) -> EstimationType {
        match self.op_estimation {
            EstimationType::Selectivity(s) => EstimationType::Selectivity(1.0 - s),
            EstimationType::Min => EstimationType::Min,
        }
    }
}

struct NonExistingUnaryOperatorFilter<'a> {
    spec: NonExistingUnaryOperatorSpec,
    negated_op: BinaryOperator<'a>,
    graph: &'a AnnotationGraph,
    query_fragment: String,
    op_estimation: EstimationType,
}

impl<'a> Display for NonExistingUnaryOperatorFilter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " !({})", self.query_fragment)
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperatorFilter<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        // The candidate match is on one side and we need to get an iterator of all possible matches for the other side
        if let Ok(mut node_search) =
            NodeSearch::from_spec(self.spec.target.clone(), 0, self.graph, None)
        {
            // Include if no nodes matches the conditions
            let has_any_match = if self.spec.target_left {
                node_search.any(|lhs| self.negated_op.filter_match(&lhs[0], m))
            } else {
                node_search.any(|rhs| self.negated_op.filter_match(m, &rhs[0]))
            };
            !has_any_match
        } else {
            false
        }
    }

    fn estimation_type(&self) -> EstimationType {
        match self.op_estimation {
            EstimationType::Selectivity(s) => EstimationType::Selectivity(1.0 - s),
            EstimationType::Min => EstimationType::Min,
        }
    }
}
