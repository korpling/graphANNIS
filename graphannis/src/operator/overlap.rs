use super::{Operator, OperatorSpec};
use {Annotation, Component, ComponentType, Match, NodeID};
use graphdb::GraphDB;
use graphstorage::GraphStorage;
use util::token_helper;
use util::token_helper::TokenHelper;
use std::collections::HashSet;

use std::rc::Rc;
use std;

#[derive(Clone)]
pub struct OverlapSpec;

pub struct  Overlap<'a> {
    gs_order: Rc<GraphStorage>,
    gs_cov: Rc<GraphStorage>,
    gs_invcov: Rc<GraphStorage>,
    tok_helper: TokenHelper<'a>,
}

lazy_static! {

    static ref COMPONENT_ORDER : Component =  {
        Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };


    static ref COMPONENT_COVERAGE : Component =  {
        Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };

    static ref COMPONENT_INV_COVERAGE : Component =  {
        Component {
            ctype: ComponentType::InverseCoverage,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for  OverlapSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let mut v: Vec<Component> = vec![COMPONENT_ORDER.clone(), 
            COMPONENT_COVERAGE.clone(), COMPONENT_INV_COVERAGE.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        let optional_op =  Overlap::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl<'a>  Overlap<'a> {
    pub fn new(db: &'a GraphDB) -> Option< Overlap<'a>> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
        let gs_cov = db.get_graphstorage(&COMPONENT_COVERAGE)?;
        let gs_invcov = db.get_graphstorage(&COMPONENT_INV_COVERAGE)?;        

        let tok_helper = TokenHelper::new(db)?;

        Some( Overlap {
            gs_order,
            gs_cov,
            gs_invcov,
            tok_helper,
        })
    }
}

impl<'a> std::fmt::Display for Overlap<'a> {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_o_")
    }
}

impl<'a> Operator for  Overlap<'a> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {

        // use set to filter out duplicates
        let mut result = HashSet::new();

        let covered : Box<Iterator<Item=NodeID>> = if self.tok_helper.is_token(&lhs.node) {
            Box::new(std::iter::once(lhs.node))
        }  else {
            // all covered token
            Box::new(self.gs_cov.find_connected(&lhs.node, 1, 1))
        };

        for t in covered {
             // get all nodes that are covering the token
            for n in self.gs_invcov.find_connected(&t, 1, 1) {
                result.insert(n);
            }
            // also add the token itself
            result.insert(t);
        }

        return Box::new(result.into_iter().map(|n| Match{node: n, anno : Annotation::default()}));
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            self.tok_helper.left_token_for(&lhs.node),
            self.tok_helper.right_token_for(&lhs.node),
            self.tok_helper.left_token_for(&rhs.node),
            self.tok_helper.right_token_for(&rhs.node),
        ) {
            // TODO: why not isConnected()? (instead of distance)
            // path between LHS left-most token and RHS right-most token exists in ORDERING component
            if self.gs_order.distance(&start_lhs, &end_rhs).is_some()
                // path between LHS left-most token and RHS right-most token exists in ORDERING component
                && self.gs_order.distance(&start_rhs, &end_lhs).is_some() {
                    return true;
            }
        }
        return false;
    }
}
