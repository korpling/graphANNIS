use crate::annis::db::exec::CostEstimate;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::{AnnotationGraph, try_as_boxed_iter};
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    errors::Result,
    graph::{GraphStorage, Match},
    model::AnnotationComponentType,
};
use graphannis_core::types::NodeID;
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY},
    types::Component,
};
use itertools::Itertools;

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct InclusionSpec;

pub struct Inclusion<'a> {
    gs_order: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
}

lazy_static! {
    static ref COMPONENT_ORDER: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

impl BinaryOperatorSpec for InclusionSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut v = HashSet::default();
        v.insert(COMPONENT_ORDER.clone());
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        _cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>> {
        let optional_op = Inclusion::new(db);
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    #[cfg(test)]
    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any> {
        self
    }

    #[cfg(test)]
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

impl<'a> Inclusion<'a> {
    pub fn new(db: &'a AnnotationGraph) -> Result<Inclusion<'a>> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "Ordering component missing (needed for _i_ operator)".to_string(),
            )
        })?;

        let tok_helper = TokenHelper::new(db)?;

        Ok(Inclusion {
            gs_order,
            tok_helper,
        })
    }
}

impl std::fmt::Display for Inclusion<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_i_")
    }
}

impl BinaryOperatorBase for Inclusion<'_> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let left_right_lhs = self.tok_helper.left_right_token_for(lhs.node)?;
        let left_right_rhs = self.tok_helper.left_right_token_for(rhs.node)?;
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            left_right_lhs.0,
            left_right_lhs.1,
            left_right_rhs.0,
            left_right_rhs.1,
        ) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(start_lhs, end_lhs)? {
                // path between left-most tokens exists in ORDERING component and has maximum length l
                if self.gs_order.is_connected(start_lhs, start_rhs, 0, std::ops::Bound::Included(l))?
                // path between right-most tokens exists in ORDERING component and has maximum length l
                && self.gs_order.is_connected(end_rhs, end_lhs, 0, std::ops::Bound::Included(l))?
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let (Some(stats_order), Some(stats_left)) = (
            self.gs_order.get_statistics(),
            self.tok_helper.get_gs_left_token().get_statistics(),
        ) {
            let mut sum_cov_nodes = 0;
            let mut sum_included = 0;

            let num_of_token = stats_order.nodes as f64;
            for gs_cov in self.tok_helper.get_gs_coverage().iter() {
                if let Some(stats_cov) = gs_cov.get_statistics() {
                    sum_cov_nodes += stats_cov.nodes;

                    let covered_token_per_node = stats_cov.fan_out_99_percentile;
                    let aligned_non_token =
                        covered_token_per_node * stats_left.inverse_fan_out_99_percentile;

                    sum_included += covered_token_per_node + aligned_non_token;
                }
            }
            if sum_cov_nodes == 0 {
                // only token in this corpus
                return Ok(EstimationType::Selectivity(1.0 / num_of_token));
            } else {
                return Ok(EstimationType::Selectivity(
                    (sum_included as f64) / (sum_cov_nodes as f64),
                ));
            }
        }

        Ok(EstimationType::Selectivity(0.1))
    }
}

enum NodeType {
    Token(NodeID),
    Other(NodeID),
}

impl BinaryOperatorIndex for Inclusion<'_> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>> + 'b> {
        let left_right_token = try_as_boxed_iter!(self.tok_helper.left_right_token_for(lhs.node));
        if let (Some(start_lhs), Some(end_lhs)) = left_right_token {
            // span length of LHS
            let l = try_as_boxed_iter!(self.gs_order.distance(start_lhs, end_lhs));
            if let Some(l) = l {
                // find each token which is between the left and right border
                let overlapped_token =
                    self.gs_order
                        .find_connected(start_lhs, 0, std::ops::Bound::Included(l));
                // get the nodes that are covering these overlapped tokens
                let candidates = overlapped_token
                    .map_ok(move |t| {
                        let others = self
                            .tok_helper
                            .get_gs_left_token()
                            .get_ingoing_edges(t)
                            .map_ok(NodeType::Other);
                        // return the token itself and all aligned nodes
                        std::iter::once(Ok(NodeType::Token(t))).chain(others)
                    })
                    .flatten_ok()
                    // Unwrap the Result<Result<_>>
                    .map(|c| match c {
                        Ok(c) => match c {
                            Ok(c) => Ok(c),
                            Err(e) => Err(GraphAnnisError::from(e)),
                        },
                        Err(e) => Err(GraphAnnisError::from(e)),
                    });
                // we need to check if the the RHS of these candidates is also included by the original span
                let result = candidates
                    .map(move |n| {
                        let n = n?;
                        match n {
                            NodeType::Token(t) => Ok(Some(t)),
                            NodeType::Other(n) => {
                                // get right-aligned token of candidate
                                let mut end_n =
                                    self.tok_helper.get_gs_right_token().get_outgoing_edges(n);
                                if let Some(end_n) = end_n.next() {
                                    let end_n = end_n?;
                                    if self.gs_order.is_connected(
                                        end_n,
                                        end_lhs,
                                        0,
                                        std::ops::Bound::Included(l),
                                    )? {
                                        // path between right-most tokens exists in ORDERING component
                                        // and has maximum length l
                                        return Ok(Some(n));
                                    }
                                }
                                Ok(None)
                            }
                        }
                    })
                    // Only include the ones where the constraint was met
                    .filter_map_ok(|n| n)
                    .map_ok(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    });
                return Box::new(result);
            }
        }

        Box::new(std::iter::empty())
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
