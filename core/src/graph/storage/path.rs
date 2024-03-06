use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    ops::Bound,
    path::PathBuf,
};

use crate::{
    annostorage::ondisk::AnnoStorageImpl,
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID},
    util::disk_collections::DiskMap,
};

use super::{EdgeContainer, GraphStatistic, GraphStorage};
use binary_layout::prelude::*;

pub(crate) const MAX_DEPTH: usize = 15;
pub(crate) const SERIALIZATION_ID: &str = "PathV1_D15";

binary_layout!(node_path, LittleEndian, {
    length: u8,
    nodes: [u8; MAX_DEPTH*8],
});

/// A [GraphStorage] that stores a single path for each node in memory.
pub struct PathStorage {
    paths: HashMap<NodeID, Vec<NodeID>>,
    inverse_edges: DiskMap<Edge, bool>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
    location: Option<PathBuf>,
}

impl PathStorage {
    pub fn new() -> Result<PathStorage> {
        Ok(PathStorage {
            paths: HashMap::default(),
            inverse_edges: DiskMap::default(),
            location: None,
            annos: AnnoStorageImpl::new(None)?,
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
        let lower_bound = Edge {
            source: node,
            target: NodeID::MIN,
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::MAX,
        };
        Box::new(
            self.inverse_edges
                .range(lower_bound..upper_bound)
                .map_ok(|(e, _)| e.target),
        )
    }

    fn has_ingoing_edges(&self, node: NodeID) -> Result<bool> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::MIN,
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::MAX,
        };

        if let Some(edge) = self.inverse_edges.range(lower_bound..upper_bound).next() {
            edge?;
            Ok(true)
        } else {
            Ok(false)
        }
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
        todo!()
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
        todo!()
    }

    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        todo!()
    }

    fn get_anno_storage(&self) -> &dyn crate::annostorage::EdgeAnnotationStorage {
        &self.annos
    }

    fn copy(
        &mut self,
        _node_annos: &dyn crate::annostorage::NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        todo!()
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
        todo!()
    }

    fn save_to(&self, location: &std::path::Path) -> Result<()> {
        todo!()
    }
}
