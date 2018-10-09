use annis::db::GraphDB;
use annis::db::graphstorage::GraphStorage;
use annis::operator::EstimationType;
use annis::operator::{Operator, OperatorSpec};
use annis::types::{AnnoKeyID, Component, ComponentType, Match};
use annis::db::token_helper;
use annis::db::token_helper::TokenHelper;

use std;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InclusionSpec;

pub struct Inclusion {
    gs_order: Arc<GraphStorage>,
    gs_left: Arc<GraphStorage>,
    gs_right: Arc<GraphStorage>,
    gs_cov: Arc<GraphStorage>,

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
    static ref COMPONENT_LEFT: Component = {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
    static ref COMPONENT_RIGHT: Component = {
        Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
    static ref COMPONENT_COV: Component = {
        let c = Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        };
        c
    };
}

impl OperatorSpec for InclusionSpec {
    fn necessary_components(&self, _db: &GraphDB) -> Vec<Component> {
        let mut v: Vec<Component> = vec![
            COMPONENT_ORDER.clone(),
            COMPONENT_LEFT.clone(),
            COMPONENT_RIGHT.clone(),
            COMPONENT_COV.clone(),
        ];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        let optional_op = Inclusion::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl Inclusion {
    pub fn new(db: &GraphDB) -> Option<Inclusion> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let gs_right = db.get_graphstorage(&COMPONENT_RIGHT)?;
        let gs_cov = db.get_graphstorage(&COMPONENT_COV)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(Inclusion {
            gs_order,
            gs_left,
            gs_right,
            gs_cov,
            tok_helper,
        })
    }
}

impl std::fmt::Display for Inclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_i_")
    }
}

impl Operator for Inclusion {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        if let (Some(start_lhs), Some(end_lhs)) = self.tok_helper.left_right_token_for(&lhs.node) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(&start_lhs, &end_lhs) {
                // find each token which is between the left and right border
                let result: VecDeque<Match> = self
                    .gs_order
                    .find_connected(&start_lhs, 0, l)
                    .flat_map(move |t| {
                        let it_aligned = self.gs_left.get_outgoing_edges(&t).into_iter().filter(
                            move |n| {
                                // right-aligned token of candidate
                                let mut end_n = self.gs_right.get_outgoing_edges(&n);
                                if let Some(end_n) = end_n.next() {
                                    // path between right-most tokens exists in ORDERING component
                                    // and has maximum length l
                                    return self.gs_order.is_connected(&end_n, &end_lhs, 0, l);
                                }
                                return false;
                            },
                        );
                        // return the token itself and all aligned nodes
                        return std::iter::once(t).chain(it_aligned);
                    }).map(|n| Match {
                        node: n,
                        anno_key: AnnoKeyID::default(),
                    }).collect();
                return Box::new(result.into_iter());
            }
        }

        return Box::new(std::iter::empty());
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let left_right_lhs = self.tok_helper.left_right_token_for(&lhs.node);
        let left_right_rhs = self.tok_helper.left_right_token_for(&rhs.node);
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            left_right_lhs.0,
            left_right_lhs.1,
            left_right_rhs.0,
            left_right_rhs.1,
        ) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(&start_lhs, &end_lhs) {
                // path between left-most tokens exists in ORDERING component and has maximum length l
                if self.gs_order.is_connected(&start_lhs, &start_rhs, 0, l)
                // path between right-most tokens exists in ORDERING component and has maximum length l
                && self.gs_order.is_connected(&end_rhs, &end_lhs, 0, l)
                {
                    return true;
                }
            }
        }

        return false;
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn estimation_type(&self) -> EstimationType {
        if let (Some(stats_cov), Some(stats_order), Some(stats_left)) = (
            self.gs_cov.get_statistics(),
            self.gs_order.get_statistics(),
            self.gs_left.get_statistics(),
        ) {
            let num_of_token = stats_order.nodes as f64;
            if stats_cov.nodes == 0 {
                // only token in this corpus
                return EstimationType::SELECTIVITY(1.0 / num_of_token);
            } else {
                let covered_token_per_node: f64 = stats_cov.fan_out_99_percentile as f64;
                let aligned_non_token: f64 =
                    covered_token_per_node * (stats_left.fan_out_99_percentile as f64);

                let sum_included = covered_token_per_node + aligned_non_token;
                return EstimationType::SELECTIVITY(sum_included / (stats_cov.nodes as f64));
            }
        }

        return EstimationType::SELECTIVITY(0.1);
    }
}
