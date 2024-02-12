use std::convert::TryInto;

use crate::{dfs::CycleSafeDFS, errors::Result, types::NodeID};

use super::{EdgeContainer, GraphStatistic, GraphStorage};
use binary_layout::prelude::*;

const MAX_DEPTH: usize = 10;

binary_layout!(node_path, LittleEndian, {
    length: u8,
    nodes: [u8; MAX_DEPTH*8],
});

/// A [GraphStorage] that stores a single path for each node on disk.
pub struct DiskPathStorage {
    file: std::fs::File,
    stats: Option<GraphStatistic>,
}

impl EdgeContainer for DiskPathStorage {
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        todo!()
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        todo!()
    }

    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        todo!()
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
            let mut output_bytes: Vec<u8> = Vec::with_capacity(node_path::SIZE.unwrap_or_default());
            let mut path_view = node_path::View::new(&mut output_bytes);
            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), source, 1, MAX_DEPTH);
            for step in dfs {
                let step = step?;
                // Set the new length
                path_view.length_mut().write(step.distance.try_into()?);
                // The distance starts at 1, but we do not repeat the source
                // node in the path
                let offset = (step.distance - 1) * 8;
                // Set the node ID at the given position
                let node_id_bytes = step.node.to_le_bytes();
                path_view.nodes_mut()[offset..(offset + 8)].copy_from_slice(&node_id_bytes);
            }
        }
        self.file = file;
        self.stats = orig.get_statistics().cloned();
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
        todo!()
    }

    fn save_to(&self, location: &std::path::Path) -> Result<()> {
        todo!()
    }
}
