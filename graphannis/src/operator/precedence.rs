use super::{Operator, OperatorSpec};
use {Annotation, Component, ComponentType, Match};
use graphdb::GraphDB;
use graphstorage::GraphStorage;
use operator::EstimationType;
use util::token_helper;
use util::token_helper::TokenHelper;

use std::sync::Arc;
use std::collections::VecDeque;
use std;

#[derive(Clone, Debug)]
pub struct PrecedenceSpec {
    pub segmentation: Option<String>,
    pub min_dist: usize,
    pub max_dist: usize,
}

pub struct Precedence {
    gs_order: Arc<GraphStorage>,
    gs_left: Arc<GraphStorage>,
    tok_helper: TokenHelper,
    spec: PrecedenceSpec,
}

lazy_static! {
    static ref COMPONENT_LEFT: Component = {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for PrecedenceSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: self.segmentation.clone().unwrap_or(String::from("")),
        };

        let mut v: Vec<Component> = vec![component_order.clone(), COMPONENT_LEFT.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        let optional_op = Precedence::new(db, self.clone());
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl std::fmt::Display for PrecedenceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        
        let range_desc = if self.min_dist == 1 && self.max_dist == usize::max_value() {
            String::from("*")
        } else if self.min_dist == 1 && self.max_dist == 1 {
            String::from("")
        } else {
            format!("{},{}", self.min_dist, self.max_dist)
        };

        if let Some(ref seg) = self.segmentation {
            write!(f, "{} {}", seg, range_desc)
        } else {
            write!(f, "{}", range_desc)
        }
        
    }
}

impl Precedence {
    pub fn new(db: &GraphDB, spec: PrecedenceSpec) -> Option<Precedence> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: spec.segmentation.clone().unwrap_or(String::from("")),
        };

        let gs_order = db.get_graphstorage(&component_order)?;
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(Precedence {
            gs_order: gs_order,
            gs_left: gs_left,
            tok_helper: tok_helper,
            spec,
        })
    }
}

impl std::fmt::Display for Precedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".{}", self.spec)
    }
}

impl Operator for Precedence {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            self.tok_helper.right_token_for(&lhs.node)
        };

        if start.is_none() {
            return Box::new(std::iter::empty::<Match>());
        }

        let start = start.unwrap();

        // materialize a list of all matches
        let result: VecDeque<Match> = self.gs_order
            // get all token in the range
            .find_connected(&start, self.spec.min_dist, self.spec.max_dist).fuse()
            // find all left aligned nodes for this token and add it together with the token itself
            .flat_map(move |t| {
                let it_aligned = self.gs_left.get_outgoing_edges(&t);
                std::iter::once(t).chain(it_aligned)
            })
            // map the result as match
            .map(|n| Match {node: n, anno: Annotation::default()})
            .collect();

        return Box::new(result.into_iter());
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.right_token_for(&lhs.node);
            let end = self.tok_helper.left_token_for(&rhs.node);
            if start.is_none() || end.is_none() {
                return false;
            }
            (start.unwrap(), end.unwrap())
        };

        return self.gs_order.is_connected(
            &start_end.0,
            &start_end.1,
            self.spec.min_dist,
            self.spec.max_dist,
        );
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_possible_dist = std::cmp::min(self.spec.max_dist, stats_order.max_depth);
            let num_of_descendants = max_possible_dist - self.spec.min_dist + 1;

            return EstimationType::SELECTIVITY(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            );
        }

        return EstimationType::SELECTIVITY(0.1);
    }

    fn get_inverse_operator(&self) -> Option<Box<Operator>> {
        let inv_precedence = InversePrecedence {
            gs_order: self.gs_order.clone(),
            gs_left: self.gs_left.clone(),
            tok_helper: self.tok_helper.clone(),
            spec: self.spec.clone(),
        };
        Some(Box::new(inv_precedence))
    }
}

pub struct InversePrecedence {
    gs_order: Arc<GraphStorage>,
    gs_left: Arc<GraphStorage>,
    tok_helper: TokenHelper,
    spec: PrecedenceSpec,
}

impl InversePrecedence {
    pub fn new(db: &GraphDB, spec: PrecedenceSpec) -> Option<InversePrecedence> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: spec.segmentation.clone().unwrap_or(String::from("")),
        };

        let gs_order = db.get_graphstorage(&component_order)?;
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(InversePrecedence {
            gs_order: gs_order,
            gs_left: gs_left,
            tok_helper: tok_helper,
            spec,
        })
    }
}

impl std::fmt::Display for InversePrecedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_inv_precedence_{}", self.spec)
    }
}

impl Operator for InversePrecedence {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            self.tok_helper.left_token_for(&lhs.node)
        };

        if start.is_none() {
            return Box::new(std::iter::empty::<Match>());
        }

        let start = start.unwrap();

        // materialize a list of all matches
        let result: VecDeque<Match> = self.gs_order
            // get all token in the range
            .find_connected_inverse(&start, self.spec.min_dist, self.spec.max_dist).fuse()
            // find all left aligned nodes for this token and add it together with the token itself
            .flat_map(move |t| {
                let it_aligned = self.gs_left.get_outgoing_edges(&t);
                std::iter::once(t).chain(it_aligned)
            })
            // map the result as match
            .map(|n| Match {node: n, anno: Annotation::default()})
            .collect();

        return Box::new(result.into_iter());
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.left_token_for(&lhs.node);
            let end = self.tok_helper.right_token_for(&rhs.node);
            if start.is_none() || end.is_none() {
                return false;
            }
            (start.unwrap(), end.unwrap())
        };

        return self.gs_order.is_connected(
            &start_end.1,
            &start_end.0,
            self.spec.min_dist,
            self.spec.max_dist,
        );
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_possible_dist = std::cmp::min(self.spec.max_dist, stats_order.max_depth);
            let num_of_descendants = max_possible_dist - self.spec.min_dist + 1;

            return EstimationType::SELECTIVITY(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            );
        }

        return EstimationType::SELECTIVITY(0.1);
    }
}
