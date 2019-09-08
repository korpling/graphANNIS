use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::Graph;
use crate::annis::types::{Component, ComponentType, NodeID};

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHelper {
    node_annos: Arc<AnnoStorage<NodeID>>,
    left_edges: Arc<dyn GraphStorage>,
    right_edges: Arc<dyn GraphStorage>,
    cov_edges: Vec<Arc<dyn GraphStorage>>,
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
}

pub fn necessary_components(db: &Graph) -> HashSet<Component> {
    let mut result = HashSet::default();
    result.insert(COMPONENT_LEFT.clone());
    result.insert(COMPONENT_RIGHT.clone());
    // we need all coverage components
    result.extend(
        db.get_all_components(Some(ComponentType::Coverage), None)
            .into_iter(),
    );

    result
}

impl TokenHelper {
    pub fn new(db: &Graph) -> Option<TokenHelper> {
        let cov_edges: Vec<Arc<dyn GraphStorage>> = db
            .get_all_components(Some(ComponentType::Coverage), None)
            .into_iter()
            .filter_map(|c| db.get_graphstorage(&c))
            .filter(|gs| {
                if let Some(stats) = gs.get_statistics() {
                    stats.nodes > 0
                } else {
                    true
                }
            })
            .collect();

        Some(TokenHelper {
            node_annos: db.node_annos.clone(),
            left_edges: db.get_graphstorage(&COMPONENT_LEFT)?,
            right_edges: db.get_graphstorage(&COMPONENT_RIGHT)?,
            cov_edges,
            tok_key: db.node_annos.get_key_id(&db.get_token_key())?,
        })
    }
    pub fn get_gs_coverage(&self) -> &Vec<Arc<dyn GraphStorage>> {
        &self.cov_edges
    }

    pub fn get_gs_left_token(&self) -> &dyn GraphStorage {
        self.left_edges.as_ref()
    }

    pub fn get_gs_right_token_(&self) -> &dyn GraphStorage {
        self.right_edges.as_ref()
    }

    pub fn is_token(&self, id: NodeID) -> bool {
        if self
            .node_annos
            .get_value_for_item_by_id(&id, self.tok_key)
            .is_some()
        {
            // check if there is no outgoing edge in any of the coverage components
            for c in self.cov_edges.iter() {
                if c.get_outgoing_edges(id).next().is_some() {
                    return false;
                }
            }
            true
        } else {
            false
        }
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
