use graphstorage::EdgeContainer;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::any::Any;
use std::clone::Clone;
use std;
use serde::{Serialize, Deserialize};
use bincode;

use {AnnoKey, Annotation, Edge, Match, NodeID, NumValue};
use super::{GraphStatistic, GraphStorage};
use annostorage::AnnoStorage;
use graphdb::GraphDB;
use dfs::{CycleSafeDFS, DFSStep};
use errors::*;

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
struct RelativePosition<PosT> {
    pub root: NodeID,
    pub pos: PosT,
}

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct LinearGraphStorage<PosT: NumValue> {
    node_to_pos: FxHashMap<NodeID, RelativePosition<PosT>>,
    node_chains: FxHashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorage<Edge>,
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
            annos: AnnoStorage::new(),
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


impl<PosT: 'static> EdgeContainer for  LinearGraphStorage<PosT> 
where PosT : NumValue {



    fn get_outgoing_edges<'a>(&'a self, node: &NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(pos) = self.node_to_pos.get(node) {
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
        return Box::from(std::iter::empty());
    }

    fn get_ingoing_edges<'a>(&'a self, node: &NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(pos) = self.node_to_pos.get(node) {
            // find the previous node in the chain
            if let Some(chain) = self.node_chains.get(&pos.root) {
                if let Some(pos) = pos.pos.to_usize() {
                    if let Some(previous_pos) = pos.checked_sub(1) {
                        return Box::from(std::iter::once(chain[previous_pos]));
                    }
                }
            }
        }
        return Box::from(std::iter::empty());
    }

    fn get_edge_annos(&self, edge : &Edge) -> Vec<Annotation> {
        return self.annos.get_annotations_for_item(edge);
    }

    fn get_anno_storage(&self) -> &AnnoStorage<Edge> {
        &self.annos
    }

    fn source_nodes<'a>(&'a self) -> Box<Iterator<Item = NodeID> + 'a> {
        // use the node chains to find source nodes, but always skip the last element
        // because the last element is only a target node, not a source node
        let it = self.node_chains
            .iter()
            .flat_map(|(_root, chain)| chain.iter().rev().skip(1))
            .cloned();

        return Box::new(it);
    }
    
    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl<PosT: 'static> GraphStorage for LinearGraphStorage<PosT>
where
    for <'de> PosT: NumValue + Deserialize<'de> + Serialize,
{
    fn serialization_id(&self) -> String {
        format!("LinearO{}V1", std::mem::size_of::<PosT>()*8)
    }

    fn serialize_gs(&self, writer: &mut std::io::Write) -> Result<()> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        source: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(start_pos) = self.node_to_pos.get(source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    let max_distance = offset + max_distance;
                    let min_distance = offset + min_distance;

                    // clip to chain length
                    let max_distance = std::cmp::min(chain.len(), max_distance + 1);
                    if min_distance < chain.len() {
                        // return all entries in the chain between min_distance..max_distance
                        return Box::new(chain[min_distance..max_distance].iter().cloned());
                    }
                }
            }
        }
        return Box::new(std::iter::empty());
    }

    fn find_connected_inverse<'a>(
        &'a self,
        source: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {

        if let Some(start_pos) = self.node_to_pos.get(source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    let max_distance = offset.checked_sub(max_distance).unwrap_or(0);

                    if let Some(min_distance) = offset.checked_sub(min_distance) {
                        if min_distance < chain.len() {
                            // return all entries in the chain between min_distance..max_distance
                            return Box::new(chain[max_distance..(min_distance+1)].iter().cloned());
                        } else {
                            // return all entries in the chain between min_distance..max_distance
                            return Box::new(chain[max_distance..chain.len()].iter().cloned());
                        }
                    }
                }
            }
        }
        return Box::new(std::iter::empty());
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize> {
        if source == target {
            return Some(0);
        }

        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(source), self.node_to_pos.get(target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    return Some(diff);
                }
            }
        }
        return None;
    }

    fn is_connected(
        &self,
        source: &NodeID,
        target: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> bool {
        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(source), self.node_to_pos.get(target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    if diff >= min_distance && diff <= max_distance {
                        return true;
                    }
                }
            }
        }

        return false;
    }


    fn copy(&mut self, db: &GraphDB, orig: &EdgeContainer) {
        self.clear();

        // find all roots of the component
        let mut roots: FxHashSet<NodeID> = FxHashSet::default();
        let node_name_key: AnnoKey = db.get_node_name_key();
        let nodes: Box<Iterator<Item = Match>> =
            db.node_annos
                .exact_anno_search(Some(node_name_key.ns.clone()), node_name_key.name.clone(), None);

        // first add all nodes that are a source of an edge as possible roots
        for m in nodes {
            let m: Match = m;
            let n = m.node;
            // insert all nodes to the root candidate list which are part of this component
            if orig.get_outgoing_edges(&n).next().is_some() {
                roots.insert(n);
            }
        }

        let nodes: Box<Iterator<Item = Match>> =
            db.node_annos
                .exact_anno_search(Some(node_name_key.ns), node_name_key.name, None);
        for m in nodes {
            let m: Match = m;

            let source = m.node;

            let out_edges = orig.get_outgoing_edges(&source);
            for target in out_edges {
                // remove the nodes that have an incoming edge from the root list
                roots.remove(&target);

                // add the edge annotations for this edge
                let e = Edge { source, target };
                let edge_annos = orig.get_edge_annos(&e);
                for a in edge_annos.into_iter() {
                    self.annos.insert(e.clone(), a);
                }
            }
        }

        for root_node in roots.iter() {
            // iterate over all edges beginning from the root
            let mut chain: Vec<NodeID> = vec![root_node.clone()];
            let pos: RelativePosition<PosT> = RelativePosition {
                root: root_node.clone(),
                pos: PosT::zero(),
            };
            self.node_to_pos.insert(root_node.clone(), pos);

            let dfs = CycleSafeDFS::new(orig, &root_node, 1, usize::max_value());
            for step in dfs {
                let step: DFSStep = step;

                if let Some(pos) = PosT::from_usize(chain.len()) {
                    let pos: RelativePosition<PosT> = RelativePosition {
                        root: root_node.clone(),
                        pos: pos,
                    };
                    self.node_to_pos.insert(step.node.clone(), pos);
                }
                chain.push(step.node);
            }
            chain.shrink_to_fit();
            self.node_chains.insert(root_node.clone(), chain);
        }

        self.node_chains.shrink_to_fit();
        self.node_to_pos.shrink_to_fit();

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics();
    }

    fn inverse_has_same_cost(&self) -> bool {true}  

    fn as_any(&self) -> &Any {
        self
    }

    fn as_edgecontainer(&self) -> &EdgeContainer {self}

    
}
