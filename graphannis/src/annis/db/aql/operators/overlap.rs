use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::operator::{BinaryIndexOperator, EstimationType};
use crate::AnnotationGraph;
use crate::{
    annis::operator::{BinaryOperator, BinaryOperatorSpec},
    graph::{GraphStorage, Match},
    model::{AnnotationComponent, AnnotationComponentType},
};
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

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<Box<dyn BinaryOperator + 'a>> {
        let optional_op = Overlap::new(db, self.reflexive);
        if let Some(op) = optional_op {
            Some(Box::new(op))
        } else {
            None
        }
    }
}

impl<'a> Overlap<'a> {
    pub fn new(graph: &'a AnnotationGraph, reflexive: bool) -> Option<Overlap<'a>> {
        let gs_order = graph.get_graphstorage(&COMPONENT_ORDER)?;
        let tok_helper = TokenHelper::new(graph)?;

        Some(Overlap {
            gs_order,
            tok_helper,
            reflexive,
        })
    }
}

impl<'a> std::fmt::Display for Overlap<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.reflexive {
            write!(f, "_o_reflexive_")
        } else {
            write!(f, "_o_")
        }
    }
}

impl<'a> BinaryOperator for Overlap<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if self.reflexive && lhs == rhs {
            return true;
        }

        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            self.tok_helper.left_token_for(lhs.node),
            self.tok_helper.right_token_for(lhs.node),
            self.tok_helper.left_token_for(rhs.node),
            self.tok_helper.right_token_for(rhs.node),
        ) {
            // TODO: why not isConnected()? (instead of distance)
            // path between LHS left-most token and RHS right-most token exists in ORDERING component
            if self.gs_order.distance(start_lhs, end_rhs).is_some()
                // path between LHS left-most token and RHS right-most token exists in ORDERING component
                && self.gs_order.distance(start_rhs, end_lhs).is_some()
            {
                return true;
            }
        }
        false
    }

    fn is_reflexive(&self) -> bool {
        self.reflexive
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Option<Box<dyn BinaryOperator + 'b>> {
        Some(Box::new(Overlap {
            gs_order: self.gs_order.clone(),
            tok_helper: TokenHelper::new(graph)?,
            reflexive: self.reflexive,
        }))
    }

    fn estimation_type(&self) -> EstimationType {
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
                return EstimationType::SELECTIVITY(1.0 / num_of_token);
            } else {
                return EstimationType::SELECTIVITY(sum_included as f64 / (sum_cov_nodes as f64));
            }
        }

        EstimationType::SELECTIVITY(0.1)
    }
}

impl<'a> BinaryIndexOperator for Overlap<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        // use set to filter out duplicates
        let mut result = FxHashSet::default();

        if self.reflexive {
            // add LHS  itself
            result.insert(lhs.node);
        }

        let lhs_is_token = self.tok_helper.is_token(lhs.node);
        let coverage_gs = self.tok_helper.get_gs_coverage();
        if lhs_is_token && coverage_gs.is_empty() {
            // There are only token in this corpus and an thus the only covered node is the LHS itself
            result.insert(lhs.node);
        } else {
            // Find covered nodes in all Coverage graph storages
            for gs_cov in coverage_gs.iter() {
                let covered: Box<dyn Iterator<Item = NodeID>> = if lhs_is_token {
                    Box::new(std::iter::once(lhs.node))
                } else {
                    // all covered token
                    Box::new(
                        gs_cov
                            .find_connected(lhs.node, 1, std::ops::Bound::Included(1))
                            .fuse(),
                    )
                };

                for t in covered {
                    // get all nodes that are covering the token (in all coverage components)
                    for gs_cov in self.tok_helper.get_gs_coverage().iter() {
                        for n in gs_cov.get_ingoing_edges(t) {
                            result.insert(n);
                        }
                    }
                    // also add the token itself
                    result.insert(t);
                }
            }
        }

        Box::new(result.into_iter().map(|n| Match {
            node: n,
            anno_key: DEFAULT_ANNO_KEY.clone(),
        }))
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperator {
        self
    }
}
