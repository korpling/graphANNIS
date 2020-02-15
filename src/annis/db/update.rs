//! Types used to describe updates on graphs.

use crate::annis::errors::*;
use crate::annis::util::disk_collections::DiskMap;
use std::convert::TryFrom;

/// Describes a single update on the graph.
#[derive(Serialize, Deserialize, Clone, Debug, MallocSizeOf)]
pub enum UpdateEvent {
    /// Add a node with a name and type.
    AddNode {
        node_name: String,
        node_type: String,
    },
    /// Delete a node given by the name.
    DeleteNode { node_name: String },
    /// Add a label to a the node given by the name.
    AddNodeLabel {
        node_name: String,
        anno_ns: String,
        anno_name: String,
        anno_value: String,
    },
    /// Delete a label of an node given by the name of the node and the qualified label name.
    DeleteNodeLabel {
        node_name: String,
        anno_ns: String,
        anno_name: String,
    },
    /// Add an edge between two nodes given by their name.
    AddEdge {
        source_node: String,
        target_node: String,
        layer: String,
        component_type: String,
        component_name: String,
    },
    /// Delete an existing edge between two nodes given by their name.
    DeleteEdge {
        source_node: String,
        target_node: String,
        layer: String,
        component_type: String,
        component_name: String,
    },
    /// Add a label to an edge between two nodes.
    AddEdgeLabel {
        source_node: String,
        target_node: String,
        layer: String,
        component_type: String,
        component_name: String,
        anno_ns: String,
        anno_name: String,
        anno_value: String,
    },
    /// Delete a label from an edge between two nodes.
    DeleteEdgeLabel {
        source_node: String,
        target_node: String,
        layer: String,
        component_type: String,
        component_name: String,
        anno_ns: String,
        anno_name: String,
    },
}

/// A list of changes to apply to an graph.
#[derive(Default)]
#[repr(C)]
pub struct GraphUpdate {
    diffs: DiskMap<u64, UpdateEvent>,
    last_change_id: u64,
}

impl GraphUpdate {
    /// Create a new empty list of updates.
    pub fn new() -> GraphUpdate {
        GraphUpdate {
            diffs: DiskMap::default(),
            last_change_id: 0,
        }
    }

    /// Add the given event to the update list.
    pub fn add_event(&mut self, event: UpdateEvent) -> Result<()> {
        self.last_change_id += 1;
        self.diffs.insert(self.last_change_id, event)?;
        Ok(())
    }

    /// Get all changes
    pub fn iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = (u64, UpdateEvent)> + 'a>> {
        let it = self.diffs.iter()?;
        Ok(Box::new(it))
    }

    /// Returns `true` if the update list is empty.
    pub fn is_empty(&self) -> Result<bool> {
        self.diffs.is_empty()
    }
}

impl TryFrom<DiskMap<u64, UpdateEvent>> for GraphUpdate {
    type Error = crate::annis::errors::Error;

    fn try_from(diffs: DiskMap<u64, UpdateEvent>) -> Result<GraphUpdate> {
        let last_change_id = diffs.iter()?.map(|(id, _)| id).max();
        Ok(GraphUpdate {
            last_change_id: last_change_id.unwrap_or(0),
            diffs,
        })
    }
}
