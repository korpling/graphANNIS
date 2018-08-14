use operator::{Operator, OperatorSpec};
use {Annotation, Component, ComponentType, Match};
use graphdb::GraphDB;
use graphstorage::{GraphStorage};
use operator::EstimationType;
use util::token_helper;
use util::token_helper::TokenHelper;

use std::sync::Arc;
use std;

#[derive(Clone, Debug)]
pub struct IdenticalCoverageSpec;

#[derive(Clone)]
pub struct IdenticalCoverage {
    gs_left: Arc<GraphStorage>,
    gs_order: Arc<GraphStorage>,
    tok_helper: TokenHelper,
}

lazy_static! {

    static ref COMPONENT_LEFT : Component =  {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };

    static ref COMPONENT_ORDER : Component =  {
        Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for IdenticalCoverageSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let mut v: Vec<Component> = vec![COMPONENT_LEFT.clone(), COMPONENT_ORDER.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        let optional_op = IdenticalCoverage::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl IdenticalCoverage {
    pub fn new(db: &GraphDB) -> Option<IdenticalCoverage> {
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
        let tok_helper = TokenHelper::new(db)?;


        Some(IdenticalCoverage {
            gs_left,
            gs_order,
            tok_helper: tok_helper,
        })
    }
}

impl std::fmt::Display for IdenticalCoverage {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_=_")
    }
}

impl Operator for IdenticalCoverage {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let n_left = self.tok_helper.left_token_for(&lhs.node);
        let n_right = self.tok_helper.right_token_for(&lhs.node);

        let mut result: Vec<Match> = Vec::new();

        if n_left.is_some() && n_right.is_some() {
            let n_left = n_left.unwrap();
            let n_right = n_right.unwrap();

            if n_left == n_right {
                // covered range is exactly one token, add token itself
                result.push(Match { node: n_left.clone(), anno: Annotation::default() });
            }

            // find left-aligned non-token
            let v = self.gs_left.get_outgoing_edges(&n_left);
            for c in v.into_iter() {
                // check if also right-aligned
                if let Some(c_right) = self.tok_helper.right_token_for(&c) {
                    if n_right == c_right {
                        result.push(Match {node : c, anno: Annotation::default()});
                    }
                } 
            }

        }


        return Box::new(result.into_iter());
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_lhs = self.tok_helper.left_token_for(&lhs.node);
        let end_lhs = self.tok_helper.right_token_for(&lhs.node);

        let start_rhs = self.tok_helper.left_token_for(&rhs.node);
        let end_rhs = self.tok_helper.right_token_for(&rhs.node);

        if start_lhs.is_none() || end_lhs.is_none() || start_rhs.is_none() || end_rhs.is_none() {
            return false;
        }

        return start_lhs.unwrap() == start_rhs.unwrap() && end_lhs.unwrap() == end_rhs.unwrap();
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator(&self) -> Option<Box<Operator>> {
        return Some(Box::new(self.clone()));
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(order_stats) = self.gs_order.get_statistics() {
            let num_of_token = order_stats.nodes as f64;

             // Assume two nodes have same identical coverage if they have the same
            // left covered token and the same length (right covered token is not independent
            // of the left one, this is why we should use length).
            // The probability for the same length is taken is assumed to be 1.0, histograms
            // of the distribution would help here.

            return EstimationType::SELECTIVITY(1.0 / num_of_token);
        }
        return EstimationType::SELECTIVITY(0.1)
    }

    
}
