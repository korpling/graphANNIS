use super::*;

use crate::{
    annostorage::{ondisk::AnnoStorageImpl, NodeAnnotationStorage},
    dfs::CycleSafeDFS,
    errors::Result,
    util::disk_collections::{DiskMap, EvictionStrategy, DEFAULT_BLOCK_CACHE_CAPACITY},
};
use itertools::Itertools;
use rustc_hash::FxHashSet;
use std::collections::{BTreeSet, HashMap};
use std::ops::Bound;
use transient_btree_index::BtreeConfig;

pub const SERIALIZATION_ID: &str = "DiskAdjacencyListV1";

pub struct DiskAdjacencyListStorage {
    edges: DiskMap<Edge, bool>,
    inverse_edges: DiskMap<Edge, bool>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

fn get_fan_outs(edges: &DiskMap<Edge, bool>) -> Result<Vec<usize>> {
    let mut fan_outs: HashMap<NodeID, usize> = HashMap::default();
    if !edges.is_empty()? {
        let all_edges = edges.iter()?;
        for e in all_edges {
            let (e, _) = e?;
            fan_outs
                .entry(e.source)
                .and_modify(|num_out| *num_out += 1)
                .or_insert(1);
        }
    }
    // order the fan-outs
    let mut fan_outs: Vec<usize> = fan_outs.into_values().collect();
    fan_outs.sort_unstable();

    Ok(fan_outs)
}

impl DiskAdjacencyListStorage {
    pub fn new() -> Result<DiskAdjacencyListStorage> {
        Ok(DiskAdjacencyListStorage {
            edges: DiskMap::default(),
            inverse_edges: DiskMap::default(),
            annos: AnnoStorageImpl::new(None)?,
            stats: None,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.edges.clear();
        self.inverse_edges.clear();
        self.annos.clear()?;
        self.stats = None;
        Ok(())
    }
}

impl EdgeContainer for DiskAdjacencyListStorage {
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::MIN,
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::MAX,
        };
        Box::new(
            self.edges
                .range(lower_bound..upper_bound)
                .map_ok(|(e, _)| e.target),
        )
    }

    fn has_outgoing_edges(&self, node: NodeID) -> Result<bool> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::MIN,
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::MAX,
        };
        if let Some(edge) = self.edges.range(lower_bound..upper_bound).next() {
            edge?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::MIN,
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::MAX,
        };
        Box::new(
            self.inverse_edges
                .range(lower_bound..upper_bound)
                .map_ok(|(e, _)| e.target),
        )
    }
    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        match self.edges.iter() {
            // The unique_by will merge all errors into a single error, which should be ok for our use case
            Ok(edges) => Box::new(edges.map_ok(|(e, _)| e.source).unique_by(|n| match n {
                Ok(n) => Some(*n),
                Err(_) => None,
            })),
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for DiskAdjacencyListStorage {
    fn get_anno_storage(&self) -> &dyn EdgeAnnotationStorage {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        SERIALIZATION_ID.to_owned()
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let stats = load_statistics_from_location(location)?;
        let result = DiskAdjacencyListStorage {
            edges: DiskMap::new(
                Some(&location.join("edges.bin")),
                EvictionStrategy::default(),
                DEFAULT_BLOCK_CACHE_CAPACITY,
                BtreeConfig::default()
                    .fixed_key_size(std::mem::size_of::<NodeID>() * 2)
                    .fixed_value_size(2),
            )?,
            inverse_edges: DiskMap::new(
                Some(&location.join("inverse_edges.bin")),
                EvictionStrategy::default(),
                DEFAULT_BLOCK_CACHE_CAPACITY,
                BtreeConfig::default()
                    .fixed_key_size(std::mem::size_of::<NodeID>() * 2)
                    .fixed_value_size(2),
            )?,
            annos: AnnoStorageImpl::new(Some(
                location.join(crate::annostorage::ondisk::SUBFOLDER_NAME),
            ))?,
            stats,
        };
        Ok(result)
    }

    fn save_to(&self, location: &Path) -> Result<()> {
        self.edges.write_to(&location.join("edges.bin"))?;
        self.inverse_edges
            .write_to(&location.join("inverse_edges.bin"))?;
        self.annos.save_annotations_to(location)?;
        save_statistics_to_toml(location, self.stats.as_ref())?;
        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::MAX,
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance - 1,
        };
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map_ok(|x| x.node)
            .filter_ok(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::MAX,
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance - 1,
        };

        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .map_ok(|x| x.node)
            .filter_ok(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>> {
        let mut it = CycleSafeDFS::new(self, source, usize::MIN, usize::MAX)
            .filter_ok(|x| target == x.node)
            .map_ok(|x| x.distance);

        match it.next() {
            Some(distance) => {
                let distance = distance?;
                Ok(Some(distance))
            }
            None => Ok(None),
        }
    }
    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        let max_distance = match max_distance {
            Bound::Unbounded => usize::MAX,
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance - 1,
        };
        let mut it = CycleSafeDFS::new(self, source, min_distance, max_distance)
            .filter_ok(|x| target == x.node);

        match it.next() {
            Some(next) => {
                if let Err(e) = next {
                    Err(e)
                } else {
                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    fn copy(
        &mut self,
        _node_annos: &dyn NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.clear()?;

        for source in orig.source_nodes() {
            let source = source?;
            for target in orig.get_outgoing_edges(source) {
                let target = target?;
                let e = Edge { source, target };
                self.add_edge(e.clone())?;
                for a in orig.get_anno_storage().get_annotations_for_item(&e)? {
                    self.add_edge_annotation(e.clone(), a)?;
                }
            }
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics()?;
        Ok(())
    }

    fn as_writeable(&mut self) -> Option<&mut dyn WriteableGraphStorage> {
        Some(self)
    }
    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }

    fn inverse_has_same_cost(&self) -> bool {
        true
    }
}

impl WriteableGraphStorage for DiskAdjacencyListStorage {
    fn add_edge(&mut self, edge: Edge) -> Result<()> {
        if edge.source != edge.target {
            // insert to both regular and inverse maps
            self.inverse_edges.insert(edge.inverse(), true)?;
            self.edges.insert(edge, true)?;
            self.stats = None;
        }
        Ok(())
    }

    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation) -> Result<()> {
        if self.edges.get(&edge)?.is_some() {
            self.annos.insert(edge, anno)?;
        }
        Ok(())
    }

    fn delete_edge(&mut self, edge: &Edge) -> Result<()> {
        self.edges.remove(edge)?;
        self.inverse_edges.remove(&edge.inverse())?;

        self.annos.remove_item(edge)?;

        Ok(())
    }
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey) -> Result<()> {
        self.annos.remove_annotation_for_item(edge, anno_key)?;
        Ok(())
    }
    fn delete_node(&mut self, node: NodeID) -> Result<()> {
        // find all both ingoing and outgoing edges
        let mut to_delete = std::collections::LinkedList::<Edge>::new();

        for target in self.get_outgoing_edges(node) {
            let target = target?;
            to_delete.push_back(Edge {
                source: node,
                target,
            });
        }

        for source in self.get_ingoing_edges(node) {
            let source = source?;
            to_delete.push_back(Edge {
                source,
                target: node,
            });
        }

        for e in to_delete {
            self.delete_edge(&e)?;
        }

        Ok(())
    }

    fn calculate_statistics(&mut self) -> Result<()> {
        let mut stats = GraphStatistic {
            max_depth: 1,
            max_fan_out: 0,
            avg_fan_out: 0.0,
            fan_out_99_percentile: 0,
            inverse_fan_out_99_percentile: 0,
            cyclic: false,
            rooted_tree: true,
            nodes: 0,
            root_nodes: 0,
            dfs_visit_ratio: 0.0,
        };

        self.annos.calculate_statistics()?;

        let mut has_incoming_edge: BTreeSet<NodeID> = BTreeSet::new();

        // find all root nodes
        let mut roots: BTreeSet<NodeID> = BTreeSet::new();
        {
            let mut all_nodes: BTreeSet<NodeID> = BTreeSet::new();
            for edge in self.edges.iter()? {
                let (e, _) = edge?;
                roots.insert(e.source);
                all_nodes.insert(e.source);
                all_nodes.insert(e.target);

                if stats.rooted_tree {
                    if has_incoming_edge.contains(&e.target) {
                        stats.rooted_tree = false;
                    } else {
                        has_incoming_edge.insert(e.target);
                    }
                }
            }
            stats.nodes = all_nodes.len();
        }

        let edges_empty = self.edges.is_empty()?;

        if !edges_empty {
            for edge in self.edges.iter()? {
                let (e, _) = edge?;
                roots.remove(&e.target);
            }
        }
        stats.root_nodes = roots.len();

        let fan_outs = get_fan_outs(&self.edges)?;
        let sum_fan_out: usize = fan_outs.iter().sum();

        if let Some(last) = fan_outs.last() {
            stats.max_fan_out = *last;
        }
        let inverse_fan_outs = get_fan_outs(&self.inverse_edges)?;

        // get the percentile value(s)
        // set some default values in case there are not enough elements in the component
        if !fan_outs.is_empty() {
            stats.fan_out_99_percentile = fan_outs[fan_outs.len() - 1];
        }
        if !inverse_fan_outs.is_empty() {
            stats.inverse_fan_out_99_percentile = inverse_fan_outs[inverse_fan_outs.len() - 1];
        }
        // calculate the more accurate values
        if fan_outs.len() >= 100 {
            let idx: usize = fan_outs.len() / 100;
            if idx < fan_outs.len() {
                stats.fan_out_99_percentile = fan_outs[idx];
            }
        }
        if inverse_fan_outs.len() >= 100 {
            let idx: usize = inverse_fan_outs.len() / 100;
            if idx < inverse_fan_outs.len() {
                stats.inverse_fan_out_99_percentile = inverse_fan_outs[idx];
            }
        }

        let mut number_of_visits = 0;
        if roots.is_empty() && !edges_empty {
            // if we have edges but no roots at all there must be a cycle
            stats.cyclic = true;
        } else {
            for root_node in &roots {
                let mut dfs = CycleSafeDFS::new(self, *root_node, 0, usize::MAX);
                for step in &mut dfs {
                    let step = step?;
                    number_of_visits += 1;
                    stats.max_depth = std::cmp::max(stats.max_depth, step.distance);
                }
                if dfs.is_cyclic() {
                    stats.cyclic = true;
                }
            }
        }

        if stats.cyclic {
            stats.rooted_tree = false;
            // it's infinite
            stats.max_depth = 0;
            stats.dfs_visit_ratio = 0.0;
        } else if stats.nodes > 0 {
            stats.dfs_visit_ratio = f64::from(number_of_visits) / (stats.nodes as f64);
        }

        if sum_fan_out > 0 && stats.nodes > 0 {
            stats.avg_fan_out = (sum_fan_out as f64) / (stats.nodes as f64);
        }

        self.stats = Some(stats);

        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.annos.clear()?;
        self.edges.clear();
        self.inverse_edges.clear();
        self.stats = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
