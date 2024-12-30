//! Types used to describe updates on graphs.

use std::convert::TryInto;
use std::fs::File;
use std::sync::Mutex;

use crate::errors::{GraphAnnisCoreError, Result};
use crate::serializer::KeySerializer;
use bincode::Options;
use serde::de::Error as DeserializeError;
use serde::de::{MapAccess, Visitor};
use serde::ser::{Error as SerializeError, SerializeMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};
use tempfile::NamedTempFile;

/// Describes a single update on the graph.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

enum ChangeSet {
    InProgress {
        table_builder: Box<TableBuilder<File>>,
        outfile: NamedTempFile,
    },
    Finished {
        table: Table,
    },
}

/// A list of changes to apply to an graph.
pub struct GraphUpdate {
    changesets: Mutex<Vec<ChangeSet>>,
    event_counter: u64,
    serialization: bincode::config::DefaultOptions,
}

impl Default for GraphUpdate {
    fn default() -> Self {
        GraphUpdate::new()
    }
}

impl GraphUpdate {
    /// Create a new empty list of updates.
    pub fn new() -> GraphUpdate {
        GraphUpdate {
            event_counter: 0,
            changesets: Mutex::new(Vec::new()),
            serialization: bincode::options(),
        }
    }

    /// Add the given event to the update list.
    pub fn add_event(&mut self, event: UpdateEvent) -> Result<()> {
        let new_event_counter = self.event_counter + 1;
        let key = new_event_counter.create_key();
        let value = self.serialization.serialize(&event)?;
        let mut changeset = self.changesets.lock()?;
        if let ChangeSet::InProgress { table_builder, .. } =
            current_inprogress_changeset(&mut changeset)?
        {
            table_builder.add(&key, &value)?;
            self.event_counter = new_event_counter;
        }
        Ok(())
    }

    /// Get all changes
    pub fn iter(&self) -> Result<GraphUpdateIterator> {
        let it = GraphUpdateIterator::new(self)?;
        Ok(it)
    }

    /// Returns `true` if the update list is empty.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.event_counter == 0)
    }

    // Returns the number of updates.
    pub fn len(&self) -> Result<usize> {
        let result = self.event_counter.try_into()?;
        Ok(result)
    }
}

fn finish_all_changesets(changesets: &mut Vec<ChangeSet>) -> Result<()> {
    // Remove all changesets from the vector and finish them
    let finished: Result<Vec<ChangeSet>> = changesets
        .drain(..)
        .map(|c| match c {
            ChangeSet::InProgress {
                table_builder,
                outfile,
            } => {
                table_builder.finish()?;
                // Re-open as table
                let file = outfile.reopen()?;
                let size = file.metadata()?.len();
                let table = Table::new(sstable::Options::default(), Box::new(file), size as usize)?;
                Ok(ChangeSet::Finished { table })
            }
            ChangeSet::Finished { table } => Ok(ChangeSet::Finished { table }),
        })
        .collect();
    // Re-add the finished changesets
    changesets.extend(finished?);

    Ok(())
}

fn current_inprogress_changeset(changesets: &mut Vec<ChangeSet>) -> Result<&mut ChangeSet> {
    let needs_new_changeset = if let Some(c) = changesets.last_mut() {
        match c {
            ChangeSet::InProgress { .. } => false,
            ChangeSet::Finished { .. } => true,
        }
    } else {
        true
    };

    if needs_new_changeset {
        // Create a new changeset
        let outfile = NamedTempFile::new()?;
        let table_builder = TableBuilder::new(sstable::Options::default(), outfile.reopen()?);
        let c = ChangeSet::InProgress {
            table_builder: Box::new(table_builder),
            outfile,
        };
        changesets.push(c);
    }

    // Get the last changeset, which must be in the InProgress state
    changesets
        .last_mut()
        .ok_or(GraphAnnisCoreError::GraphUpdatePersistanceFileMissing)
}

pub struct GraphUpdateIterator {
    iterators: Vec<TableIterator>,
    size_hint: u64,
    serialization: bincode::config::DefaultOptions,
}

impl GraphUpdateIterator {
    fn new(g: &GraphUpdate) -> Result<GraphUpdateIterator> {
        let mut changesets = g.changesets.lock()?;

        finish_all_changesets(&mut changesets)?;

        let iterators: Vec<_> = changesets
            .iter()
            .filter_map(|c| match c {
                ChangeSet::InProgress { .. } => None,
                ChangeSet::Finished { table } => {
                    let mut it = table.iter();
                    it.seek_to_first();
                    Some(it)
                }
            })
            .collect();
        Ok(GraphUpdateIterator {
            size_hint: g.event_counter,
            iterators,
            serialization: g.serialization,
        })
    }
}

impl std::iter::Iterator for GraphUpdateIterator {
    type Item = Result<(u64, UpdateEvent)>;

    fn next(&mut self) -> Option<Self::Item> {
        // Remove all empty table iterators.
        self.iterators.retain(|it| it.valid());

        if let Some(it) = self.iterators.first_mut() {
            // Get the current values
            if let Some((key, value)) = sstable::current_key_val(it) {
                // Create the actual types
                let id = match u64::parse_key(&key) {
                    Ok(id) => id,
                    Err(e) => return Some(Err(e.into())),
                };
                let event: UpdateEvent = match self.serialization.deserialize(&value) {
                    Ok(event) => event,
                    Err(e) => return Some(Err(e.into())),
                };

                // Advance for next iteration
                it.advance();
                return Some(Ok((id, event)));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Ok(s) = self.size_hint.try_into() {
            (s, Some(s))
        } else {
            (0, None)
        }
    }
}

impl Serialize for GraphUpdate {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let iter = self.iter().map_err(S::Error::custom)?;
        let number_of_updates = self.len().map_err(S::Error::custom)?;
        let mut map_serializer = serializer.serialize_map(Some(number_of_updates))?;

        for entry in iter {
            let (key, value) = entry.map_err(S::Error::custom)?;
            map_serializer
                .serialize_entry(&key, &value)
                .map_err(S::Error::custom)?;
        }

        map_serializer.end()
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
        let serialization = bincode::options();
        let outfile = NamedTempFile::new().map_err(M::Error::custom)?;
        let mut table_builder = TableBuilder::new(
            sstable::Options::default(),
            outfile.reopen().map_err(M::Error::custom)?,
        );

        let mut event_counter = 0;

        while let Some((id, event)) = access.next_entry::<u64, UpdateEvent>()? {
            event_counter = id;
            let key = id.create_key();
            let value = serialization.serialize(&event).map_err(M::Error::custom)?;
            table_builder.add(&key, &value).map_err(M::Error::custom)?
        }

        let c = ChangeSet::InProgress {
            outfile,
            table_builder: Box::new(table_builder),
        };
        let mut changesets = vec![c];
        finish_all_changesets(&mut changesets).map_err(M::Error::custom)?;
        let g = GraphUpdate {
            changesets: Mutex::new(changesets),
            event_counter,
            serialization,
        };

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

#[cfg(test)]
mod tests {

    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn serialize_deserialize_bincode() {
        let example_updates = vec![
            UpdateEvent::AddNode {
                node_name: "parent".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddNode {
                node_name: "child".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddEdge {
                source_node: "child".into(),
                target_node: "parent".into(),
                layer: "annis".into(),
                component_type: "PartOf".into(),
                component_name: "".into(),
            },
        ];

        let mut updates = GraphUpdate::new();
        for e in example_updates.iter() {
            updates.add_event(e.clone()).unwrap();
        }

        let seralized_bytes: Vec<u8> = bincode::serialize(&updates).unwrap();
        let deseralized_update: GraphUpdate = bincode::deserialize(&seralized_bytes).unwrap();

        assert_eq!(3, deseralized_update.len().unwrap());
        let deseralized_events: Vec<UpdateEvent> = deseralized_update
            .iter()
            .unwrap()
            .map(|e| e.unwrap().1)
            .collect();
        assert_eq!(example_updates, deseralized_events);
    }

    #[test]
    fn serialize_deserialize_bincode_empty() {
        let example_updates: Vec<UpdateEvent> = Vec::new();

        let mut updates = GraphUpdate::new();
        for e in example_updates.iter() {
            updates.add_event(e.clone()).unwrap();
        }

        let seralized_bytes: Vec<u8> = bincode::serialize(&updates).unwrap();
        let deseralized_update: GraphUpdate = bincode::deserialize(&seralized_bytes).unwrap();

        assert_eq!(0, deseralized_update.len().unwrap());
        assert_eq!(true, deseralized_update.is_empty().unwrap());
    }

    #[test]
    fn serialize_json() {
        let example_updates = vec![
            UpdateEvent::AddNode {
                node_name: "parent".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddNode {
                node_name: "child".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddEdge {
                source_node: "child".into(),
                target_node: "parent".into(),
                layer: "annis".into(),
                component_type: "PartOf".into(),
                component_name: "".into(),
            },
        ];

        let mut updates = GraphUpdate::new();
        for e in example_updates.iter() {
            updates.add_event(e.clone()).unwrap();
        }

        let seralized_string = serde_json::to_string_pretty(&updates).unwrap();
        assert_snapshot!(seralized_string);
    }

    #[test]
    fn serialize_deserialize_json() {
        let example_updates = vec![
            UpdateEvent::AddNode {
                node_name: "parent".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddNode {
                node_name: "child".into(),
                node_type: "corpus".into(),
            },
            UpdateEvent::AddEdge {
                source_node: "child".into(),
                target_node: "parent".into(),
                layer: "annis".into(),
                component_type: "PartOf".into(),
                component_name: "".into(),
            },
        ];

        let mut updates = GraphUpdate::new();
        for e in example_updates.iter() {
            updates.add_event(e.clone()).unwrap();
        }

        let seralized_string = serde_json::to_string_pretty(&updates).unwrap();
        let deseralized_update: GraphUpdate = serde_json::from_str(&seralized_string).unwrap();

        assert_eq!(3, deseralized_update.len().unwrap());
        let deseralized_events: Vec<UpdateEvent> = deseralized_update
            .iter()
            .unwrap()
            .map(|e| e.unwrap().1)
            .collect();
        assert_eq!(example_updates, deseralized_events);
    }
}
