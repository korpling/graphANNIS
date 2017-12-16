use graphstorage::{EdgeContainer, ReadableGraphStorage};
use graphdb::GraphDB;
use {NodeID, ComponentType, Component};

#[derive(Clone)]
pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: &'a ReadableGraphStorage,
    right_edges: &'a ReadableGraphStorage,
    cov_edges: &'a ReadableGraphStorage,
}

impl<'a> TokenHelper<'a> {

    pub fn new(
        db: &'a GraphDB,
        left_edges: &'a ReadableGraphStorage,
        right_edges: &'a ReadableGraphStorage,
        cov_edges: &'a ReadableGraphStorage,
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
