use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::{Graph, Match};
use crate::annis::operator::EstimationType;
use crate::annis::operator::{BinaryOperator, BinaryOperatorSpec};
use crate::annis::types::{AnnoKeyID, Component, ComponentType, NodeID};
use rustc_hash::FxHashSet;

use std;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct OverlapSpec;

#[derive(Clone)]
pub struct Overlap {
    gs_order: Arc<GraphStorage>,
    gs_cov: Vec<Arc<GraphStorage>>,
    tok_helper: TokenHelper,
}

lazy_static! {
    static ref COMPONENT_ORDER: Component = {
        Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl BinaryOperatorSpec for OverlapSpec {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component> {
        let mut v = HashSet::default();
        v.insert(COMPONENT_ORDER.clone());
        v.extend(db.get_all_components(Some(ComponentType::Coverage), None));
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<BinaryOperator>> {
        let optional_op = Overlap::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl Overlap {
    pub fn new(db: &Graph) -> Option<Overlap> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
        let gs_cov: Vec<Arc<GraphStorage>> = db
            .get_all_components(Some(ComponentType::Coverage), None)
            .into_iter()
            .filter_map(|c| db.get_graphstorage(&c))
            .filter(|gs| {
                if let Some(stats) = gs.get_statistics() {
                    stats.nodes > 0
                } else {
                    true
                }
            })
            .collect();
        let tok_helper = TokenHelper::new(db)?;

        Some(Overlap {
            gs_order,
            gs_cov,
            tok_helper,
        })
    }
}

impl std::fmt::Display for Overlap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_o_")
    }
}

impl BinaryOperator for Overlap {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        // use set to filter out duplicates
        let mut result = FxHashSet::default();

        let lhs_is_token = self.tok_helper.is_token(lhs.node);

        for gs_cov in self.gs_cov.iter() {
            let covered: Box<Iterator<Item = NodeID>> = if lhs_is_token {
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
                // get all nodes that are covering the token
                for n in gs_cov.get_ingoing_edges(t) {
                    result.insert(n);
                }
                // also add the token itself
                result.insert(t);
            }
        }

        Box::new(result.into_iter().map(|n| Match {
            node: n,
            anno_key: AnnoKeyID::default(),
        }))
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            self.tok_helper.left_token_for(lhs.node),
            self.tok_helper.right_token_for(lhs.node),
            self.tok_helper.left_token_for(rhs.node),
            self.tok_helper.right_token_for(rhs.node),
        ) {
            // TODO: why not isConnected()? (instead of distance)
            // path between LHS left-most token and RHS right-most token exists in ORDERING component
            if self.gs_order.distance(&start_lhs, &end_rhs).is_some()
                // path between LHS left-most token and RHS right-most token exists in ORDERING component
                && self.gs_order.distance(&start_rhs, &end_lhs).is_some()
            {
                return true;
            }
        }
        false
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        Some(Box::new(self.clone()))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let mut sum_included = 0;
            let mut sum_cov_nodes = 0;

            let num_of_token = stats_order.nodes as f64;

            for gs_cov in self.gs_cov.iter() {
                if let Some(stats_cov) = gs_cov.get_statistics() {
                    sum_cov_nodes += stats_cov.nodes;
                    let covered_token_per_node = stats_cov.fan_out_99_percentile;
                    // for each covered token get the number of inverse covered non-token nodes
                    let aligned_non_token =
                        covered_token_per_node * (stats_cov.inverse_fan_out_99_percentile);

                    sum_included += covered_token_per_node + aligned_non_token;
                }
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
