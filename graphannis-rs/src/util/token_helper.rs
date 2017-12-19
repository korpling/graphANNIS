use graphstorage::GraphStorage;
use graphdb::GraphDB;
use {Component, ComponentType, NodeID};

use std::rc::Rc;

#[derive(Clone)]
pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: Rc<GraphStorage>,
    right_edges: Rc<GraphStorage>,
    cov_edges: Rc<GraphStorage>,
}

lazy_static! {

    static ref COMPONENT_LEFT : Component =  {
        let c = Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        };
        c
    };

    static ref COMPONENT_RIGHT : Component =  {
        let c = Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        };
        c
    };

    static ref COMPONENT_COV : Component =  {
        let c = Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        };
        c
    };
}

pub fn necessary_components() -> Vec<Component> {
    vec![COMPONENT_LEFT.clone(), COMPONENT_RIGHT.clone(), COMPONENT_COV.clone()]
}

impl<'a> TokenHelper<'a> {
    pub fn new(db: &'a GraphDB) -> Option<TokenHelper<'a>> {
     
        Some(TokenHelper {
            db,
            left_edges: db.get_graphstorage(&COMPONENT_LEFT)?,
            right_edges: db.get_graphstorage(&COMPONENT_RIGHT)?,
            cov_edges: db.get_graphstorage(&COMPONENT_COV)?,
        })
    }

    pub fn is_token(&self, id: &NodeID) -> bool {
        let tok = self.db.get_token_key();
        self.db.node_annos.get(id, &tok).is_some()
            && self.cov_edges.get_outgoing_edges(id).is_empty()
    }

    pub fn right_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let out = self.right_edges.get_outgoing_edges(n);
             if !out.is_empty() {
                return Some(out[0]);
            }
        }
        return None;
    }

    pub fn left_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let out = self.left_edges.get_outgoing_edges(n);
            if !out.is_empty() {
                return Some(out[0]);
            }
        }
        return None;
    }
}
