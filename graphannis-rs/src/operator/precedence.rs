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
}

impl<'a> Precedence<'a> {
    pub fn new(db : &'a mut GraphDB) -> Option<Precedence<'a>> {
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

        db.ensure_loaded(&component_order);
        db.ensure_loaded(&component_left);
        db.ensure_loaded(&component_right);
        db.ensure_loaded(&component_cov);

        let tok_helper = TokenHelper::new(db, 
            db.get_graphstorage(&component_left)?, 
            db.get_graphstorage(&component_right)?,
            db.get_graphstorage(&component_cov)?);

        let gs_order = db.get_graphstorage(&component_order)?;
        let gs_left = db.get_graphstorage(&component_left)?;
        

        Some(Precedence {
            gs_order: gs_order,
            gs_left: gs_left,
            tok_helper: tok_helper,
        })
    }
}

impl<'a> Operator for Precedence<'a> {

    fn retrieve_matches(&self, lhs : &Match) -> Box<Iterator<Item = Match>> {
         unimplemented!()
     }

    fn filter_match(&self, lhs : &Match, rhs : &Match) -> bool {
        unimplemented!()
    }
}