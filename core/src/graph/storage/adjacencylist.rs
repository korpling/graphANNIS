use crate::{
    annostorage::{
        AnnotationStorage, EdgeAnnotationStorage, NodeAnnotationStorage, inmemory::AnnoStorageImpl,
    },
    dfs::CycleSafeDFS,
    errors::Result,
    types::{AnnoKey, Annotation, Edge, NodeID},
};

use super::{
    EdgeContainer, GraphStatistic, GraphStorage, WriteableGraphStorage, deserialize_gs_field,
    legacy::{self, AdjacencyListStorageV1},
    load_statistics_from_location, save_statistics_to_toml, serialize_gs_field,
};
use itertools::Itertools;
use rustc_hash::FxHashSet;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::{ops::Bound, path::Path};

#[derive(Serialize, Deserialize, Clone)]
pub struct AdjacencyListStorage {
    edges: HashMap<NodeID, Vec<NodeID>>,
    inverse_edges: HashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

fn get_fan_outs(edges: &HashMap<NodeID, Vec<NodeID>>) -> Vec<usize> {
    let mut fan_outs: Vec<usize> = Vec::new();
    if !edges.is_empty() {
        for outgoing in edges.values() {
            fan_outs.push(outgoing.len());
        }
    }
    // order the fan-outs
    fan_outs.sort_unstable();

    fan_outs
}

impl Default for AdjacencyListStorage {
    fn default() -> Self {
        AdjacencyListStorage::new()
    }
}

impl AdjacencyListStorage {
    pub fn new() -> AdjacencyListStorage {
        AdjacencyListStorage {
            edges: HashMap::default(),
            inverse_edges: HashMap::default(),
            annos: AnnoStorageImpl::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        self.edges.clear();
        self.inverse_edges.clear();
        self.annos.clear()?;
        self.stats = None;
        Ok(())
    }
}

impl EdgeContainer for AdjacencyListStorage {
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(outgoing) = self.edges.get(&node) {
            return match outgoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(Ok(outgoing[0]))),
                _ => Box::new(outgoing.iter().copied().map(Ok)),
            };
        }
        Box::new(std::iter::empty())
    }

    fn has_outgoing_edges(&self, node: NodeID) -> Result<bool> {
        if let Some(outgoing) = self.edges.get(&node) {
            Ok(!outgoing.is_empty())
        } else {
            Ok(false)
        }
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(ingoing) = self.inverse_edges.get(&node) {
            return match ingoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(Ok(ingoing[0]))),
                _ => Box::new(ingoing.iter().map(|e| Ok(*e))),
            };
        }
        Box::new(std::iter::empty())
    }
    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let it = self
            .edges
            .iter()
            .filter(|(_, outgoing)| !outgoing.is_empty())
            .map(|(key, _)| Ok(*key));
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for AdjacencyListStorage {
    fn get_anno_storage(&self) -> &dyn EdgeAnnotationStorage {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        "AdjacencyListV1".to_owned()
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let legacy_path = location.join("component.bin");
        let mut result: Self = if legacy_path.is_file() {
            let component: AdjacencyListStorageV1 = deserialize_gs_field(location, "component")?;
            Self {
                stats: component.stats.map(GraphStatistic::from),
                edges: component.edges,
                inverse_edges: component.inverse_edges,
                annos: component.annos,
            }
        } else {
            let stats = load_statistics_from_location(location)?;
            Self {
                edges: deserialize_gs_field(location, "edges")?,
                inverse_edges: deserialize_gs_field(location, "inverse_edges")?,
                annos: deserialize_gs_field(location, "annos")?,
                stats,
            }
        };

        result.annos.after_deserialization();
        Ok(result)
    }

    fn save_to(&self, location: &Path) -> Result<()> {
        serialize_gs_field(&self.edges, "edges", location)?;
        serialize_gs_field(&self.inverse_edges, "inverse_edges", location)?;
        serialize_gs_field(&self.annos, "annos", location)?;
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
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance).filter_map_ok(
            move |x| {
                if visited.insert(x.node) {
                    Some(x.node)
                } else {
                    None
                }
            },
        );
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
            .filter_map_ok(move |x| {
                if visited.insert(x.node) {
                    Some(x.node)
                } else {
                    None
                }
            });
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

        Ok(it.next().is_some())
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

impl WriteableGraphStorage for AdjacencyListStorage {
    fn add_edge(&mut self, edge: Edge) -> Result<()> {
        if edge.source != edge.target {
            // insert to both regular and inverse maps

            let inverse_entry = self.inverse_edges.entry(edge.target).or_default();
            // no need to insert it: edge already exists
            if let Err(insertion_idx) = inverse_entry.binary_search(&edge.source) {
                inverse_entry.insert(insertion_idx, edge.source);
            }

            let regular_entry = self.edges.entry(edge.source).or_default();
            if let Err(insertion_idx) = regular_entry.binary_search(&edge.target) {
                regular_entry.insert(insertion_idx, edge.target);
            }
            self.stats = None;
        }
        Ok(())
    }

    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation) -> Result<()> {
        if let Some(outgoing) = self.edges.get(&edge.source)
            && outgoing.contains(&edge.target)
        {
            self.annos.insert(edge, anno)?;
        }
        Ok(())
    }

    fn delete_edge(&mut self, edge: &Edge) -> Result<()> {
        if let Some(outgoing) = self.edges.get_mut(&edge.source)
            && let Ok(idx) = outgoing.binary_search(&edge.target)
        {
            outgoing.remove(idx);
        }

        if let Some(ingoing) = self.inverse_edges.get_mut(&edge.target)
            && let Ok(idx) = ingoing.binary_search(&edge.source)
        {
            ingoing.remove(idx);
        }
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

        if let Some(outgoing) = self.edges.get(&node) {
            for target in outgoing.iter() {
                to_delete.push_back(Edge {
                    source: node,
                    target: *target,
                })
            }
        }
        if let Some(ingoing) = self.inverse_edges.get(&node) {
            for source in ingoing.iter() {
                to_delete.push_back(Edge {
                    source: *source,
                    target: node,
                })
            }
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
            for (source, outgoing) in &self.edges {
                roots.insert(*source);
                all_nodes.insert(*source);
                for target in outgoing {
                    all_nodes.insert(*target);

                    if stats.rooted_tree {
                        if has_incoming_edge.contains(target) {
                            stats.rooted_tree = false;
                        } else {
                            has_incoming_edge.insert(*target);
                        }
                    }
                }
            }
            stats.nodes = all_nodes.len();
        }

        if !self.edges.is_empty() {
            for outgoing in self.edges.values() {
                for target in outgoing {
                    roots.remove(target);
                }
            }
        }
        stats.root_nodes = roots.len();

        let fan_outs = get_fan_outs(&self.edges);
        let sum_fan_out: usize = fan_outs.iter().sum();

        if let Some(last) = fan_outs.last() {
            stats.max_fan_out = *last;
        }
        let inverse_fan_outs = get_fan_outs(&self.inverse_edges);

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
        if roots.is_empty() && !self.edges.is_empty() {
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

impl From<legacy::AdjacencyListStorageV1> for AdjacencyListStorage {
    fn from(value: legacy::AdjacencyListStorageV1) -> Self {
        Self {
            edges: value.edges,
            inverse_edges: value.inverse_edges,
            annos: value.annos,
            stats: value.stats.map(GraphStatistic::from),
        }
    }
}

#[cfg(test)]
mod tests;
