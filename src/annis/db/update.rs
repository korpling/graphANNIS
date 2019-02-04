//! Types used to describe updates on graphs.

/// Describes a single update on the graph.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[repr(C)]
pub struct GraphUpdate {
    diffs: Vec<(u64, UpdateEvent)>,
    last_consistent_change_id: u64,
}

impl GraphUpdate {
    /// Create a new empty list of updates.
    pub fn new() -> GraphUpdate {
        GraphUpdate {
            diffs: Vec::default(),
            last_consistent_change_id: 0,
        }
    }

    /// Add the given event to the update list.
    pub fn add_event(&mut self, event: UpdateEvent) {
        let change_id = self.last_consistent_change_id + (self.diffs.len() as u64) + 1;
        self.diffs.push((change_id, event));
    }

    /// Check if the last item of the last has been marked as consistent.
    pub fn is_consistent(&self) -> bool {
        if let Some(last) = self.diffs.last() {
            self.last_consistent_change_id == last.0
        } else {
            true
        }
    }

    /// Get the ID of the last change that has been marked as consistent.
    pub fn get_last_consistent_change_id(&self) -> u64 {
        self.last_consistent_change_id
    }

    /// Mark the current state as consistent.
    pub fn finish(&mut self) {
        if let Some(last) = self.diffs.last() {
            self.last_consistent_change_id = last.0;
        }
    }

    /// Get all consistent changes
    pub fn consistent_changes<'a>(&'a self) -> Box<Iterator<Item = (u64, UpdateEvent)> + 'a> {
        let last_consistent_change_id = self.last_consistent_change_id;
        let it = self.diffs.iter().filter_map(move |d| {
            if d.0 <= last_consistent_change_id {
                Some((d.0, d.1.clone()))
            } else {
                None
            }
        });

        Box::new(it)
    }

    /// Return the number of updates in the list.
    pub fn len(&self) -> usize {
        self.diffs.len()
    }

    /// Returns `true` if the update list is empty.
    pub fn is_empty(&self) -> bool {
        self.diffs.is_empty()
    }
}
