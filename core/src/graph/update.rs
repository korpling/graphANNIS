//! Types used to describe updates on graphs.

use crate::util::disk_collections::EvictionStrategy;
use crate::{errors::Result, util::disk_collections::DiskMap};
use serde::de::Error as DeserializeError;
use serde::de::{MapAccess, Visitor};
use serde::ser::Error as SerializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
pub struct GraphUpdate {
    diffs: DiskMap<u64, UpdateEvent>,
    event_counter: u64,
}

impl GraphUpdate {
    /// Create a new empty list of updates.
    pub fn new() -> GraphUpdate {
        GraphUpdate {
            diffs: DiskMap::new_temporary(EvictionStrategy::default(), None, 0),
            event_counter: 0,
        }
    }

    /// Add the given event to the update list.
    pub fn add_event(&mut self, event: UpdateEvent) -> Result<()> {
        self.event_counter += 1;
        self.diffs.insert(self.event_counter, event)?;
        Ok(())
    }

    /// Get all changes
    pub fn iter(&self) -> Result<GraphUpdateIterator> {
        let it = GraphUpdateIterator::new(self)?;
        Ok(it)
    }

    /// Returns `true` if the update list is empty.
    pub fn is_empty(&self) -> Result<bool> {
        self.diffs.try_is_empty()
    }
}

pub struct GraphUpdateIterator<'a> {
    diff_iter: Box<dyn Iterator<Item = (u64, UpdateEvent)> + 'a>,
    length: u64,
}

impl<'a> GraphUpdateIterator<'a> {
    fn new(g: &'a GraphUpdate) -> Result<GraphUpdateIterator<'a>> {
        Ok(GraphUpdateIterator {
            length: g.event_counter,
            diff_iter: g.diffs.try_iter()?,
        })
    }
}

impl<'a> std::iter::Iterator for GraphUpdateIterator<'a> {
    type Item = (u64, UpdateEvent);

    fn next(&mut self) -> Option<(u64, UpdateEvent)> {
        self.diff_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.length as usize, Some(self.length as usize))
    }
}

impl Serialize for GraphUpdate {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let it = self.iter().map_err(S::Error::custom)?;
        serializer.collect_map(it)
    }
}

struct GraphUpdateVisitor {}

impl<'de> Visitor<'de> for GraphUpdateVisitor {
    type Value = GraphUpdate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of graph updates")
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut g = GraphUpdate::default();

        while let Some((key, value)) = access.next_entry().map_err(M::Error::custom)? {
            g.diffs.insert(key, value).map_err(M::Error::custom)?;
            g.event_counter = key;
        }

        Ok(g)
    }
}

impl<'de> Deserialize<'de> for GraphUpdate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GraphUpdateVisitor {})
    }
}
