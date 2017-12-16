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

impl<'a> TokenHelper<'a> {
    pub fn new(db: &'a mut GraphDB) -> Option<TokenHelper<'a>> {
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

        db.ensure_loaded(&component_left).ok()?;
        db.ensure_loaded(&component_right).ok()?;
        db.ensure_loaded(&component_cov).ok()?;

        Some(TokenHelper {
            db,
            left_edges: db.get_graphstorage(&component_left)?,
            right_edges: db.get_graphstorage(&component_right)?,
            cov_edges: db.get_graphstorage(&component_cov)?,
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
