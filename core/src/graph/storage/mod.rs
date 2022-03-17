pub mod adjacencylist;
pub mod dense_adjacency;
pub mod disk_adjacency;
pub mod linear;
pub mod prepost;
pub mod registry;
pub mod union;

use crate::malloc_size_of::MallocSizeOf;
use crate::{
    annostorage::AnnotationStorage,
    errors::Result,
    types::{AnnoKey, Annotation, Edge, NodeID},
};
use serde::{Deserialize, Serialize};
use std::{self, path::Path};

/// Some general statistical numbers specific to a graph component
#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct GraphStatistic {
    /// True if the component contains any cycle.
    pub cyclic: bool,

    /// True if the component consists of a [rooted trees](https://en.wikipedia.org/wiki/Tree_(graph_theory)).
    pub rooted_tree: bool,

    /// Number of nodes in this graph storage (both source and target nodes).
    pub nodes: usize,

    /// Average fan out.
    pub avg_fan_out: f64,
    /// Max fan-out of 99% of the data.
    pub fan_out_99_percentile: usize,

    /// Max inverse fan-out of 99% of the data.
    pub inverse_fan_out_99_percentile: usize,

    /// Maximal number of children of a node.
    pub max_fan_out: usize,
    /// Maximum length from a root node to a terminal node.
    pub max_depth: usize,

    /// Only valid for acyclic graphs: the average number of times a DFS will visit each node.
    pub dfs_visit_ratio: f64,
}

impl std::fmt::Display for GraphStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "nodes={}, avg_fan_out={:.2}, max_fan_out={}, max_depth={}",
            self.nodes, self.avg_fan_out, self.max_fan_out, self.max_depth
        )?;
        if self.cyclic {
            write!(f, ", cyclic")?;
        }
        if self.rooted_tree {
            write!(f, ", tree")?;
        }
        Ok(())
    }
}

/// Basic trait for accessing edges of a graph for a specific component.
pub trait EdgeContainer: Sync + Send + MallocSizeOf {
    /// Get all outgoing edges for a given `node`.
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a>;

    /// Return true of the given node has any outgoing edges.
    fn has_outgoing_edges(&self, node: NodeID) -> Result<bool> {
        if let Some(outgoing) = self.get_outgoing_edges(node).next() {
            outgoing?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all incoming edges for a given `node`.
    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a>;

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        None
    }

    /// Provides an iterator over all nodes of this edge container that are the source of an edge
    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = NodeID> + 'a>;
}

/// A graph storage is the representation of an edge component of a graph with specific structures.
/// These specific structures are exploited to efficiently implement reachability queries.
pub trait GraphStorage: EdgeContainer {
    /// Find all nodes reachable from a given start node inside the component.
    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a>;

    /// Find all nodes reachable from a given start node inside the component, when the directed edges are inversed.
    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a>;

    /// Compute the distance (shortest path length) of two nodes inside this component.
    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>>;

    /// Check if two nodes are connected with any path in this component given a minimum (`min_distance`) and maximum (`max_distance`) path length.
    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> bool;

    /// Get the annotation storage for the edges of this graph storage.
    fn get_anno_storage(&self) -> &dyn AnnotationStorage<Edge>;

    /// Copy the content of another component.
    /// This removes the existing content of this graph storage.
    fn copy(
        &mut self,
        node_annos: &dyn AnnotationStorage<NodeID>,
        orig: &dyn GraphStorage,
    ) -> Result<()>;

    /// Upcast this graph storage to the [EdgeContainer](trait.EdgeContainer.html) trait.
    fn as_edgecontainer(&self) -> &dyn EdgeContainer;

    /// Try to downcast this graph storage to a [WriteableGraphStorage](trait.WriteableGraphStorage.html) trait.
    /// Returns `None` if this graph storage is not writable.
    fn as_writeable(&mut self) -> Option<&mut dyn WriteableGraphStorage> {
        None
    }

    /// If true, finding the inverse connected nodes via [find_connected_inverse(...)](#tymethod.find_connected_inverse) has the same cost as the non-inverse case.
    fn inverse_has_same_cost(&self) -> bool {
        false
    }

    /// Return an identifier for this graph storage which is used to distinguish the different graph storages when (de-) serialized.
    fn serialization_id(&self) -> String;

    /// Load the graph storage from a `location` on the disk. This location is a directory, which can contain files specific to this graph storage.
    fn load_from(location: &Path) -> Result<Self>
    where
        Self: std::marker::Sized;

    /// Save the graph storage a `location` on the disk. This location must point to an existing directory.
    fn save_to(&self, location: &Path) -> Result<()>;
}

pub fn default_serialize_gs<GS>(gs: &GS, location: &Path) -> Result<()>
where
    GS: Serialize,
{
    let data_path = location.join("component.bin");
    let f_data = std::fs::File::create(&data_path)?;
    let mut writer = std::io::BufWriter::new(f_data);
    bincode::serialize_into(&mut writer, gs)?;
    Ok(())
}

pub fn default_deserialize_gs<GS>(location: &Path) -> Result<GS>
where
    for<'de> GS: std::marker::Sized + Deserialize<'de>,
{
    let data_path = location.join("component.bin");
    let f_data = std::fs::File::open(data_path)?;
    let input = std::io::BufReader::new(f_data);

    let result = bincode::deserialize_from(input)?;

    Ok(result)
}

/// Trait for accessing graph storages which can be written to.
pub trait WriteableGraphStorage: GraphStorage {
    /// Add an edge to this graph storage.
    fn add_edge(&mut self, edge: Edge) -> Result<()>;

    /// Add an annotation to an edge in this graph storage.
    /// The edge has to exist.
    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation) -> Result<()>;

    /// Delete an existing edge.
    fn delete_edge(&mut self, edge: &Edge) -> Result<()>;

    /// Delete the annotation (defined by the qualified annotation name in `anno_key`) for an `edge`.
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey) -> Result<()>;

    /// Delete a node from this graph storage.
    /// This deletes both edges edges where the node is the source or the target node.
    fn delete_node(&mut self, node: NodeID) -> Result<()>;

    /// Re-calculate the [statistics](struct.GraphStatistic.html) of this graph storage.
    fn calculate_statistics(&mut self) -> Result<()>;

    /// Remove all edges from this grap storage.
    fn clear(&mut self) -> Result<()>;
}
