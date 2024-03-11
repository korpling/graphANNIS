use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    ops::Bound,
    path::PathBuf,
};

use crate::{
    annostorage::inmemory::AnnoStorageImpl,
    annostorage::AnnotationStorage,
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID},
};

use super::{EdgeContainer, GraphStatistic, GraphStorage};

pub(crate) const MAX_DEPTH: usize = 15;
pub(crate) const SERIALIZATION_ID: &str = "PathV1_D15";

/// A [GraphStorage] that stores a single path for each node in memory.
#[derive(Serialize, Deserialize)]
pub struct PathStorage {
    paths: HashMap<NodeID, Vec<NodeID>>,
    inverse_edges: HashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
    location: Option<PathBuf>,
}

impl PathStorage {
    pub fn new() -> Result<PathStorage> {
        Ok(PathStorage {
            paths: HashMap::default(),
            inverse_edges: HashMap::default(),
            location: None,
            annos: AnnoStorageImpl::new(),
            stats: None,
        })
    }

    fn get_outgoing_edge(&self, node: NodeID) -> Result<Option<NodeID>> {
        if let Some(p) = self.paths.get(&node) {
            Ok(p.first().copied())
        } else {
            Ok(None)
        }
    }
}

impl EdgeContainer for PathStorage {
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        match self.get_outgoing_edge(node) {
            Ok(Some(n)) => Box::new(std::iter::once(Ok(n))),
            Ok(None) => Box::new(std::iter::empty()),
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(ingoing) = self.inverse_edges.get(&node) {
            return match ingoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(Ok(ingoing[0]))),
                _ => Box::new(ingoing.iter().map(|e| Ok(*e))),
            };
        }
        Box::new(std::iter::empty())
    }

    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let it = self.paths.keys().copied().map(Ok);
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for PathStorage {
    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut result = Vec::default();
        if min_distance == 0 {
            result.push(Ok(node));
        }

        if let Some(path) = self.paths.get(&node) {
            // The 0th index of the path is the node with distance 1, so always subtract 1
            let start = min_distance.saturating_sub(1);

            let end = match max_distance {
                std::ops::Bound::Included(max_distance) => max_distance,
                std::ops::Bound::Excluded(max_distance) => max_distance.saturating_sub(1),
                std::ops::Bound::Unbounded => path.len(),
            };
            let end = end.min(path.len());
            if start < end {
                result.extend(path[start..end].iter().map(|n| Ok(*n)));
            }
        }
        Box::new(result.into_iter())
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut visited = HashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::MAX,
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance - 1,
        };

        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .filter_map_ok(move |x| {
                if visited.insert(x.node) {
                    Some(x.node)
                } else {
                    None
                }
            });
        Box::new(it)
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>> {
        if let Some(path) = self.paths.get(&source) {
            // Find the target node in the path. The path starts at distance "0".
            let result = path.iter().position(|n| *n == target).map(|idx| idx + 1);
            Ok(result)
        } else {
            Ok(None)
        }
    }

    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        if let Some(path) = self.paths.get(&source) {
            // There is a connection when the target node is located in the path
            // (given the min/max constraints)
            let start = min_distance.saturating_sub(1).clamp(0, path.len());
            let end = match max_distance {
                Bound::Unbounded => path.len(),
                Bound::Included(max_distance) => max_distance,
                Bound::Excluded(max_distance) => max_distance.saturating_sub(1),
            };
            let end = end.clamp(0, path.len());
            for p in path.iter().take(end).skip(start) {
                if *p == target {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn get_anno_storage(&self) -> &dyn crate::annostorage::EdgeAnnotationStorage {
        &self.annos
    }

    fn copy(
        &mut self,
        _node_annos: &dyn crate::annostorage::NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.paths.clear();
        self.inverse_edges.clear();

        // Get the paths for all source nodes in the original graph storage
        for source in orig.source_nodes().sorted_by(|a, b| {
            let a = a.as_ref().unwrap_or(&0);
            let b = b.as_ref().unwrap_or(&0);
            a.cmp(b)
        }) {
            let source = source?;

            let mut path = Vec::new();
            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), source, 1, MAX_DEPTH);
            for step in dfs {
                let step = step?;
                let target = step.node;

                path.push(target);

                if step.distance == 1 {
                    let edge = Edge { source, target };
                    // insert inverse edge
                    let inverse_entry = self.inverse_edges.entry(edge.target).or_default();
                    // no need to insert it: edge already exists
                    if let Err(insertion_idx) = inverse_entry.binary_search(&edge.source) {
                        inverse_entry.insert(insertion_idx, edge.source);
                    }

                    // Copy all annotations for this edge
                    for a in orig.get_anno_storage().get_annotations_for_item(&edge)? {
                        self.annos.insert(edge.clone(), a)?;
                    }
                }
            }
            self.paths.insert(source, path);
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics()?;
        Ok(())
    }

    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }

    fn serialization_id(&self) -> String {
        SERIALIZATION_ID.to_string()
    }

    fn load_from(location: &std::path::Path) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let mut result: Self = super::default_deserialize_gs(location)?;
        result.annos.after_deserialization();
        Ok(result)
    }

    fn save_to(&self, location: &std::path::Path) -> Result<()> {
        super::default_serialize_gs(self, location)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
