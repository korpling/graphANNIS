use graphstorage::GraphStorage;
use graphdb::GraphDB;
use {Component, ComponentType, NodeID, AnnoKey};

use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: Arc<GraphStorage>,
    right_edges: Arc<GraphStorage>,
    cov_edges: Option<Arc<GraphStorage>>,
    tok_key: AnnoKey,
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
            tok_key: db.get_token_key(),
        })
    }

    pub fn is_token(&self, id: &NodeID) -> bool {
      
      return self.db.node_annos.get(id, &self.tok_key).is_some() 
        && self.cov_edges.is_some() && self.cov_edges.as_ref().unwrap().get_outgoing_edges(id).next().is_none();
    }

    pub fn right_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let mut out = self.right_edges.get_outgoing_edges(n);
            return out.next();
        }
    }

    pub fn left_token_for(&self, n: &NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            return Some(n.clone());
        } else {
            let mut out = self.left_edges.get_outgoing_edges(n);
            return out.next();
        }
    }

    pub fn left_right_token_for(&self, n: &NodeID) -> (Option<NodeID>, Option<NodeID>) {
        if self.is_token(n) {
            return (Some(n.clone()), Some(n.clone()));
        } else {
            let mut out_left = self.left_edges.get_outgoing_edges(n);
            let mut out_right = self.right_edges.get_outgoing_edges(n);

            return (out_left.next(), out_right.next());
        }
    }
}
