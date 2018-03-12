use {NodeID};
use std::collections::BTreeMap;

pub struct Edge<T> {
    pub source_id : NodeID,
    pub target_id : NodeID,
    pub component_type : T,
    pub component_layer : T,
    pub component_name : T,
    ///  Maps a fully qualified label name (seperated by "::") to a label value
    pub labels : BTreeMap<T, T>,
}

pub struct Node<T> {
    pub id : NodeID,
    /// Maps a fully qualified label name (seperated by "::") to a label value
    pub labels : BTreeMap<T, T>,
    pub outgoing_edges : Vec<Edge<T>>,
}