use crate::annis::db::aql::operators::RangeSpec;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::{Graph, Match};
use crate::annis::operator::EstimationType;
use crate::annis::operator::{BinaryOperator, BinaryOperatorSpec};
use crate::annis::types::{AnnoKeyID, Component, ComponentType};

use std;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrecedenceSpec {
    pub segmentation: Option<String>,
    pub dist: RangeSpec,
}

pub struct Precedence {
    gs_order: Arc<GraphStorage>,
    gs_left: Arc<GraphStorage>,
    gs_right: Arc<GraphStorage>,
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
    static ref COMPONENT_RIGHT: Component = {
        Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl BinaryOperatorSpec for PrecedenceSpec {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: self
                .segmentation
                .clone()
                .unwrap_or_else(|| String::from("")),
        };

        let mut v = HashSet::default();
        v.insert(component_order.clone());
        v.insert(COMPONENT_LEFT.clone());
        v.insert(COMPONENT_RIGHT.clone());
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<BinaryOperator>> {
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
        if let Some(ref seg) = self.segmentation {
            write!(f, "{} {}", seg, self.dist)
        } else {
            write!(f, "{}", self.dist)
        }
    }
}

impl Precedence {
    pub fn new(db: &Graph, spec: PrecedenceSpec) -> Option<Precedence> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: spec
                .segmentation
                .clone()
                .unwrap_or_else(|| String::from("")),
        };

        let gs_order = db.get_graphstorage(&component_order)?;
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let gs_right = db.get_graphstorage(&COMPONENT_RIGHT)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(Precedence {
            gs_order,
            gs_left,
            gs_right,
            tok_helper,
            spec,
        })
    }
}

impl std::fmt::Display for Precedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".{}", self.spec)
    }
}

impl BinaryOperator for Precedence {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            self.tok_helper.right_token_for(lhs.node)
        };

        if start.is_none() {
            return Box::new(std::iter::empty::<Match>());
        }

        let start = start.unwrap();

        // materialize a list of all matches
        let result: VecDeque<Match> = self
            .gs_order
            // get all token in the range
            .find_connected(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
            .fuse()
            // find all left aligned nodes for this token and add it together with the token itself
            .flat_map(move |t| {
                let it_aligned = self.gs_left.get_ingoing_edges(t);
                std::iter::once(t).chain(it_aligned)
            })
            // map the result as match
            .map(|n| Match {
                node: n,
                anno_key: AnnoKeyID::default(),
            }).collect();

        Box::new(result.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.right_token_for(lhs.node);
            let end = self.tok_helper.left_token_for(rhs.node);
            if start.is_none() || end.is_none() {
                return false;
            }
            (start.unwrap(), end.unwrap())
        };

        self.gs_order.is_connected(
            &start_end.0,
            &start_end.1,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_dist = match self.spec.dist.max_dist() {
                std::ops::Bound::Unbounded => usize::max_value(),
                std::ops::Bound::Included(max_dist) => max_dist,
                std::ops::Bound::Excluded(max_dist) => max_dist - 1,
            };
            let max_possible_dist = std::cmp::min(max_dist, stats_order.max_depth);
            let num_of_descendants = max_possible_dist - self.spec.dist.min_dist() + 1;

            return EstimationType::SELECTIVITY(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            );
        }

        EstimationType::SELECTIVITY(0.1)
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        // Check if order graph storages has the same inverse cost.
        // If not, we don't provide an inverse operator, because the plans would not account for the different costs
        if !self.gs_order.inverse_has_same_cost() {
            return None;
        }

        let inv_precedence = InversePrecedence {
            gs_order: self.gs_order.clone(),
            gs_left: self.gs_left.clone(),
            gs_right: self.gs_right.clone(),
            tok_helper: self.tok_helper.clone(),
            spec: self.spec.clone(),
        };
        Some(Box::new(inv_precedence))
    }
}

pub struct InversePrecedence {
    gs_order: Arc<GraphStorage>,
    gs_left: Arc<GraphStorage>,
    gs_right: Arc<GraphStorage>,
    tok_helper: TokenHelper,
    spec: PrecedenceSpec,
}

impl std::fmt::Display for InversePrecedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".\u{20D6}{}", self.spec)
    }
}

impl BinaryOperator for InversePrecedence {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            self.tok_helper.left_token_for(lhs.node)
        };

        if start.is_none() {
            return Box::new(std::iter::empty::<Match>());
        }

        let start = start.unwrap();

        // materialize a list of all matches
        let result: VecDeque<Match> = self
            .gs_order
            // get all token in the range
            .find_connected_inverse(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
            .fuse()
            // find all right aligned nodes for this token and add it together with the token itself
            .flat_map(move |t| {
                let it_aligned = self.gs_right.get_ingoing_edges(t);
                std::iter::once(t).chain(it_aligned)
            })
            // map the result as match
            .map(|n| Match {
                node: n,
                anno_key: AnnoKeyID::default(),
            }).collect();

        Box::new(result.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.left_token_for(lhs.node);
            let end = self.tok_helper.right_token_for(rhs.node);
            if start.is_none() || end.is_none() {
                return false;
            }
            (start.unwrap(), end.unwrap())
        };

        self.gs_order.is_connected(
            &start_end.1,
            &start_end.0,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        let prec = Precedence {
            gs_order: self.gs_order.clone(),
            gs_left: self.gs_left.clone(),
            gs_right: self.gs_right.clone(),
            tok_helper: self.tok_helper.clone(),
            spec: self.spec.clone(),
        };
        Some(Box::new(prec))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_dist = match self.spec.dist.max_dist() {
                std::ops::Bound::Unbounded => usize::max_value(),
                std::ops::Bound::Included(max_dist) => max_dist,
                std::ops::Bound::Excluded(max_dist) => max_dist - 1,
            };
            let max_possible_dist = std::cmp::min(max_dist, stats_order.max_depth);
            let num_of_descendants = max_possible_dist - self.spec.dist.min_dist() + 1;

            return EstimationType::SELECTIVITY(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            );
        }

        EstimationType::SELECTIVITY(0.1)
    }
}
