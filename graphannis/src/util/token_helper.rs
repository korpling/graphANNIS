use graphstorage::GraphStorage;
use graphdb::GraphDB;
use {Component, ComponentType, NodeID};

use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: Arc<GraphStorage>,
    right_edges: Arc<GraphStorage>,
    cov_edges: Option<Arc<GraphStorage>>,
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
            cov_edges: db.get_graphstorage(&COMPONENT_COV),
        })
    }

    pub fn is_token(&self, id: &NodeID) -> bool {
        let tok = self.db.get_token_key();
        if self.db.node_annos.get(id, &tok).is_some() {
            if let Some(ref cov_edges) = self.cov_edges {
                // check if there are no outgoing edges for this node in the coverage component
                return cov_edges.get_outgoing_edges(id).next().is_none();
            } else {
                // if there is no covering component, the outgoing edges are always empty
                return true;
            }
        }
        return false;
    }

    pub fn right_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let mut out = self.right_edges.get_outgoing_edges(n);
             if let Some(out) = out.next() {
                return Some(out);
            }
        }
        return None;
    }

    pub fn left_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let mut out = self.left_edges.get_outgoing_edges(n);
            if let Some(out) = out.next() {
                return Some(out);
            }
        }
        return None;
    }
}
