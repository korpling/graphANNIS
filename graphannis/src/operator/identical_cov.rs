use super::{Operator, OperatorSpec};
use {Annotation, Component, ComponentType, Match};
use graphdb::GraphDB;
use graphstorage::GraphStorage;
use util::token_helper;
use util::token_helper::TokenHelper;

use std::rc::Rc;
use std;

#[derive(Clone)]
pub struct IdenticalCoverageSpec {}

pub struct IdenticalCoverage<'a> {
    gs_left: Rc<GraphStorage>,
    tok_helper: TokenHelper<'a>,
}

lazy_static! {

    static ref COMPONENT_LEFT : Component =  {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for IdenticalCoverageSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let mut v: Vec<Component> = vec![COMPONENT_LEFT.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        let optional_op = IdenticalCoverage::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl<'a> IdenticalCoverage<'a> {
    pub fn new(db: &'a GraphDB) -> Option<IdenticalCoverage<'a>> {
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let tok_helper = TokenHelper::new(db)?;


        Some(IdenticalCoverage {
            gs_left,
            tok_helper: tok_helper,
        })
    }
}

impl<'a> std::fmt::Display for IdenticalCoverage<'a> {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_=_")
    }
}

impl<'a> Operator for IdenticalCoverage<'a> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {
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
}
