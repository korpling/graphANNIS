use crate::annis::db::exec::CostEstimate;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    errors::Result,
    graph::{GraphStorage, Match},
    model::{AnnotationComponent, AnnotationComponentType},
};
use crate::{try_as_boxed_iter, AnnotationGraph};
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY},
    types::NodeID,
};
use rustc_hash::FxHashSet;

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct OverlapSpec {
    /// If true, the overlap operator can match the same node-annotation combination as LHS and RHS
    pub reflexive: bool,
}

#[derive(Clone)]
pub struct Overlap<'a> {
    gs_order: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
    reflexive: bool,
}

lazy_static! {
    static ref COMPONENT_ORDER: AnnotationComponent = {
        AnnotationComponent::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

impl BinaryOperatorSpec for OverlapSpec {
    fn necessary_components(&self, db: &AnnotationGraph) -> HashSet<AnnotationComponent> {
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
        let optional_op = Overlap::new(db, self.reflexive);
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

impl<'a> Overlap<'a> {
    pub fn new(graph: &'a AnnotationGraph, reflexive: bool) -> Result<Overlap<'a>> {
        let gs_order = graph.get_graphstorage(&COMPONENT_ORDER).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "Ordering component missing (needed for _o_ operator)".to_string(),
            )
        })?;
        let tok_helper = TokenHelper::new(graph)?;

        Ok(Overlap {
            gs_order,
            tok_helper,
            reflexive,
        })
    }
}

impl std::fmt::Display for Overlap<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.reflexive {
            write!(f, "_o_reflexive_")
        } else {
            write!(f, "_o_")
        }
    }
}

impl BinaryOperatorBase for Overlap<'_> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        if self.reflexive && lhs == rhs {
            return Ok(true);
        }

        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            self.tok_helper.left_token_for(lhs.node)?,
            self.tok_helper.right_token_for(lhs.node)?,
            self.tok_helper.left_token_for(rhs.node)?,
            self.tok_helper.right_token_for(rhs.node)?,
        ) {
            // TODO: why not isConnected()? (instead of distance)
            // path between LHS left-most token and RHS right-most token exists in ORDERING component
            if self.gs_order.distance(start_lhs, end_rhs)?.is_some()
                // path between LHS left-most token and RHS right-most token exists in ORDERING component
                && self.gs_order.distance(start_rhs, end_lhs)?.is_some()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_reflexive(&self) -> bool {
        self.reflexive
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        let inverse = BinaryOperator::Index(Box::new(Overlap {
            gs_order: self.gs_order.clone(),
            tok_helper: TokenHelper::new(graph)?,
            reflexive: self.reflexive,
        }));
        Ok(Some(inverse))
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let mut sum_included = 0;
            let mut sum_cov_nodes = 0;

            let num_of_token = stats_order.nodes as f64;

            for gs_cov in self.tok_helper.get_gs_coverage().iter() {
                if let Some(stats_cov) = gs_cov.get_statistics() {
                    sum_cov_nodes += stats_cov.nodes;
                    let covered_token_per_node = stats_cov.fan_out_99_percentile;
                    // for each covered token get the number of inverse covered non-token nodes
                    let aligned_non_token =
                        covered_token_per_node * (stats_cov.inverse_fan_out_99_percentile);

                    sum_included += covered_token_per_node + aligned_non_token;
                }
            }
            if self.reflexive {
                sum_included += 1;
            }

            if sum_cov_nodes == 0 {
                // only token in this corpus
                return Ok(EstimationType::Selectivity(1.0 / num_of_token));
            } else {
                return Ok(EstimationType::Selectivity(
                    sum_included as f64 / (sum_cov_nodes as f64),
                ));
            }
        }

        Ok(EstimationType::Selectivity(0.1))
    }
}

impl BinaryOperatorIndex for Overlap<'_> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>>> {
        // use set to filter out duplicates
        let mut result = FxHashSet::default();

        if self.reflexive {
            // add LHS  itself
            result.insert(lhs.node);
        }

        let lhs_is_token = try_as_boxed_iter!(self.tok_helper.is_token(lhs.node));
        let coverage_gs = self.tok_helper.get_gs_coverage();
        if lhs_is_token && coverage_gs.is_empty() {
            // There are only token in this corpus and an thus the only covered node is the LHS itself
            result.insert(lhs.node);
        } else {
            // Find covered nodes in all Coverage graph storages
            for gs_cov in coverage_gs.iter() {
                let covered: Box<dyn Iterator<Item = Result<NodeID>>> = if lhs_is_token {
                    Box::new(std::iter::once(Ok(lhs.node)))
                } else {
                    // all covered token
                    Box::new(
                        gs_cov
                            .find_connected(lhs.node, 1, std::ops::Bound::Included(1))
                            .map(|m| m.map_err(GraphAnnisError::from))
                            .fuse(),
                    )
                };

                for t in covered {
                    let t = try_as_boxed_iter!(t);
                    // get all nodes that are covering the token (in all coverage components)
                    for gs_cov in self.tok_helper.get_gs_coverage().iter() {
                        for n in gs_cov.get_ingoing_edges(t) {
                            let n = try_as_boxed_iter!(n);
                            result.insert(n);
                        }
                    }
                    // also add the token itself
                    result.insert(t);
                }
            }
        }

        Box::new(result.into_iter().map(|n| {
            Ok(Match {
                node: n,
                anno_key: DEFAULT_ANNO_KEY.clone(),
            })
        }))
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
