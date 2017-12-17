use super::{Operator, OperatorSpec};
use {Component, ComponentType, Match};
use graphdb::GraphDB;
use graphstorage::GraphStorage;
use util::token_helper;
use util::token_helper::TokenHelper;
use std::collections::HashSet;

use std::rc::Rc;


pub struct PrecedenceSpec {
    pub segmentation: Option<String>,
    pub min_dist: usize,
    pub max_dist: usize,
}

pub struct Precedence<'a> {
    gs_order: Rc<GraphStorage>,
    gs_left: Rc<GraphStorage>,
    tok_helper: TokenHelper<'a>,
    spec: PrecedenceSpec,
}

lazy_static! {

    static ref COMPONENT_ORDER : Component =  {
        Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };

    static ref COMPONENT_LEFT : Component =  {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for PrecedenceSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let mut v : Vec<Component> = vec![COMPONENT_ORDER.clone(), COMPONENT_LEFT.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }
}

impl<'a> Precedence<'a> {
    pub fn new(db: &'a GraphDB, spec: PrecedenceSpec) -> Option<Precedence<'a>> {
     
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
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

impl<'a> Operator for Precedence<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        unimplemented!()
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
}
