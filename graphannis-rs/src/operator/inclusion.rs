use super::{Operator, OperatorSpec};
use {Annotation, Component, ComponentType, Match};
use graphdb::GraphDB;
use graphstorage::GraphStorage;
use util::token_helper;
use util::token_helper::TokenHelper;

use std::rc::Rc;
use std;

#[derive(Clone)]
pub struct InclusionSpec;

pub struct Inclusion<'a> {
    gs_order: Rc<GraphStorage>,
    gs_left: Rc<GraphStorage>,
    gs_right: Rc<GraphStorage>,
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

    static ref COMPONENT_LEFT : Component =  {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };

    static ref COMPONENT_RIGHT : Component =  {
        Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for InclusionSpec {
    fn necessary_components(&self) -> Vec<Component> {
        let mut v: Vec<Component> = vec![COMPONENT_ORDER.clone(), COMPONENT_LEFT.clone(), COMPONENT_RIGHT.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        let optional_op = Inclusion::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl<'a> Inclusion<'a> {
    pub fn new(db: &'a GraphDB) -> Option<Inclusion<'a>> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let gs_right = db.get_graphstorage(&COMPONENT_RIGHT)?;
        

        let tok_helper = TokenHelper::new(db)?;

        Some(Inclusion {
            gs_order,
            gs_left,
            gs_right,
            tok_helper,
        })
    }
}

impl<'a> Operator for Inclusion<'a> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {
        if let (Some(start_lhs), Some(end_lhs)) = (
            self.tok_helper.left_token_for(&lhs.node),
            self.tok_helper.right_token_for(&lhs.node),
        ) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(&start_lhs, &end_lhs) {
                // find each token which is between the left and right border
                let it = self.gs_order.find_connected(&start_lhs, 0, l)
                        .flat_map(move |t| {
                            let it_aligned = self.gs_left.get_outgoing_edges(&t).into_iter()
                                .filter(move |n| {
                                    // right-aligned token of candidate
                                    let mut end_n = self.gs_right.get_outgoing_edges(&n);
                                    if let Some(end_n) = end_n.next() {
                                        // path between right-most tokens exists in ORDERING component 
                                        // and has maximum length l
                                        return self.gs_order.is_connected(&end_n, &end_lhs, 0, l);
                                    }
                                    return false;
                                });
                            // return the token itself and all aligned nodes
                            return std::iter::once(t).chain(it_aligned);
                        })
                        .map(|n| Match{node: n, anno: Annotation::default()});
                return Box::new(it);
            }

            return Box::new(std::iter::empty());
        }

        unimplemented!();
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            self.tok_helper.left_token_for(&lhs.node),
            self.tok_helper.right_token_for(&lhs.node),
            self.tok_helper.left_token_for(&rhs.node),
            self.tok_helper.right_token_for(&rhs.node),
        ) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(&start_lhs, &end_lhs) {
                // path between left-most tokens exists in ORDERING component and has maximum length l
                if self.gs_order.is_connected(&start_lhs, &start_rhs, 0, l)
                // path between right-most tokens exists in ORDERING component and has maximum length l
                && self.gs_order.is_connected(&end_rhs, &end_lhs, 0, l) {
                    return true;
                }
            }
        }

        return false;
    }
}
