use {NodeID};
use std::collections::BTreeMap;

pub struct Edge {
    pub source_id : NodeID,
    pub target_id : NodeID,
    pub component_type : String,
    pub component_layer : String,
    pub component_name : String,
    ///  Maps a fully qualified label name (seperated by "::") to a label value
    pub labels : BTreeMap<String, String>,
}

pub struct Node {
    pub id : NodeID,
    /// Maps a fully qualified label name (seperated by "::") to a label value
    pub labels : BTreeMap<String, String>,
    pub outgoing_edges : Vec<Edge>,
}