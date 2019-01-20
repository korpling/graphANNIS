use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::graphstorage::{EdgeContainer, GraphStatistic, GraphStorage};
use crate::annis::db::{AnnotationStorage, Graph};
use crate::annis::dfs::CycleSafeDFS;
use crate::annis::errors::*;
use crate::annis::types::{Edge, NodeID};
use num::ToPrimitive;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::ops::Bound;

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
                let outgoing: &Vec<NodeID> = &self.edges[node];
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
        if let Ok(found_idx) = self.inverse_edges.binary_search_by_key(&node, |e| e.source) {
            // check forward and backward to find all edges with this source node
            let mut start_idx = found_idx;
            while start_idx > 0 && self.inverse_edges[start_idx - 1].source == node {
                start_idx -= 1;
            }
            let mut end_idx = found_idx;
            while end_idx < self.inverse_edges.len() - 1
                && self.inverse_edges[end_idx + 1].source == node
            {
                end_idx += 1;
            }
            let outgoing: &[Edge] = &self.inverse_edges[start_idx..end_idx + 1];
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

impl GraphStorage for DenseAdjacencyListStorage {
    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(n.clone()));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };

        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(n.clone()));
        Box::new(it)
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize> {
        let mut it = CycleSafeDFS::new(self, *source, usize::min_value(), usize::max_value())
            .filter(|x| *target == x.node)
            .map(|x| x.distance);

        it.next()
    }
    fn is_connected(
        &self,
        source: &NodeID,
        target: &NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> bool {
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let mut it = CycleSafeDFS::new(self, *source, min_distance, max_distance)
            .filter(|x| *target == x.node);

        it.next().is_some()
    }

    fn get_anno_storage(&self) -> &AnnotationStorage<Edge> {
        &self.annos
    }

    fn copy(&mut self, _db: &Graph, orig: &GraphStorage) {
        self.annos.clear();
        self.edges.clear();
        self.inverse_edges.clear();

        for source in orig.source_nodes() {
            if let Some(idx) = source.to_usize() {
                if idx >= self.edges.len() {
                    self.edges.resize(idx, vec![]);
                }
                let outgoing: &mut Vec<NodeID> = &mut self.edges[idx];
                for target in orig.get_outgoing_edges(source) {

                    // insert edge 
                    outgoing.push(target);

                    // insert inverse edge
                    let e = Edge { source, target};
                    let ie = e.inverse();
                    if let Err(inverse_idx) = self.inverse_edges.binary_search(&ie) {
                        self.inverse_edges.insert(inverse_idx, ie);
                    }
                    
                    // insert annotation
                    for a in orig.get_anno_storage().get_annotations_for_item(&e) {
                        self.annos.insert(e.clone(), a);
                    }
                }
            }
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics();
    }

    fn as_edgecontainer(&self) -> &EdgeContainer {
        self
    }

    /// Return an identifier for this graph storage which is used to distinguish the different graph storages when (de-) serialized.
    fn serialization_id(&self) -> String {
        "DemseAdjacencyListV1".to_owned()
    }

    /// Serialize this graph storage.
    fn serialize_gs(&self, writer: &mut std::io::Write) -> Result<()> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    /// De-serialize this graph storage.
    fn deserialize_gs(input: &mut std::io::Read) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let mut result: DenseAdjacencyListStorage = bincode::deserialize_from(input)?;
        result.annos.after_deserialization();
        Ok(result)
    }
}
