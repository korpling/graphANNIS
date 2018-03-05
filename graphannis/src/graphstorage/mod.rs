use std;
use std::any::Any;
use {AnnoKey, Annotation, Edge, NodeID};
use annostorage::AnnoStorage;
use stringstorage::StringStorage;
use graphdb::GraphDB;

/// Some general statistical numbers specific to a graph component
#[derive(Serialize, Deserialize, Clone, HeapSizeOf)]
pub struct GraphStatistic
{
    pub cyclic : bool,
    pub rooted_tree : bool,

    /// number of nodes
    pub nodes: usize,

    /// Average fan out
    pub avg_fan_out : f64,
    /// Max fan-out of 99% of the data
    pub fan_out_99_percentile : usize,
    /// maximal number of children of a node
    pub max_fan_out : usize,
    /// maximum length from a root node to a terminal node
    pub max_depth : usize,

    /// only for acyclic graphs: the average number of times a DFS will visit each node
    pub dfs_visit_ratio : f64,
}

pub trait GraphStorage : Sync + Send {

    fn get_outgoing_edges<'a>(&'a self, source: &NodeID) -> Box<Iterator<Item = NodeID> + 'a>;

    fn get_edge_annos(&self, edge : &Edge) -> Vec<Annotation>;
    
    fn find_connected<'a>(
        &'a self,
        source: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a>;
    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize>;
    fn is_connected(&self, source: &NodeID, target: &NodeID, min_distance: usize, max_distance: usize) -> bool;

    fn copy(&mut self, db : &GraphDB, orig : &GraphStorage);

    fn get_anno_storage(&self) -> &AnnoStorage<Edge>;

    fn as_any(&self) -> &Any;

    fn as_writeable(&mut self) -> Option<&mut WriteableGraphStorage> {None}
    
    fn get_statistics(&self) -> Option<&GraphStatistic> {None}

    fn calculate_statistics(&mut self, _string_storage : &StringStorage) {}

}

pub trait WriteableGraphStorage:  GraphStorage {
    fn add_edge(&mut self, edge: Edge);
    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation);

    fn delete_edge(&mut self, edge: &Edge);
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey);
    fn delete_node(&mut self, node: &NodeID);
}

pub mod adjacencylist;
pub mod prepost;
pub mod linear;
pub mod registry;
