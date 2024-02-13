use itertools::Itertools;
use normpath::PathExt;
use std::{convert::TryInto, fs::File, io::BufReader, os::unix::fs::FileExt, path::PathBuf};

use crate::{
    annostorage::{ondisk::AnnoStorageImpl, AnnotationStorage},
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID},
};

use super::{EdgeContainer, GraphStatistic, GraphStorage};
use binary_layout::prelude::*;

const MAX_DEPTH: usize = 15;
const ENTRY_SIZE: usize = (MAX_DEPTH * 8) + 1;

binary_layout!(node_path, LittleEndian, {
    length: u8,
    nodes: [u8; MAX_DEPTH*8],
});

/// A [GraphStorage] that stores a single path for each node on disk.
pub struct DiskPathStorage {
    paths: std::fs::File,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
    location: Option<PathBuf>,
}

fn offset_in_file(n: NodeID) -> u64 {
    n * (node_path::SIZE.unwrap_or(1) as u64)
}

fn offset_in_path(path_idx: usize) -> usize {
    path_idx * 8
}

impl DiskPathStorage {
    fn get_outgoing_edge<'a>(&'a self, node: NodeID) -> Result<Option<NodeID>> {
        let mut buffer = [0; ENTRY_SIZE];
        self.paths
            .read_exact_at(&mut buffer, offset_in_file(node))?;
        let view = node_path::View::new(&buffer);
        if view.length().read() == 0 {
            // No outgoing edges
            Ok(None)
        } else {
            // Read the node ID at the first position
            let buffer: [u8; 8] = view.nodes()[offset_in_path(0)..offset_in_path(1)].try_into()?;
            Ok(Some(u64::from_le_bytes(buffer)))
        }
    }

    fn number_of_nodes(&self) -> Result<u64> {
        let file_size = self.paths.metadata()?.len();
        Ok(file_size / (ENTRY_SIZE as u64))
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
        todo!()
    }

    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        match self.number_of_nodes() {
            Ok(nr_nodes) => {
                // ignore node entries with empty path in result
                let it = (0..nr_nodes)
                    .map(move |n| {
                        let mut buffer = [0; ENTRY_SIZE];

                        self.paths.read_exact_at(&mut buffer, offset_in_file(n))?;
                        let view = node_path::View::new(&buffer);

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
            Err(e) => Box::new(std::iter::once(Err(e.into()))),
        }
    }
}

impl GraphStorage for DiskPathStorage {
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
        todo!()
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
        todo!()
    }

    fn copy(
        &mut self,
        node_annos: &dyn crate::annostorage::NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        // Create a new file which is large enough to contain the paths for all nodes.
        let number_of_nodes = node_annos.get_largest_item()?.unwrap_or(0);
        let file_capacity = number_of_nodes * (node_path::SIZE.unwrap_or(1) as u64);
        let file = tempfile::tempfile()?;
        if file_capacity > 0 {
            file.set_len(file_capacity)?;
        }

        // Get the paths for all source nodes in the original graph storage
        for source in orig.source_nodes() {
            let source = source?;
            let mut output_bytes = [0; ENTRY_SIZE];
            let mut path_view = node_path::View::new(&mut output_bytes);
            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), source, 1, MAX_DEPTH);
            for step in dfs {
                let step = step?;
                // Set the new length
                path_view.length_mut().write(step.distance.try_into()?);
                // The distance starts at 1, but we do not repeat the source
                // node in the path
                let offset = offset_in_path(step.distance - 1);
                // Set the node ID at the given position
                let node_id_bytes = step.node.to_le_bytes();
                path_view.nodes_mut()[offset..(offset + 8)].copy_from_slice(&node_id_bytes);

                // Copy all annotations for this edge
                let e = Edge {
                    source,
                    target: step.node,
                };
                for a in orig.get_anno_storage().get_annotations_for_item(&e)? {
                    self.annos.insert(e.clone(), a)?;
                }
            }
            // Save the path at the node offset
            self.paths
                .write_all_at(&output_bytes, offset_in_file(source))?;
        }
        self.paths = file;
        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics()?;
        Ok(())
    }

    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }

    fn serialization_id(&self) -> String {
        todo!()
    }

    fn load_from(location: &std::path::Path) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        // Open the new paths file
        let paths_file = location.join("paths.bin");
        let paths = File::open(paths_file)?;

        // Create annotatio storage
        let annos = AnnoStorageImpl::new(Some(location.to_path_buf()))?;

        // Read stats
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::open(stats_path)?;
        let input = std::io::BufReader::new(f_stats);
        let stats = bincode::deserialize_from(input)?;

        Ok(Self {
            paths,
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
        let mut new_paths = File::open(&new_paths_file)?;
        let mut reader = BufReader::new(&self.paths);
        std::io::copy(&mut reader, &mut new_paths)?;

        self.annos.save_annotations_to(location)?;
        // Write stats with bincode
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::create(stats_path)?;
        let mut writer = std::io::BufWriter::new(f_stats);
        bincode::serialize_into(&mut writer, &self.stats)?;

        Ok(())
    }
}
