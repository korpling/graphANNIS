use super::{EdgeContainer, GraphStatistic, GraphStorage};
use crate::{
    annostorage::{inmemory::AnnoStorageImpl, AnnotationStorage},
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID},
};
use itertools::Itertools;
use num_traits::ToPrimitive;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use std::{ops::Bound, path::Path};

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct DenseAdjacencyListStorage {
    edges: Vec<Option<NodeID>>,
    inverse_edges: FxHashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorageImpl<Edge>,
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
            inverse_edges: FxHashMap::default(),
            annos: AnnoStorageImpl::new(),
            stats: None,
        }
    }
}

impl EdgeContainer for DenseAdjacencyListStorage {
    /// Get all outgoing edges for a given `node`.
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(node) = node.to_usize() {
            if node < self.edges.len() {
                if let Some(outgoing) = self.edges[node] {
                    return Box::new(std::iter::once(Ok(outgoing)));
                }
            }
        }
        Box::new(std::iter::empty())
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(ingoing) = self.inverse_edges.get(&node) {
            return match ingoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(Ok(ingoing[0]))),
                _ => Box::new(ingoing.iter().map(|e| Ok(e.clone()))),
            };
        }
        Box::new(std::iter::empty())
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }

    /// Provides an iterator over all nodes of this edge container that are the source an edge
    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let it = self
            .edges
            .iter()
            .enumerate()
            .filter(|(_, outgoing)| outgoing.is_some())
            .filter_map(|(key, _)| key.to_u64())
            .map(|n| Ok(n));
        Box::new(it)
    }
}

impl GraphStorage for DenseAdjacencyListStorage {
    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map_ok(|x| x.node)
            .filter_ok(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };

        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .map_ok(|x| x.node)
            .filter_ok(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>> {
        let mut it = CycleSafeDFS::new(self, source, usize::min_value(), usize::max_value())
            .filter_ok(|x| target == x.node)
            .map_ok(|x| x.distance);

        match it.next() {
            Some(distance) => {
                let distance = distance?;
                Ok(Some(distance))
            }
            None => Ok(None),
        }
    }
    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let mut it = CycleSafeDFS::new(self, source, min_distance, max_distance)
            .filter_ok(|x| target == x.node);

        Ok(it.next().is_some())
    }

    fn get_anno_storage(&self) -> &dyn AnnotationStorage<Edge> {
        &self.annos
    }

    fn copy(
        &mut self,
        node_annos: &dyn AnnotationStorage<NodeID>,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.annos.clear()?;
        self.edges.clear();
        self.inverse_edges.clear();

        if let Some(largest_idx) = node_annos.get_largest_item().and_then(|idx| idx.to_usize()) {
            debug!("Resizing dense adjacency list to size {}", largest_idx + 1);
            self.edges.resize(largest_idx + 1, None);

            for source in orig.source_nodes() {
                let source = source?;
                if let Some(idx) = source.to_usize() {
                    if let Some(target) = orig.get_outgoing_edges(source).next() {
                        let target = target?;
                        // insert edge
                        self.edges[idx] = Some(target);

                        // insert inverse edge
                        let e = Edge { source, target };
                        let inverse_entry = self
                            .inverse_edges
                            .entry(e.target)
                            .or_insert_with(Vec::default);
                        // no need to insert it: edge already exists
                        if let Err(insertion_idx) = inverse_entry.binary_search(&e.source) {
                            inverse_entry.insert(insertion_idx, e.source);
                        }
                        // insert annotation
                        for a in orig.get_anno_storage().get_annotations_for_item(&e) {
                            self.annos.insert(e.clone(), a)?;
                        }
                    }
                }
            }
            self.stats = orig.get_statistics().cloned();
            self.annos.calculate_statistics();
        }
        Ok(())
    }

    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }

    fn inverse_has_same_cost(&self) -> bool {
        true
    }

    /// Return an identifier for this graph storage which is used to distinguish the different graph storages when (de-) serialized.
    fn serialization_id(&self) -> String {
        "DenseAdjacencyListV1".to_owned()
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let mut result: Self = super::default_deserialize_gs(location)?;
        result.annos.after_deserialization();
        Ok(result)
    }

    fn save_to(&self, location: &Path) -> Result<()> {
        super::default_serialize_gs(self, location)?;
        Ok(())
    }
}
