use std;
use std::any::Any;
use {AnnoKey, Annotation, Edge, NodeID};

/// Some general statistical numbers specific to a graph component
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphStatistic
{
    cyclic : bool,
    rooted_tree : bool,

    /// number of nodes
    nodes: usize,

    /// Average fan out
    avg_fan_out : f64,
    /// Max fan-out of 99% of the data
    fan_out_99_percentile : usize,
    /// maximal number of children of a node
    max_fan_out : usize,
    /// maximum length from a root node to a terminal node
    max_depth : usize,

    /// only for acyclic graphs: the average number of times a DFS will visit each node
    dfs_visit_ratio : f64,
}

pub trait GraphStorage  {

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

    fn copy(&mut self, orig : &GraphStorage);

    fn as_writeable(&mut self) -> Option<&mut WriteableGraphStorage> {None}
    fn as_any(&self) -> &Any;

}

pub trait WriteableGraphStorage:  GraphStorage {
    fn add_edge(&mut self, edge: Edge);
    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation);

    fn delete_edge(&mut self, edge: &Edge);
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey);
    fn delete_node(&mut self, node: &NodeID);

    fn calculate_statistics(&mut self);
}

pub mod adjacencylist;
pub mod registry;
