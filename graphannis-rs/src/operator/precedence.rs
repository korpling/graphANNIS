use super::Operator;
use {Match, Component, ComponentType};
use graphdb::GraphDB;
use graphstorage::{GraphStorage};
use util::token_helper::TokenHelper;

use std::rc::Rc;

pub struct Precedence <'a>{
    gs_order: Rc<GraphStorage>,
    gs_left: Rc<GraphStorage>,
    tok_helper : TokenHelper<'a>,
    segmentation : Option<String>,
    min_dist : usize,
    max_dist : usize,
}

impl<'a> Precedence<'a> {
    pub fn new(db : &'a mut GraphDB, segmentation : Option<String>, min_dist : usize, max_dist: usize) -> Option<Precedence<'a>> {
        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        };
        let component_left = Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        };
        let component_right = Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        };
        let component_cov = Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        };

        db.ensure_loaded(&component_order).ok()?;
        db.ensure_loaded(&component_left).ok()?;
        db.ensure_loaded(&component_right).ok()?;
        db.ensure_loaded(&component_cov).ok()?;


        let gs_order = db.get_graphstorage(&component_order)?;
        let gs_left = db.get_graphstorage(&component_left)?;

        let tok_helper = TokenHelper::new(db)?;
        
        Some(Precedence {
            gs_order: gs_order,
            gs_left: gs_left,
            tok_helper: tok_helper,
            segmentation,
            min_dist,
            max_dist,
        })
    }
}

impl<'a> Operator for Precedence<'a> {

    fn retrieve_matches(&self, lhs : &Match) -> Box<Iterator<Item = Match>> {
         unimplemented!()
     }

    fn filter_match(&self, lhs : &Match, rhs : &Match) -> bool {
        let start_end = if self.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.right_token_for(&lhs.node);
            let end = self.tok_helper.left_token_for(&rhs.node);
            if start.is_none() || end.is_none() {
                return false;
            }
            (start.unwrap(), end.unwrap())
        };

        return self.gs_order.is_connected(&start_end.0, &start_end.1, self.min_dist, self.max_dist);
    }
}