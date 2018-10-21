use super::*;
use annis::db::annostorage::AnnoStorage;
use annis::db::AnnotationStorage;
use annis::dfs::CycleSafeDFS;
use annis::types::Edge;

use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;

use bincode;

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct AdjacencyListStorage {
    edges: FxHashMap<NodeID, Vec<NodeID>>,
    inverse_edges: FxHashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorage<Edge>,
    stats: Option<GraphStatistic>,
}

impl Default for AdjacencyListStorage {
    fn default() -> Self {
        AdjacencyListStorage::new()
    }
}

impl AdjacencyListStorage {
    pub fn new() -> AdjacencyListStorage {
        AdjacencyListStorage {
            edges: FxHashMap::default(),
            inverse_edges: FxHashMap::default(),
            annos: AnnoStorage::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.inverse_edges.clear();
        self.annos.clear();
        self.stats = None;
    }
}

impl EdgeContainer for AdjacencyListStorage {
    fn get_outgoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(outgoing) = self.edges.get(&node) {
            return match outgoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(outgoing[0])),
                _ => Box::new(outgoing.iter().cloned()),
            };
        }
        Box::new(std::iter::empty())
    }

    fn get_ingoing_edges<'a>(&'a self, node: NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        if let Some(ingoing) = self.inverse_edges.get(&node) {
            return match ingoing.len() {
                0 => Box::new(std::iter::empty()),
                1 => Box::new(std::iter::once(ingoing[0])),
                _ => Box::new(ingoing.iter().cloned()),
            };
        }
        Box::new(std::iter::empty())
    }

    fn get_anno_storage(&self) -> &AnnotationStorage<Edge> {
        &self.annos
    }

    fn source_nodes<'a>(&'a self) -> Box<Iterator<Item = NodeID> + 'a> {
        let it = self
            .edges
            .iter()
            .filter(|(_, outgoing)| !outgoing.is_empty())
            .map(|(key, _)| key.clone());
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for AdjacencyListStorage {
    fn serialization_id(&self) -> String {
        "AdjacencyListV1".to_owned()
    }

    fn serialize_gs(&self, writer: &mut std::io::Write) -> Result<()> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(n.clone()));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(n.clone()));
        Box::new(it)
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize> {
        let mut it = CycleSafeDFS::new(self, *source, usize::min_value(), usize::max_value())
            .filter(|x| *target == x.node)
            .map(|x| x.distance);

        it.next()
    }
    fn is_connected(
        &self,
        source: &NodeID,
        target: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> bool {
        let mut it = CycleSafeDFS::new(self, *source, min_distance, max_distance)
            .filter(|x| *target == x.node);

        it.next().is_some()
    }

    fn copy(&mut self, _db: &Graph, orig: &EdgeContainer) {
        self.clear();

        for source in orig.source_nodes() {
            for target in orig.get_outgoing_edges(source) {
                let e = Edge { source, target };
                self.add_edge(e.clone());
                for a in orig.get_anno_storage().get_annotations_for_item(&e) {
                    self.add_edge_annotation(e.clone(), a);
                }
            }
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics();
    }

    fn as_writeable(&mut self) -> Option<&mut WriteableGraphStorage> {
        Some(self)
    }
    fn as_edgecontainer(&self) -> &EdgeContainer {
        self
    }

    fn inverse_has_same_cost(&self) -> bool {
        true
    }
}

impl WriteableGraphStorage for AdjacencyListStorage {
    fn add_edge(&mut self, edge: Edge) {
        if edge.source != edge.target {
            // insert to both regular and inverse maps

            let inverse_entry = self
                .inverse_edges
                .entry(edge.target)
                .or_insert_with(|| Vec::default());
            // no need to insert it edge already exists
            if let Err(insertion_idx) = inverse_entry.binary_search(&edge.source) {
                inverse_entry.insert(insertion_idx, edge.source);
            }

            let regular_entry = self.edges.entry(edge.source).or_insert_with(|| Vec::default());
            if let Err(insertion_idx) = regular_entry.binary_search(&edge.target) {
                regular_entry.insert(insertion_idx, edge.target);
            }
            // TODO: invalid graph statistics
        }
    }
    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation) {
        if let Some(outgoing) = self.edges.get(&edge.source) {
            if outgoing.contains(&edge.target) {
                self.annos.insert(edge, anno);
            }
        }
    }

    fn delete_edge(&mut self, edge: &Edge) {
        if let Some(outgoing) = self.edges.get_mut(&edge.source) {
            if let Ok(idx) = outgoing.binary_search(&edge.target) {
                outgoing.remove(idx);
            }
        }

        if let Some(ingoing) = self.inverse_edges.get_mut(&edge.target) {
            if let Ok(idx) = ingoing.binary_search(&edge.source) {
                ingoing.remove(idx);
            }
        }
        let annos = self.annos.get_annotations_for_item(edge);
        for a in annos.into_iter() {
            self.annos.remove_annotation_for_item(edge, &a.key);
        }
    }
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey) {
        self.annos.remove_annotation_for_item(edge, anno_key);
    }
    fn delete_node(&mut self, node: &NodeID) {
        // find all both ingoing and outgoing edges
        let mut to_delete = std::collections::LinkedList::<Edge>::new();

        if let Some(outgoing) = self.edges.get(node) {
            for target in outgoing.iter() {
                to_delete.push_back(Edge {
                    source: *node,
                    target: *target,
                })
            }
        }
        if let Some(ingoing) = self.inverse_edges.get(node) {
            for source in ingoing.iter() {
                to_delete.push_back(Edge {
                    source: *source,
                    target: *node,
                })
            }
        }

        for e in to_delete {
            self.delete_edge(&e);
        }
    }

    fn calculate_statistics(&mut self) {
        let mut stats = GraphStatistic {
            max_depth: 1,
            max_fan_out: 0,
            avg_fan_out: 0.0,
            fan_out_99_percentile: 0,
            cyclic: false,
            rooted_tree: true,
            nodes: 0,
            dfs_visit_ratio: 0.0,
        };

        self.annos.calculate_statistics();

        let mut sum_fan_out = 0;
        let mut has_incoming_edge: BTreeSet<NodeID> = BTreeSet::new();

        // find all root nodes
        let mut roots: BTreeSet<NodeID> = BTreeSet::new();
        {
            let mut all_nodes: BTreeSet<NodeID> = BTreeSet::new();
            for (source, outgoing) in self.edges.iter() {
                roots.insert(*source);
                all_nodes.insert(*source);
                for target in outgoing.iter() {
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

        let mut fan_outs: Vec<usize> = Vec::new();
        let mut last_source_id: Option<NodeID> = None;
        let mut current_fan_out = 0;
        if !self.edges.is_empty() {
            for (source, outgoing) in self.edges.iter() {
                for target in outgoing.iter() {
                    roots.remove(&target);

                    if let Some(last) = last_source_id {
                        if last != *source {
                            stats.max_fan_out = std::cmp::max(stats.max_fan_out, current_fan_out);
                            sum_fan_out += current_fan_out;
                            fan_outs.push(current_fan_out);

                            current_fan_out = 0;
                        }
                    }
                    last_source_id = Some(*source);
                    current_fan_out += 1;
                }
            }
            // add the statistics for the last node
            stats.max_fan_out = std::cmp::max(stats.max_fan_out, current_fan_out);
            sum_fan_out += current_fan_out;
            fan_outs.push(current_fan_out);
        }
        // order the fan-outs
        fan_outs.sort();

        // get the percentile value(s)
        // set some default values in case there are not enough elements in the component
        if !fan_outs.is_empty() {
            stats.fan_out_99_percentile = fan_outs[fan_outs.len() - 1];
        }
        // calculate the more accurate values
        if fan_outs.len() >= 100 {
            let idx: usize = fan_outs.len() / 100;
            if idx < fan_outs.len() {
                stats.fan_out_99_percentile = fan_outs[idx];
            }
        }

        let mut number_of_visits = 0;
        if roots.is_empty() && !self.edges.is_empty() {
            // if we have edges but no roots at all there must be a cycle
            stats.cyclic = true;
        } else {
            for root_node in roots.iter() {
                let mut dfs = CycleSafeDFS::new(self, *root_node, 0, usize::max_value());
                while let Some(step) = dfs.next() {
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
            stats.dfs_visit_ratio = (number_of_visits as f64) / (stats.nodes as f64);
        }

        if sum_fan_out > 0 && stats.nodes > 0 {
            stats.avg_fan_out = (sum_fan_out as f64) / (stats.nodes as f64);
        }

        self.stats = Some(stats);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use itertools::Itertools;

    #[test]
    fn simple_dag_find_all() {
        /*
        +---+     +---+     +---+     +---+
        | 7 | <-- | 5 | <-- | 3 | <-- | 1 |
        +---+     +---+     +---+     +---+
                    |         |         |
                    |         |         |
                    v         |         v
                  +---+       |       +---+
                  | 6 |       |       | 2 |
                  +---+       |       +---+
                              |         |
                              |         |
                              |         v
                              |       +---+
                              +-----> | 4 |
                                      +---+
        */
        let mut gs = AdjacencyListStorage::new();

        gs.add_edge(Edge {
            source: 1,
            target: 2,
        });
        gs.add_edge(Edge {
            source: 2,
            target: 4,
        });
        gs.add_edge(Edge {
            source: 1,
            target: 3,
        });
        gs.add_edge(Edge {
            source: 3,
            target: 5,
        });
        gs.add_edge(Edge {
            source: 5,
            target: 7,
        });
        gs.add_edge(Edge {
            source: 5,
            target: 6,
        });
        gs.add_edge(Edge {
            source: 3,
            target: 4,
        });

        assert_eq!(
            vec![2, 3],
            gs.get_outgoing_edges(1)
                .collect::<Vec<NodeID>>()
                .into_iter()
                .sorted()
        );
        assert_eq!(
            vec![4, 5],
            gs.get_outgoing_edges(3)
                .collect::<Vec<NodeID>>()
                .into_iter()
                .sorted()
        );
        assert_eq!(0, gs.get_outgoing_edges(6).count());
        assert_eq!(vec![4], gs.get_outgoing_edges(2).collect::<Vec<NodeID>>());

        let mut reachable: Vec<NodeID> = gs.find_connected(1, 1, 100).collect();
        reachable.sort();
        assert_eq!(vec![2, 3, 4, 5, 6, 7], reachable);

        let mut reachable: Vec<NodeID> = gs.find_connected(3, 2, 100).collect();
        reachable.sort();
        assert_eq!(vec![6, 7], reachable);

        let mut reachable: Vec<NodeID> = gs.find_connected(1, 2, 4).collect();
        reachable.sort();
        assert_eq!(vec![4, 5, 6, 7], reachable);

        let reachable: Vec<NodeID> = gs.find_connected(7, 1, 100).collect();
        assert_eq!(true, reachable.is_empty());
    }

}
