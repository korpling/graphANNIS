use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::graphstorage::EdgeContainer;
use crate::annis::db::graphstorage::GraphStatistic;
use crate::annis::types::{Edge, NodeID};
use num::ToPrimitive;

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct DenseAdjacencyListStorage {
    edges: Vec<Vec<NodeID>>,
    inverse_edges: Vec<Edge>,
    annos: AnnoStorage<Edge>,
    stats: Option<GraphStatistic>,
}

impl Default for DenseAdjacencyListStorage {
    fn default() -> Self {
        DenseAdjacencyListStorage::new()
    }
}

impl DenseAdjacencyListStorage {
    pub fn new() -> DenseAdjacencyListStorage {
        DenseAdjacencyListStorage {
            edges: Vec::default(),
            inverse_edges: Vec::default(),
            annos: AnnoStorage::new(),
            stats: None,
        }
    }
}

impl EdgeContainer for DenseAdjacencyListStorage {
    /// Get all outgoing edges for a given `node`.
    fn get_outgoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(node) = node.to_usize() {
            if node < self.edges.len() {
                let outgoing : &Vec<NodeID> = &self.edges[node];
                return match outgoing.len() {
                    0 => Box::new(std::iter::empty()),
                    1 => Box::new(std::iter::once(outgoing[0])),
                    _ => Box::new(outgoing.iter().cloned()),
                };
            }
        }
        Box::new(std::iter::empty())
    }

    /// Get all incoming edges for a given `node`.
    fn get_ingoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        // inverse is a sorted vector of edges, find any index with the correct source node
        if let Ok(found_idx) =  self.inverse_edges.binary_search_by_key(&node, |e| e.source) {
            // check forward and backward to find all edges with this source node
            let mut start_idx = found_idx;
            while start_idx > 0 && self.inverse_edges[start_idx-1].source == node {
                start_idx -= 1;
            }
            let mut end_idx = found_idx;
            while end_idx < self.inverse_edges.len() - 1 && self.inverse_edges[end_idx+1].source == node {
                end_idx += 1;
            }
            let outgoing : &[Edge] = &self.inverse_edges[start_idx..end_idx+1];
            return match outgoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(outgoing[0].target)),
                _ => Box::new(outgoing.iter().map(|e| e.target)),
            };
        }
        Box::new(std::iter::empty())
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }

    /// Provides an iterator over all nodes of this edge container that are the source an edge
    fn source_nodes<'a>(&'a self) -> Box<Iterator<Item = NodeID> + 'a> {
        let it = self
            .edges
            .iter()
            .enumerate()
            .filter(|(_, outgoing)| !outgoing.is_empty())
            .filter_map(|(key, _)| key.to_u64());
        Box::new(it)
    }
}
