use annis::graphstorage::EdgeContainer;
use annis::graphdb::GraphDB;
use annis::NodeID;
use annis::AnnoKey;

pub struct TokenHelper<'a> {
    db: &'a GraphDB,
    left_edges: &'a EdgeContainer,
    right_edges: &'a EdgeContainer,
    cov_edges: &'a EdgeContainer,
}

impl<'a> TokenHelper<'a> {
    pub fn new(
        db: &'a GraphDB,
        left_edges: &'a EdgeContainer,
        right_edges: &'a EdgeContainer,
        cov_edges: &'a EdgeContainer,
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
