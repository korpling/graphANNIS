use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::Graph;
use crate::annis::types::{Component, ComponentType, NodeID};

use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHelper {
    node_annos: Arc<AnnoStorage<NodeID>>,
    left_edges: Arc<GraphStorage>,
    right_edges: Arc<GraphStorage>,
    cov_edges: Option<Arc<GraphStorage>>,
    tok_key: usize,
}

lazy_static! {
    static ref COMPONENT_LEFT: Component = {
        Component {
            ctype: ComponentType::LeftToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
    static ref COMPONENT_RIGHT: Component = {
        Component {
            ctype: ComponentType::RightToken,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
    static ref COMPONENT_COV: Component = {
        Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

pub fn necessary_components() -> Vec<Component> {
    vec![
        COMPONENT_LEFT.clone(),
        COMPONENT_RIGHT.clone(),
        COMPONENT_COV.clone(),
    ]
}

impl TokenHelper {
    pub fn new(db: &Graph) -> Option<TokenHelper> {
        Some(TokenHelper {
            node_annos: db.node_annos.clone(),
            left_edges: db.get_graphstorage(&COMPONENT_LEFT)?,
            right_edges: db.get_graphstorage(&COMPONENT_RIGHT)?,
            cov_edges: db.get_graphstorage(&COMPONENT_COV),
            tok_key: db.node_annos.get_key_id(&db.get_token_key())?,
        })
    }

    pub fn is_token(&self, id: NodeID) -> bool {
        self.node_annos
            .get_value_for_item_by_id(&id, self.tok_key)
            .is_some()
            && self.cov_edges.is_some()
            && self
                .cov_edges
                .as_ref()
                .unwrap()
                .get_outgoing_edges(id)
                .next()
                .is_none()
    }

    pub fn right_token_for(&self, n: NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            Some(n)
        } else {
            let mut out = self.right_edges.get_outgoing_edges(n);
            out.next()
        }
    }

    pub fn left_token_for(&self, n: NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            Some(n)
        } else {
            let mut out = self.left_edges.get_outgoing_edges(n);
            out.next()
        }
    }

    pub fn left_right_token_for(&self, n: NodeID) -> (Option<NodeID>, Option<NodeID>) {
        if self.is_token(n) {
            (Some(n), Some(n))
        } else {
            let mut out_left = self.left_edges.get_outgoing_edges(n);
            let mut out_right = self.right_edges.get_outgoing_edges(n);

            (out_left.next(), out_right.next())
        }
    }
}
