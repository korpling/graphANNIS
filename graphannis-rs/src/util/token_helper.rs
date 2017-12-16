use graphstorage::{GraphStorage};
use graphdb::GraphDB;
use {NodeID, ComponentType, Component};

use std::rc::Rc;

#[derive(Clone)]
pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: Rc<GraphStorage>,
    right_edges: Rc<GraphStorage>,
    cov_edges: Rc<GraphStorage>,
}

impl<'a> TokenHelper<'a> {

    pub fn new(
        db: &'a GraphDB,
        left_edges: Rc<GraphStorage>,
        right_edges: Rc<GraphStorage>,
        cov_edges: Rc<GraphStorage>,
    ) -> TokenHelper<'a> {
        TokenHelper {
            db,
            left_edges,
            right_edges,
            cov_edges,
        }
    }

    pub fn is_token(&self, id : &NodeID) -> bool {
        let tok = self.db.get_token_key();
        self.db.node_annos.get(id, &tok).is_some() 
            &&  self.cov_edges.get_outgoing_edges(id).is_empty()
    }
}
