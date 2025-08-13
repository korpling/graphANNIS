//! Legacy structures of graph storages. Old versions of graph storages need to
//! be kept for compatibility reasons, but are not further developed. If
//! possible, only the legacy data structure is kept, the graph storage is
//! converted into a newer version and there is no specific implementation for
//! the old data structure.

use std::collections::HashMap;

use rustc_hash::FxHashMap;

use crate::{
    annostorage::inmemory::AnnoStorageImpl,
    types::{Edge, NodeID, NumValue},
};

use super::{
    GraphStatistic,
    linear::RelativePosition,
    prepost::{OrderVecEntry, PrePost},
};

/// Some general statistical numbers specific to a graph component
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct GraphStatisticV1 {
    /// True if the component contains any cycle.
    pub cyclic: bool,

    /// True if the component consists of [rooted trees](https://en.wikipedia.org/wiki/Tree_(graph_theory)).
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

impl From<GraphStatisticV1> for GraphStatistic {
    fn from(value: GraphStatisticV1) -> Self {
        let root_nodes = if value.nodes > 0 { 1 } else { 0 };
        Self {
            cyclic: value.cyclic,
            rooted_tree: value.rooted_tree,
            nodes: value.nodes,
            root_nodes,
            avg_fan_out: value.avg_fan_out,
            fan_out_99_percentile: value.fan_out_99_percentile,
            inverse_fan_out_99_percentile: value.inverse_fan_out_99_percentile,
            max_fan_out: value.max_fan_out,
            max_depth: value.max_depth,
            dfs_visit_ratio: value.dfs_visit_ratio,
        }
    }
}

/// An adjacency list based storage that uses the [`GraphStatisticV1`]
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct AdjacencyListStorageV1 {
    pub(crate) edges: HashMap<NodeID, Vec<NodeID>>,
    pub(crate) inverse_edges: HashMap<NodeID, Vec<NodeID>>,
    pub(crate) annos: AnnoStorageImpl<Edge>,
    pub(crate) stats: Option<GraphStatisticV1>,
}

/// An adjacency list based storage that uses the [`GraphStatisticV1`] and is
/// optimized for graphs where almost all nodes have an outgoing edge.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DenseAdjacencyListStorageV1 {
    pub(crate) edges: Vec<Option<NodeID>>,
    pub(crate) inverse_edges: HashMap<NodeID, Vec<NodeID>>,
    pub(crate) annos: AnnoStorageImpl<Edge>,
    pub(crate) stats: Option<GraphStatisticV1>,
}

/// A graph storage for linar graphs that uses the [`GraphStatisticV1`].
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LinearGraphStorageV1<PosT: NumValue> {
    pub(crate) node_to_pos: HashMap<NodeID, RelativePosition<PosT>>,
    pub(crate) node_chains: HashMap<NodeID, Vec<NodeID>>,
    pub(crate) annos: AnnoStorageImpl<Edge>,
    pub(crate) stats: Option<GraphStatisticV1>,
}

/// A graph storage for trees that uses the [`GraphStatisticV1`] and indexes graphs using the pre/post order.
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct PrePostOrderStorageV1<OrderT: NumValue, LevelT: NumValue> {
    pub(crate) node_to_order: FxHashMap<NodeID, Vec<PrePost<OrderT, LevelT>>>,
    pub(crate) order_to_node: Vec<OrderVecEntry<OrderT, LevelT>>,
    pub(crate) annos: AnnoStorageImpl<Edge>,
    pub(crate) stats: Option<GraphStatisticV1>,
}
