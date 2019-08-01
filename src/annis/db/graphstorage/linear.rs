use super::{GraphStatistic, GraphStorage};
use crate::annis::db::annostorage::inmemory::AnnoStorageImpl;
use crate::annis::db::graphstorage::EdgeContainer;
use crate::annis::db::AnnotationStorage;
use crate::annis::db::Graph;
use crate::annis::db::Match;
use crate::annis::dfs::{CycleSafeDFS, DFSStep};
use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, Edge, NodeID, NumValue};
use bincode;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std;
use std::clone::Clone;

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
struct RelativePosition<PosT> {
    pub root: NodeID,
    pub pos: PosT,
}

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct LinearGraphStorage<PosT: NumValue> {
    node_to_pos: FxHashMap<NodeID, RelativePosition<PosT>>,
    node_chains: FxHashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

impl<PosT> LinearGraphStorage<PosT>
where
    PosT: NumValue,
{
    pub fn new() -> LinearGraphStorage<PosT> {
        LinearGraphStorage {
            node_to_pos: FxHashMap::default(),
            node_chains: FxHashMap::default(),
            annos: AnnoStorageImpl::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) {
        self.node_to_pos.clear();
        self.node_chains.clear();
        self.annos.clear();
        self.stats = None;
    }
}

impl<PosT> Default for LinearGraphStorage<PosT>
where
    PosT: NumValue,
{
    fn default() -> Self {
        LinearGraphStorage::new()
    }
}

impl<PosT: 'static> EdgeContainer for LinearGraphStorage<PosT>
where
    PosT: NumValue,
{
    fn get_outgoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(pos) = self.node_to_pos.get(&node) {
            // find the next node in the chain
            if let Some(chain) = self.node_chains.get(&pos.root) {
                let next_pos = pos.pos.clone() + PosT::one();
                if let Some(next_pos) = next_pos.to_usize() {
                    if next_pos < chain.len() {
                        return Box::from(std::iter::once(chain[next_pos]));
                    }
                }
            }
        }
        Box::from(std::iter::empty())
    }

    fn get_ingoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(pos) = self.node_to_pos.get(&node) {
            // find the previous node in the chain
            if let Some(chain) = self.node_chains.get(&pos.root) {
                if let Some(pos) = pos.pos.to_usize() {
                    if let Some(previous_pos) = pos.checked_sub(1) {
                        return Box::from(std::iter::once(chain[previous_pos]));
                    }
                }
            }
        }
        Box::from(std::iter::empty())
    }

    fn source_nodes<'a>(&'a self) -> Box<Iterator<Item = NodeID> + 'a> {
        // use the node chains to find source nodes, but always skip the last element
        // because the last element is only a target node, not a source node
        let it = self
            .node_chains
            .iter()
            .flat_map(|(_root, chain)| chain.iter().rev().skip(1))
            .cloned();

        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl<PosT: 'static> GraphStorage for LinearGraphStorage<PosT>
where
    for<'de> PosT: NumValue + Deserialize<'de> + Serialize,
{
    fn get_anno_storage(&self) -> &AnnotationStorage<Edge> {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        format!("LinearO{}V1", std::mem::size_of::<PosT>() * 8)
    }

    fn serialize_gs(&self, writer: &mut std::io::Write) -> Result<()> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    fn deserialize_gs(input: &mut std::io::Read) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let mut result: LinearGraphStorage<PosT> = bincode::deserialize_from(input)?;
        result.annos.after_deserialization();
        Ok(result)
    }

    fn find_connected<'a>(
        &'a self,
        source: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(start_pos) = self.node_to_pos.get(&source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    if let Some(min_distance) = offset.checked_add(min_distance) {
                        if min_distance < chain.len() {
                            let max_distance = match max_distance {
                                std::ops::Bound::Unbounded => {
                                    return Box::new(chain[min_distance..].iter().cloned());
                                }
                                std::ops::Bound::Included(max_distance) => {
                                    offset + max_distance + 1
                                }
                                std::ops::Bound::Excluded(max_distance) => offset + max_distance,
                            };
                            // clip to chain length
                            let max_distance = std::cmp::min(chain.len(), max_distance);
                            if min_distance < max_distance {
                                return Box::new(chain[min_distance..max_distance].iter().cloned());
                            }
                        }
                    }
                }
            }
        }
        Box::new(std::iter::empty())
    }

    fn find_connected_inverse<'a>(
        &'a self,
        source: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(start_pos) = self.node_to_pos.get(&source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    let max_distance = match max_distance {
                        std::ops::Bound::Unbounded => 0,
                        std::ops::Bound::Included(max_distance) => {
                            offset.checked_sub(max_distance).unwrap_or(0)
                        }
                        std::ops::Bound::Excluded(max_distance) => {
                            offset.checked_sub(max_distance + 1).unwrap_or(0)
                        }
                    };

                    if let Some(min_distance) = offset.checked_sub(min_distance) {
                        if min_distance < chain.len() && max_distance <= min_distance {
                            // return all entries in the chain between min_distance..max_distance (inclusive)
                            return Box::new(chain[max_distance..=min_distance].iter().cloned());
                        } else if max_distance < chain.len() {
                            // return all entries in the chain between min_distance..max_distance
                            return Box::new(chain[max_distance..chain.len()].iter().cloned());
                        }
                    }
                }
            }
        }
        Box::new(std::iter::empty())
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Option<usize> {
        if source == target {
            return Some(0);
        }

        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(&source), self.node_to_pos.get(&target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    return Some(diff);
                }
            }
        }
        None
    }

    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> bool {
        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(&source), self.node_to_pos.get(&target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    match max_distance {
                        std::ops::Bound::Unbounded => {
                            return diff >= min_distance;
                        }
                        std::ops::Bound::Included(max_distance) => {
                            return diff >= min_distance && diff <= max_distance;
                        }
                        std::ops::Bound::Excluded(max_distance) => {
                            return diff >= min_distance && diff < max_distance;
                        }
                    }
                }
            }
        }

        false
    }

    fn copy(&mut self, db: &Graph, orig: &GraphStorage) -> Result<()> {
        self.clear();

        // find all roots of the component
        let mut roots: FxHashSet<NodeID> = FxHashSet::default();
        let node_name_key: AnnoKey = db.get_node_name_key();
        let nodes: Box<Iterator<Item = Match>> = db.node_annos.exact_anno_search(
            Some(node_name_key.ns.clone()),
            node_name_key.name.clone(),
            None.into(),
        );

        // first add all nodes that are a source of an edge as possible roots
        for m in nodes {
            let m: Match = m;
            let n = m.node;
            // insert all nodes to the root candidate list which are part of this component
            if orig.get_outgoing_edges(n).next().is_some() {
                roots.insert(n);
            }
        }

        let nodes: Box<Iterator<Item = Match>> = db.node_annos.exact_anno_search(
            Some(node_name_key.ns),
            node_name_key.name,
            None.into(),
        );
        for m in nodes {
            let m: Match = m;

            let source = m.node;

            let out_edges = orig.get_outgoing_edges(source);
            for target in out_edges {
                // remove the nodes that have an incoming edge from the root list
                roots.remove(&target);

                // add the edge annotations for this edge
                let e = Edge { source, target };
                let edge_annos = orig.get_anno_storage().get_annotations_for_item(&e);
                for a in edge_annos {
                    self.annos.insert(e.clone(), a)?;
                }
            }
        }

        for root_node in &roots {
            // iterate over all edges beginning from the root
            let mut chain: Vec<NodeID> = vec![*root_node];
            let pos: RelativePosition<PosT> = RelativePosition {
                root: *root_node,
                pos: PosT::zero(),
            };
            self.node_to_pos.insert(*root_node, pos);

            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), *root_node, 1, usize::max_value());
            for step in dfs {
                let step: DFSStep = step;

                if let Some(pos) = PosT::from_usize(chain.len()) {
                    let pos: RelativePosition<PosT> = RelativePosition {
                        root: *root_node,
                        pos,
                    };
                    self.node_to_pos.insert(step.node, pos);
                }
                chain.push(step.node);
            }
            chain.shrink_to_fit();
            self.node_chains.insert(*root_node, chain);
        }

        self.node_chains.shrink_to_fit();
        self.node_to_pos.shrink_to_fit();

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics();

        Ok(())
    }

    fn inverse_has_same_cost(&self) -> bool {
        true
    }

    fn as_edgecontainer(&self) -> &EdgeContainer {
        self
    }
}
