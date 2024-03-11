use itertools::Itertools;
use memmap2::{Mmap, MmapMut};
use normpath::PathExt;
use std::{
    collections::HashSet, convert::TryInto, fs::File, io::BufReader, ops::Bound, path::PathBuf,
};
use tempfile::tempfile;
use transient_btree_index::BtreeConfig;

use crate::{
    annostorage::{ondisk::AnnoStorageImpl, AnnotationStorage},
    dfs::CycleSafeDFS,
    errors::Result,
    try_as_boxed_iter,
    types::{Edge, NodeID},
    util::disk_collections::{DiskMap, EvictionStrategy, DEFAULT_BLOCK_CACHE_CAPACITY},
};

use super::{EdgeContainer, GraphStatistic, GraphStorage};
use binary_layout::prelude::*;

pub(crate) const MAX_DEPTH: usize = 15;
pub(crate) const SERIALIZATION_ID: &str = "DiskPathV1_D15";
const ENTRY_SIZE: usize = (MAX_DEPTH * 8) + 1;

binary_layout!(node_path, LittleEndian, {
    length: u8,
    nodes: [u8; MAX_DEPTH*8],
});

/// A [GraphStorage] that stores a single path for each node on disk.
pub struct DiskPathStorage {
    paths: Mmap,
    paths_file_size: u64,
    inverse_edges: DiskMap<Edge, bool>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
    location: Option<PathBuf>,
}

fn offset_in_file(n: NodeID) -> u64 {
    n * (ENTRY_SIZE as u64)
}

fn offset_in_path(path_idx: usize) -> usize {
    path_idx * 8
}

impl DiskPathStorage {
    pub fn new() -> Result<DiskPathStorage> {
        let paths = unsafe { Mmap::map(&tempfile()?)? };
        Ok(DiskPathStorage {
            paths,
            paths_file_size: 0,
            inverse_edges: DiskMap::default(),
            location: None,
            annos: AnnoStorageImpl::new(None)?,
            stats: None,
        })
    }

    fn get_outgoing_edge(&self, node: NodeID) -> Result<Option<NodeID>> {
        if node > self.max_node_id()? {
            return Ok(None);
        }
        let offset = offset_in_file(node) as usize;

        let view = node_path::View::new(&self.paths[offset..(offset + ENTRY_SIZE)]);
        if view.length().read() == 0 {
            // No outgoing edges
            Ok(None)
        } else {
            // Read the node ID at the first position
            let buffer: [u8; 8] = view.nodes()[offset_in_path(0)..offset_in_path(1)].try_into()?;
            Ok(Some(u64::from_le_bytes(buffer)))
        }
    }

    fn max_node_id(&self) -> Result<u64> {
        let number_of_entries = self.paths_file_size / (ENTRY_SIZE as u64);
        Ok(number_of_entries - 1)
    }

    fn path_for_node(&self, node: NodeID) -> Result<Vec<NodeID>> {
        if node > self.max_node_id()? {
            return Ok(Vec::default());
        }
        let offset = offset_in_file(node) as usize;

        let view = node_path::View::new(&self.paths[offset..(offset + ENTRY_SIZE)]);
        let length = view.length().read();
        if length == 0 {
            // No outgoing edges
            Ok(Vec::default())
        } else {
            // Add all path elements
            let mut result = Vec::with_capacity(length as usize);
            for i in 0..length {
                let i = i as usize;
                let element_buffer: [u8; 8] =
                    view.nodes()[offset_in_path(i)..offset_in_path(i + 1)].try_into()?;
                let ancestor_id = u64::from_le_bytes(element_buffer);
                result.push(ancestor_id);
            }

            Ok(result)
        }
    }
}

impl EdgeContainer for DiskPathStorage {
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
        let max_node_id = try_as_boxed_iter!(self.max_node_id());
        // ignore node entries with empty path in result
        let it = (0..=max_node_id)
            .map(move |n| {
                let offset = offset_in_file(n) as usize;
                let view = node_path::View::new(&self.paths[offset..(offset + ENTRY_SIZE)]);

                let path_length = view.length().read();
                if path_length == 0 {
                    Ok(None)
                } else {
                    Ok(Some(n))
                }
            })
            .filter_map_ok(|n| n);
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for DiskPathStorage {
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

        let path = try_as_boxed_iter!(self.path_for_node(node));
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
        let path = self.path_for_node(source)?;
        // Find the target node in the path. The path starts at distance "0".
        let result = path
            .into_iter()
            .position(|n| n == target)
            .map(|idx| idx + 1);
        Ok(result)
    }

    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        let path = self.path_for_node(source)?;
        // There is a connection when the target node is located in the path (given the min/max constraints)
        let start = min_distance.saturating_sub(1).clamp(0, path.len());
        let end = match max_distance {
            Bound::Unbounded => path.len(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance.saturating_sub(1),
        };
        let end = end.clamp(0, path.len());
        for p in path.into_iter().take(end).skip(start) {
            if p == target {
                return Ok(true);
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
        self.inverse_edges.clear();

        // Create a new file which is large enough to contain the paths for all nodes.
        let max_node_id = orig
            .source_nodes()
            .fold_ok(0, |acc, node_id| acc.max(node_id))?;
        let file_capacity = (max_node_id + 1) * (ENTRY_SIZE as u64);

        let file = tempfile::tempfile()?;
        if file_capacity > 0 {
            file.set_len(file_capacity)?;
        }
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        // Get the paths for all source nodes in the original graph storage
        for source in orig.source_nodes().sorted_by(|a, b| {
            let a = a.as_ref().unwrap_or(&0);
            let b = b.as_ref().unwrap_or(&0);
            a.cmp(b)
        }) {
            let source = source?;

            let offset = offset_in_file(source) as usize;
            let mut path_view = node_path::View::new(&mut mmap[offset..(offset + ENTRY_SIZE)]);
            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), source, 1, MAX_DEPTH);
            for step in dfs {
                let step = step?;
                let target = step.node;

                // Store directly outgoing edges in our inverse list
                if step.distance == 1 {
                    let edge = Edge { source, target };
                    self.inverse_edges.insert(edge.inverse(), true)?;

                    // Copy all annotations for this edge
                    for a in orig.get_anno_storage().get_annotations_for_item(&edge)? {
                        self.annos.insert(edge.clone(), a)?;
                    }
                }

                // Set the new length
                path_view.length_mut().write(step.distance.try_into()?);
                // The distance starts at 1, but we do not repeat the source
                // node in the path
                let offset = offset_in_path(step.distance - 1);
                // Set the node ID at the given position
                let target_node_id_bytes = target.to_le_bytes();
                path_view.nodes_mut()[offset..(offset + 8)]
                    .copy_from_slice(&target_node_id_bytes[..]);
            }
        }

        mmap.flush()?;
        // Re-map file read-only

        self.paths = unsafe { Mmap::map(&file)? };
        self.paths_file_size = file_capacity;
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
        // Open the new paths file
        let paths_file = location.join("paths.bin");
        let paths = File::open(paths_file)?;
        let paths_file_size = paths.metadata()?.len();
        let paths = unsafe { Mmap::map(&paths)? };

        // Load the inverse edges map
        let inverse_edges = DiskMap::new(
            Some(&location.join("inverse_edges.bin")),
            EvictionStrategy::default(),
            DEFAULT_BLOCK_CACHE_CAPACITY,
            BtreeConfig::default()
                .fixed_key_size(std::mem::size_of::<NodeID>() * 2)
                .fixed_value_size(2),
        )?;

        // Load annotation storage
        let annos = AnnoStorageImpl::new(Some(
            location.join(crate::annostorage::ondisk::SUBFOLDER_NAME),
        ))?;

        // Read stats
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::open(stats_path)?;
        let input = std::io::BufReader::new(f_stats);
        let stats = bincode::deserialize_from(input)?;

        Ok(Self {
            paths,
            paths_file_size,
            inverse_edges,
            annos,
            stats,
            location: Some(location.to_path_buf()),
        })
    }

    fn save_to(&self, location: &std::path::Path) -> Result<()> {
        // Make sure the output location exists before trying to normalize the paths
        std::fs::create_dir_all(location)?;
        // Normalize all paths to check if they are the same
        let new_location = location.normalize()?;
        if let Some(old_location) = &self.location {
            let old_location = old_location.normalize()?;
            if new_location == old_location {
                // This is an immutable graph storage so there can't be any
                // changes to write to the existing location we already use.
                return Ok(());
            }
        }
        // Copy the current paths file to the new location
        let new_paths_file = new_location.join("paths.bin");
        let mut new_paths = File::create(new_paths_file)?;
        let mut old_reader = BufReader::new(&self.paths[..]);
        std::io::copy(&mut old_reader, &mut new_paths)?;

        // Copy the inverse edges map to the new location
        self.inverse_edges
            .write_to(&location.join("inverse_edges.bin"))?;

        // Save edge annotations
        self.annos.save_annotations_to(location)?;

        // Write stats with bincode
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::create(stats_path)?;
        let mut writer = std::io::BufWriter::new(f_stats);
        bincode::serialize_into(&mut writer, &self.stats)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
