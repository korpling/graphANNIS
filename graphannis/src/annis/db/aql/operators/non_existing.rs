use std::{
    collections::HashSet,
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use graphannis_core::{
    graph::{ANNIS_NS, NODE_NAME},
    types::NodeID,
};
use itertools::Itertools;

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

    fn create_filter_operator<'b>(
        &'b self,
        g: &'b AnnotationGraph,
        negated_op: BinaryOperator<'b>,
        target_left: bool,
        op_estimation: EstimationType,
    ) -> Box<dyn crate::annis::operator::UnaryOperator + 'b> {
        Box::new(NonExistingUnaryOperatorFilter {
            target: self.target.clone(),
            target_left,
            graph: g,
            op_estimation,
            negated_op,
        })
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
    ) -> Result<Box<dyn crate::annis::operator::UnaryOperator + 'b>> {
        let mut target_left = self.target_left;
        let mut orig_op = self.op.create_operator(g)?;

        if target_left {
            // Check if we can avoid a costly filter operation by switching operands
            if let Some(inverted_op) = orig_op.get_inverse_operator(g)? {
                orig_op = inverted_op;
                target_left = false;
            }
        }
        let op_estimation = orig_op.estimation_type()?;

        // When possible, use the index-based implementation, use the filter
        // version as a fallback
        let negated_op: Box<dyn crate::annis::operator::UnaryOperator + 'b> = if !target_left {
            if let BinaryOperator::Index(orig_op) = orig_op {
                Box::new(NonExistingUnaryOperatorIndex {
                    target: self.target.clone(),
                    graph: g,
                    op_estimation,
                    negated_op: orig_op,
                })
            } else {
                self.create_filter_operator(g, orig_op, target_left, op_estimation)
            }
        } else {
            self.create_filter_operator(g, orig_op, target_left, op_estimation)
        };

        Ok(negated_op)
    }
}

struct NonExistingUnaryOperatorIndex<'a> {
    target: NodeSearchSpec,
    negated_op: Box<dyn BinaryOperatorIndex + 'a>,
    graph: &'a AnnotationGraph,
    op_estimation: EstimationType,
}

impl<'a> Display for NonExistingUnaryOperatorIndex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " !{} {}", self.negated_op, self.target)?;
        Ok(())
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperatorIndex<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> Result<bool> {
        // Extract the annotation keys for the matches
        let it = self.negated_op.retrieve_matches(m).map(|m| match m {
            Ok(m) => Ok(m.node),
            Err(e) => {
                let e: Box<dyn Error + Send + Sync> = Box::new(e);
                Err(e)
            }
        });
        let qname = self.target.get_anno_qname();

        let it: Box<
            dyn Iterator<Item = std::result::Result<NodeID, Box<dyn Error + Send + Sync>>>,
        > = Box::from(it);

        let candidates = self.graph.get_node_annos().get_keys_for_iterator(
            qname.0.as_deref(),
            qname.1.as_deref(),
            Box::new(it),
        )?;

        let value_filter = self
            .target
            .get_value_filter(self.graph, None)
            .unwrap_or_default();

        // Only return true if no match was found which matches the operator and
        // the node value filter
        for c in candidates {
            for f in value_filter.iter() {
                if f(&c, self.graph.get_node_annos())? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn estimation_type(&self) -> EstimationType {
        match self.op_estimation {
            EstimationType::Selectivity(s) => EstimationType::Selectivity(1.0 - s),
            EstimationType::Min => EstimationType::Min,
        }
    }
}

struct NonExistingUnaryOperatorFilter<'a> {
    target: NodeSearchSpec,
    target_left: bool,
    negated_op: BinaryOperator<'a>,
    graph: &'a AnnotationGraph,
    op_estimation: EstimationType,
}

impl<'a> Display for NonExistingUnaryOperatorFilter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.target_left {
            write!(f, " !({} {} x)", self.target, self.negated_op)?;
        } else {
            write!(f, " !(x {} {})", self.negated_op, self.target,)?;
        }
        Ok(())
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperatorFilter<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> Result<bool> {
        // The candidate match is on one side and we need to get an iterator of all possible matches for the other side
        if let Ok(node_search) = NodeSearch::from_spec(self.target.clone(), 0, self.graph, None) {
            // Include if no nodes matches the conditions
            if self.target_left {
                for lhs in node_search {
                    let lhs = lhs?;
                    if self.negated_op.filter_match(&lhs[0], m)? {
                        return Ok(false);
                    }
                }
            } else {
                for rhs in node_search {
                    let rhs = rhs?;
                    if self.negated_op.filter_match(m, &rhs[0])? {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn estimation_type(&self) -> EstimationType {
        match self.op_estimation {
            EstimationType::Selectivity(s) => EstimationType::Selectivity(1.0 - s),
            EstimationType::Min => EstimationType::Min,
        }
    }
}
