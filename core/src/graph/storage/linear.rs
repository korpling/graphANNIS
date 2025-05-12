use super::{
    deserialize_gs_field, legacy::LinearGraphStorageV1, load_statistics_from_location,
    save_statistics_to_toml, serialize_gs_field, EdgeContainer, GraphStatistic, GraphStorage,
};
use crate::{
    annostorage::{
        inmemory::AnnoStorageImpl, AnnotationStorage, EdgeAnnotationStorage, NodeAnnotationStorage,
    },
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID, NumValue},
};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::{clone::Clone, collections::HashMap, path::Path};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RelativePosition<PosT> {
    pub root: NodeID,
    pub pos: PosT,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LinearGraphStorage<PosT: NumValue> {
    node_to_pos: HashMap<NodeID, RelativePosition<PosT>>,
    node_chains: HashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

impl<PosT> LinearGraphStorage<PosT>
where
    PosT: NumValue,
{
    pub fn new() -> LinearGraphStorage<PosT> {
        LinearGraphStorage {
            node_to_pos: HashMap::default(),
            node_chains: HashMap::default(),
            annos: AnnoStorageImpl::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        self.node_to_pos.clear();
        self.node_chains.clear();
        self.annos.clear()?;
        self.stats = None;
        Ok(())
    }

    fn copy_edge_annos_for_node(
        &mut self,
        source_node: NodeID,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        // Iterate over the outgoing edges of this node to add the edge
        // annotations
        let out_edges = orig.get_outgoing_edges(source_node);
        for target in out_edges {
            let target = target?;
            let e = Edge {
                source: source_node,
                target,
            };
            let edge_annos = orig.get_anno_storage().get_annotations_for_item(&e)?;
            for a in edge_annos {
                self.annos.insert(e.clone(), a)?;
            }
        }
        Ok(())
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
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(pos) = self.node_to_pos.get(&node) {
            // find the next node in the chain
            if let Some(chain) = self.node_chains.get(&pos.root) {
                let next_pos = pos.pos.clone() + PosT::one();
                if let Some(next_pos) = next_pos.to_usize() {
                    if next_pos < chain.len() {
                        return Box::from(std::iter::once(Ok(chain[next_pos])));
                    }
                }
            }
        }
        Box::from(std::iter::empty())
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(pos) = self.node_to_pos.get(&node) {
            // find the previous node in the chain
            if let Some(chain) = self.node_chains.get(&pos.root) {
                if let Some(pos) = pos.pos.to_usize() {
                    if let Some(previous_pos) = pos.checked_sub(1) {
                        return Box::from(std::iter::once(Ok(chain[previous_pos])));
                    }
                }
            }
        }
        Box::from(std::iter::empty())
    }

    fn has_ingoing_edges(&self, node: NodeID) -> Result<bool> {
        let result = self
            .node_to_pos
            .get(&node)
            .map(|pos| !pos.pos.is_zero())
            .unwrap_or(false);
        Ok(result)
    }

    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        // use the node chains to find source nodes, but always skip the last element
        // because the last element is only a target node, not a source node
        let it = self
            .node_chains
            .iter()
            .flat_map(|(_root, chain)| chain.iter().rev().skip(1))
            .cloned()
            .map(Ok);

        Box::new(it)
    }

    fn root_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let it = self.node_chains.keys().copied().map(Ok);
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
    fn get_anno_storage(&self) -> &dyn EdgeAnnotationStorage {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        format!("LinearO{}V1", std::mem::size_of::<PosT>() * 8)
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let legacy_path = location.join("component.bin");
        let mut result: Self = if legacy_path.is_file() {
            let component: LinearGraphStorageV1<PosT> =
                deserialize_gs_field(location, "component")?;
            Self {
                node_to_pos: component.node_to_pos,
                node_chains: component.node_chains,
                annos: component.annos,
                stats: component.stats.map(|s| GraphStatistic::from(s)),
            }
        } else {
            let stats = load_statistics_from_location(location)?;
            Self {
                node_to_pos: deserialize_gs_field(location, "node_to_pos")?,
                node_chains: deserialize_gs_field(location, "node_chains")?,
                annos: deserialize_gs_field(location, "annos")?,
                stats: stats,
            }
        };

        result.annos.after_deserialization();
        Ok(result)
    }

    fn save_to(&self, location: &Path) -> Result<()> {
        serialize_gs_field(&self.node_to_pos, "node_to_pos", location)?;
        serialize_gs_field(&self.node_chains, "node_chains", location)?;
        serialize_gs_field(&self.annos, "annos", location)?;
        save_statistics_to_toml(location, self.stats.as_ref())?;
        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        source: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(start_pos) = self.node_to_pos.get(&source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    if let Some(min_distance) = offset.checked_add(min_distance) {
                        if min_distance < chain.len() {
                            let max_distance = match max_distance {
                                std::ops::Bound::Unbounded => {
                                    return Box::new(chain[min_distance..].iter().map(|n| Ok(*n)));
                                }
                                std::ops::Bound::Included(max_distance) => {
                                    offset + max_distance + 1
                                }
                                std::ops::Bound::Excluded(max_distance) => offset + max_distance,
                            };
                            // clip to chain length
                            let max_distance = std::cmp::min(chain.len(), max_distance);
                            if min_distance < max_distance {
                                return Box::new(
                                    chain[min_distance..max_distance].iter().map(|n| Ok(*n)),
                                );
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
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(start_pos) = self.node_to_pos.get(&source) {
            if let Some(chain) = self.node_chains.get(&start_pos.root) {
                if let Some(offset) = start_pos.pos.to_usize() {
                    let max_distance = match max_distance {
                        std::ops::Bound::Unbounded => 0,
                        std::ops::Bound::Included(max_distance) => {
                            offset.saturating_sub(max_distance)
                        }
                        std::ops::Bound::Excluded(max_distance) => {
                            offset.saturating_sub(max_distance + 1)
                        }
                    };

                    if let Some(min_distance) = offset.checked_sub(min_distance) {
                        if min_distance < chain.len() && max_distance <= min_distance {
                            // return all entries in the chain between min_distance..max_distance (inclusive)
                            return Box::new(
                                chain[max_distance..=min_distance].iter().map(|n| Ok(*n)),
                            );
                        } else if max_distance < chain.len() {
                            // return all entries in the chain between min_distance..max_distance
                            return Box::new(
                                chain[max_distance..chain.len()].iter().map(|n| Ok(*n)),
                            );
                        }
                    }
                }
            }
        }
        Box::new(std::iter::empty())
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>> {
        if source == target {
            return Ok(Some(0));
        }

        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(&source), self.node_to_pos.get(&target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    return Ok(Some(diff));
                }
            }
        }
        Ok(None)
    }

    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        if let (Some(source_pos), Some(target_pos)) =
            (self.node_to_pos.get(&source), self.node_to_pos.get(&target))
        {
            if source_pos.root == target_pos.root && source_pos.pos <= target_pos.pos {
                let diff = target_pos.pos.clone() - source_pos.pos.clone();
                if let Some(diff) = diff.to_usize() {
                    match max_distance {
                        std::ops::Bound::Unbounded => {
                            return Ok(diff >= min_distance);
                        }
                        std::ops::Bound::Included(max_distance) => {
                            return Ok(diff >= min_distance && diff <= max_distance);
                        }
                        std::ops::Bound::Excluded(max_distance) => {
                            return Ok(diff >= min_distance && diff < max_distance);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn copy(
        &mut self,
        _node_annos: &dyn NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.clear()?;

        // find all roots of the component
        let roots: Result<FxHashSet<NodeID>> = orig.root_nodes().collect();
        let roots = roots?;

        for root_node in &roots {
            self.copy_edge_annos_for_node(*root_node, orig)?;

            // iterate over all edges beginning from the root
            let mut chain: Vec<NodeID> = vec![*root_node];
            let pos: RelativePosition<PosT> = RelativePosition {
                root: *root_node,
                pos: PosT::zero(),
            };
            self.node_to_pos.insert(*root_node, pos);

            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), *root_node, 1, usize::MAX);
            for step in dfs {
                let step = step?;

                self.copy_edge_annos_for_node(step.node, orig)?;

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
        self.annos.calculate_statistics()?;

        Ok(())
    }

    fn inverse_has_same_cost(&self) -> bool {
        true
    }

    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }
}

#[cfg(test)]
mod tests;
