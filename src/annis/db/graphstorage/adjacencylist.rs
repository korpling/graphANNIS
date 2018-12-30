use super::*;
use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::AnnotationStorage;
use crate::annis::dfs::CycleSafeDFS;
use crate::annis::types::Edge;

use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;
use std::ops::Bound;

use bincode;

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
pub struct AdjacencyListStorage {
    edges: FxHashMap<NodeID, Vec<NodeID>>,
    inverse_edges: FxHashMap<NodeID, Vec<NodeID>>,
    annos: AnnoStorage<Edge>,
    stats: Option<GraphStatistic>,
}

fn get_fan_outs(edges: &FxHashMap<NodeID, Vec<NodeID>>) -> Vec<usize> {
    let mut fan_outs: Vec<usize> = Vec::new();
    if !edges.is_empty() {
        for (_, outgoing) in edges {
            fan_outs.push(outgoing.len());
        }
    }
    // order the fan-outs
    fan_outs.sort();

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
    fn source_nodes<'a>(&'a self) -> Box<Iterator<Item = NodeID> + 'a> {
        let it = self
            .edges
            .iter()
            .filter(|(_, outgoing)| !outgoing.is_empty())
            .map(|(key, _)| *key);
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for AdjacencyListStorage {
    fn get_anno_storage(&self) -> &AnnotationStorage<Edge> {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        "AdjacencyListV1".to_owned()
    }

    fn serialize_gs(&self, writer: &mut std::io::Write) -> Result<()> {
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    fn deserialize_gs(input: &mut std::io::Read) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let mut result: AdjacencyListStorage = bincode::deserialize_from(input)?;
        result.annos.after_deserialization();
        Ok(result)
    }

    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(n.clone()));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };

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
        max_distance: std::ops::Bound<usize>,
    ) -> bool {
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let mut it = CycleSafeDFS::new(self, *source, min_distance, max_distance)
            .filter(|x| *target == x.node);

        it.next().is_some()
    }

    fn copy(&mut self, _db: &Graph, orig: &GraphStorage) {
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
                .or_insert_with(Vec::default);
            // no need to insert it: edge already exists
            if let Err(insertion_idx) = inverse_entry.binary_search(&edge.source) {
                inverse_entry.insert(insertion_idx, edge.source);
            }

            let regular_entry = self.edges.entry(edge.source).or_insert_with(Vec::default);
            if let Err(insertion_idx) = regular_entry.binary_search(&edge.target) {
                regular_entry.insert(insertion_idx, edge.target);
            }
            self.stats = None;
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
        for a in annos {
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
            inverse_fan_out_99_percentile: 0,
            cyclic: false,
            rooted_tree: true,
            nodes: 0,
            dfs_visit_ratio: 0.0,
        };

        self.annos.calculate_statistics();

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
            for (_, outgoing) in &self.edges {
                for target in outgoing {
                    roots.remove(&target);
                }
            }
        }

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
            stats.dfs_visit_ratio = f64::from(number_of_visits) / (stats.nodes as f64);
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
    fn multiple_paths_find_range() {
        /*
        +---+
        | 1 | -+
        +---+  |
            |    |
            |    |
            v    |
        +---+  |
        | 2 |  |
        +---+  |
            |    |
            |    |
            v    |
        +---+  |
        | 3 | <+
        +---+
            |
            |
            v
        +---+
        | 4 |
        +---+
            |
            |
            v
        +---+
        | 5 |
        +---+
        */

        let mut gs = AdjacencyListStorage::new();
        gs.add_edge(Edge {
            source: 1,
            target: 2,
        });
        gs.add_edge(Edge {
            source: 2,
            target: 3,
        });
        gs.add_edge(Edge {
            source: 3,
            target: 4,
        });
        gs.add_edge(Edge {
            source: 1,
            target: 3,
        });
        gs.add_edge(Edge {
            source: 4,
            target: 5,
        });

        let mut found: Vec<NodeID> = gs
            .find_connected(1, 3, std::ops::Bound::Included(3))
            .collect();

        assert_eq!(2, found.len());
        found.sort();

        assert_eq!(4, found[0]);
        assert_eq!(5, found[1]);
    }

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

        let mut reachable: Vec<NodeID> = gs.find_connected(1, 1, Bound::Included(100)).collect();
        reachable.sort();
        assert_eq!(vec![2, 3, 4, 5, 6, 7], reachable);

        let mut reachable: Vec<NodeID> = gs.find_connected(3, 2, Bound::Included(100)).collect();
        reachable.sort();
        assert_eq!(vec![6, 7], reachable);

        let mut reachable: Vec<NodeID> = gs.find_connected(1, 2, Bound::Included(4)).collect();
        reachable.sort();
        assert_eq!(vec![4, 5, 6, 7], reachable);

        let reachable: Vec<NodeID> = gs.find_connected(7, 1, Bound::Included(100)).collect();
        assert_eq!(true, reachable.is_empty());
    }

    #[test]
    fn indirect_cycle_statistics() {
        let mut gs = AdjacencyListStorage::new();

        gs.add_edge(Edge {
            source: 1,
            target: 2,
        });

        gs.add_edge(Edge {
            source: 2,
            target: 3,
        });

        gs.add_edge(Edge {
            source: 3,
            target: 4,
        });

        gs.add_edge(Edge {
            source: 4,
            target: 5,
        });

        gs.add_edge(Edge {
            source: 5,
            target: 2,
        });

        gs.calculate_statistics();
        assert_eq!(true, gs.get_statistics().is_some());
        let stats = gs.get_statistics().unwrap();
        assert_eq!(true, stats.cyclic);
    }

    #[ignore]
    #[test]
    fn multi_branch_cycle_statistics() {
        let mut gs = AdjacencyListStorage::new();

        gs.add_edge(Edge {
            source: 903,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1343,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1342,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1339,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 904,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1336,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1337,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1335,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1340,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1338,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1334,
        });
        gs.add_edge(Edge {
            source: 903,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 904,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1174,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1295,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1310,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1334,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1335,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1336,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1337,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1338,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1339,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1340,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1341,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1342,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 1343,
            target: 1341,
        });
        gs.add_edge(Edge {
            source: 905,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 908,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 909,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 916,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 942,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 945,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 946,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 906,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 934,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 967,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 990,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1062,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1177,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1263,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1266,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1291,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1294,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1296,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 1311,
            target: 946,
        });
        gs.add_edge(Edge {
            source: 948,
            target: 948,
        });
        gs.add_edge(Edge {
            source: 951,
            target: 954,
        });
        gs.add_edge(Edge {
            source: 954,
            target: 954,
        });
        gs.add_edge(Edge {
            source: 1016,
            target: 954,
        });
        gs.add_edge(Edge {
            source: 944,
            target: 944,
        });
        gs.add_edge(Edge {
            source: 1081,
            target: 1081,
        });
        gs.add_edge(Edge {
            source: 957,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 959,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 962,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 1126,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 1306,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 1326,
            target: 962,
        });
        gs.add_edge(Edge {
            source: 964,
            target: 964,
        });
        gs.add_edge(Edge {
            source: 965,
            target: 965,
        });
        gs.add_edge(Edge {
            source: 975,
            target: 976,
        });
        gs.add_edge(Edge {
            source: 976,
            target: 976,
        });
        gs.add_edge(Edge {
            source: 978,
            target: 978,
        });
        gs.add_edge(Edge {
            source: 980,
            target: 982,
        });
        gs.add_edge(Edge {
            source: 982,
            target: 982,
        });
        gs.add_edge(Edge {
            source: 983,
            target: 983,
        });
        gs.add_edge(Edge {
            source: 1083,
            target: 1083,
        });
        gs.add_edge(Edge {
            source: 912,
            target: 912,
        });
        gs.add_edge(Edge {
            source: 914,
            target: 914,
        });
        gs.add_edge(Edge {
            source: 1125,
            target: 914,
        });
        gs.add_edge(Edge {
            source: 933,
            target: 935,
        });
        gs.add_edge(Edge {
            source: 935,
            target: 935,
        });
        gs.add_edge(Edge {
            source: 963,
            target: 935,
        });
        gs.add_edge(Edge {
            source: 938,
            target: 939,
        });
        gs.add_edge(Edge {
            source: 939,
            target: 939,
        });
        gs.add_edge(Edge {
            source: 940,
            target: 940,
        });
        gs.add_edge(Edge {
            source: 941,
            target: 941,
        });
        gs.add_edge(Edge {
            source: 937,
            target: 937,
        });
        gs.add_edge(Edge {
            source: 924,
            target: 926,
        });
        gs.add_edge(Edge {
            source: 926,
            target: 926,
        });
        gs.add_edge(Edge {
            source: 927,
            target: 927,
        });
        gs.add_edge(Edge {
            source: 931,
            target: 931,
        });
        gs.add_edge(Edge {
            source: 922,
            target: 922,
        });
        gs.add_edge(Edge {
            source: 1086,
            target: 1086,
        });
        gs.add_edge(Edge {
            source: 987,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 993,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1097,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1193,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1264,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1292,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1297,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 1312,
            target: 993,
        });
        gs.add_edge(Edge {
            source: 994,
            target: 994,
        });
        gs.add_edge(Edge {
            source: 1265,
            target: 994,
        });
        gs.add_edge(Edge {
            source: 1293,
            target: 994,
        });
        gs.add_edge(Edge {
            source: 995,
            target: 995,
        });
        gs.add_edge(Edge {
            source: 998,
            target: 999,
        });
        gs.add_edge(Edge {
            source: 999,
            target: 999,
        });
        gs.add_edge(Edge {
            source: 1327,
            target: 999,
        });
        gs.add_edge(Edge {
            source: 1003,
            target: 1003,
        });
        gs.add_edge(Edge {
            source: 1004,
            target: 950,
        });
        gs.add_edge(Edge {
            source: 936,
            target: 950,
        });
        gs.add_edge(Edge {
            source: 950,
            target: 950,
        });
        gs.add_edge(Edge {
            source: 1010,
            target: 952,
        });
        gs.add_edge(Edge {
            source: 952,
            target: 952,
        });
        gs.add_edge(Edge {
            source: 953,
            target: 953,
        });
        gs.add_edge(Edge {
            source: 955,
            target: 955,
        });
        gs.add_edge(Edge {
            source: 956,
            target: 956,
        });
        gs.add_edge(Edge {
            source: 1040,
            target: 956,
        });
        gs.add_edge(Edge {
            source: 1019,
            target: 1047,
        });
        gs.add_edge(Edge {
            source: 1046,
            target: 1047,
        });
        gs.add_edge(Edge {
            source: 1047,
            target: 1047,
        });
        gs.add_edge(Edge {
            source: 1092,
            target: 1047,
        });
        gs.add_edge(Edge {
            source: 1048,
            target: 1048,
        });
        gs.add_edge(Edge {
            source: 1049,
            target: 1050,
        });
        gs.add_edge(Edge {
            source: 1050,
            target: 1050,
        });
        gs.add_edge(Edge {
            source: 1054,
            target: 1000,
        });
        gs.add_edge(Edge {
            source: 996,
            target: 1000,
        });
        gs.add_edge(Edge {
            source: 1000,
            target: 1000,
        });
        gs.add_edge(Edge {
            source: 1001,
            target: 1001,
        });
        gs.add_edge(Edge {
            source: 1002,
            target: 1002,
        });
        gs.add_edge(Edge {
            source: 1042,
            target: 1042,
        });
        gs.add_edge(Edge {
            source: 1078,
            target: 1078,
        });
        gs.add_edge(Edge {
            source: 919,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1058,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1068,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1005,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1167,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1203,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1298,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1313,
            target: 1005,
        });
        gs.add_edge(Edge {
            source: 1007,
            target: 1007,
        });
        gs.add_edge(Edge {
            source: 1008,
            target: 1008,
        });
        gs.add_edge(Edge {
            source: 1015,
            target: 1015,
        });
        gs.add_edge(Edge {
            source: 1020,
            target: 1020,
        });
        gs.add_edge(Edge {
            source: 1071,
            target: 1022,
        });
        gs.add_edge(Edge {
            source: 1022,
            target: 1022,
        });
        gs.add_edge(Edge {
            source: 1073,
            target: 918,
        });
        gs.add_edge(Edge {
            source: 913,
            target: 918,
        });
        gs.add_edge(Edge {
            source: 918,
            target: 918,
        });
        gs.add_edge(Edge {
            source: 920,
            target: 920,
        });
        gs.add_edge(Edge {
            source: 921,
            target: 921,
        });
        gs.add_edge(Edge {
            source: 1076,
            target: 923,
        });
        gs.add_edge(Edge {
            source: 923,
            target: 923,
        });
        gs.add_edge(Edge {
            source: 1060,
            target: 923,
        });
        gs.add_edge(Edge {
            source: 928,
            target: 928,
        });
        gs.add_edge(Edge {
            source: 1023,
            target: 1023,
        });
        gs.add_edge(Edge {
            source: 1077,
            target: 930,
        });
        gs.add_edge(Edge {
            source: 1079,
            target: 930,
        });
        gs.add_edge(Edge {
            source: 930,
            target: 930,
        });
        gs.add_edge(Edge {
            source: 932,
            target: 932,
        });
        gs.add_edge(Edge {
            source: 1025,
            target: 1025,
        });
        gs.add_edge(Edge {
            source: 1026,
            target: 1026,
        });
        gs.add_edge(Edge {
            source: 1028,
            target: 1028,
        });
        gs.add_edge(Edge {
            source: 1029,
            target: 1029,
        });
        gs.add_edge(Edge {
            source: 1063,
            target: 1030,
        });
        gs.add_edge(Edge {
            source: 1030,
            target: 1030,
        });
        gs.add_edge(Edge {
            source: 1066,
            target: 1031,
        });
        gs.add_edge(Edge {
            source: 1031,
            target: 1031,
        });
        gs.add_edge(Edge {
            source: 1032,
            target: 1032,
        });
        gs.add_edge(Edge {
            source: 1033,
            target: 1033,
        });
        gs.add_edge(Edge {
            source: 1034,
            target: 1034,
        });
        gs.add_edge(Edge {
            source: 1035,
            target: 1035,
        });
        gs.add_edge(Edge {
            source: 925,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1082,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1084,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1036,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1129,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1208,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1299,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1314,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1328,
            target: 1036,
        });
        gs.add_edge(Edge {
            source: 1037,
            target: 1037,
        });
        gs.add_edge(Edge {
            source: 1088,
            target: 1038,
        });
        gs.add_edge(Edge {
            source: 1038,
            target: 1038,
        });
        gs.add_edge(Edge {
            source: 1039,
            target: 1039,
        });
        gs.add_edge(Edge {
            source: 1052,
            target: 1039,
        });
        gs.add_edge(Edge {
            source: 1043,
            target: 1043,
        });
        gs.add_edge(Edge {
            source: 1091,
            target: 1053,
        });
        gs.add_edge(Edge {
            source: 1053,
            target: 1053,
        });
        gs.add_edge(Edge {
            source: 1158,
            target: 1053,
        });
        gs.add_edge(Edge {
            source: 1095,
            target: 1055,
        });
        gs.add_edge(Edge {
            source: 1055,
            target: 1055,
        });
        gs.add_edge(Edge {
            source: 1056,
            target: 1056,
        });
        gs.add_edge(Edge {
            source: 1057,
            target: 1057,
        });
        gs.add_edge(Edge {
            source: 1059,
            target: 1059,
        });
        gs.add_edge(Edge {
            source: 1061,
            target: 1061,
        });
        gs.add_edge(Edge {
            source: 929,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 943,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1099,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1120,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1122,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1090,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1209,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1300,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1315,
            target: 1090,
        });
        gs.add_edge(Edge {
            source: 1093,
            target: 1093,
        });
        gs.add_edge(Edge {
            source: 1094,
            target: 1094,
        });
        gs.add_edge(Edge {
            source: 1116,
            target: 1067,
        });
        gs.add_edge(Edge {
            source: 1065,
            target: 1067,
        });
        gs.add_edge(Edge {
            source: 1067,
            target: 1067,
        });
        gs.add_edge(Edge {
            source: 1329,
            target: 1067,
        });
        gs.add_edge(Edge {
            source: 1096,
            target: 1096,
        });
        gs.add_edge(Edge {
            source: 1069,
            target: 1069,
        });
        gs.add_edge(Edge {
            source: 1098,
            target: 1098,
        });
        gs.add_edge(Edge {
            source: 1124,
            target: 1100,
        });
        gs.add_edge(Edge {
            source: 1100,
            target: 1100,
        });
        gs.add_edge(Edge {
            source: 1101,
            target: 1101,
        });
        gs.add_edge(Edge {
            source: 1103,
            target: 1103,
        });
        gs.add_edge(Edge {
            source: 1105,
            target: 1105,
        });
        gs.add_edge(Edge {
            source: 907,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 947,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1021,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1041,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1134,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1149,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1160,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1182,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1114,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1118,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1211,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1224,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1232,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1267,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1270,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1301,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1316,
            target: 1118,
        });
        gs.add_edge(Edge {
            source: 1121,
            target: 1121,
        });
        gs.add_edge(Edge {
            source: 1183,
            target: 1024,
        });
        gs.add_edge(Edge {
            source: 1018,
            target: 1024,
        });
        gs.add_edge(Edge {
            source: 1024,
            target: 1024,
        });
        gs.add_edge(Edge {
            source: 1148,
            target: 1148,
        });
        gs.add_edge(Edge {
            source: 1027,
            target: 1027,
        });
        gs.add_edge(Edge {
            source: 1152,
            target: 1152,
        });
        gs.add_edge(Edge {
            source: 1150,
            target: 1150,
        });
        gs.add_edge(Edge {
            source: 1151,
            target: 1151,
        });
        gs.add_edge(Edge {
            source: 1153,
            target: 1153,
        });
        gs.add_edge(Edge {
            source: 1154,
            target: 1154,
        });
        gs.add_edge(Edge {
            source: 1155,
            target: 1155,
        });
        gs.add_edge(Edge {
            source: 911,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 910,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 915,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1127,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1185,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1189,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1192,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1227,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1269,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1302,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1317,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1321,
            target: 1127,
        });
        gs.add_edge(Edge {
            source: 1156,
            target: 1156,
        });
        gs.add_edge(Edge {
            source: 1128,
            target: 1131,
        });
        gs.add_edge(Edge {
            source: 1131,
            target: 1131,
        });
        gs.add_edge(Edge {
            source: 1011,
            target: 1012,
        });
        gs.add_edge(Edge {
            source: 1012,
            target: 1012,
        });
        gs.add_edge(Edge {
            source: 1013,
            target: 1013,
        });
        gs.add_edge(Edge {
            source: 1157,
            target: 1157,
        });
        gs.add_edge(Edge {
            source: 1159,
            target: 1159,
        });
        gs.add_edge(Edge {
            source: 1235,
            target: 1159,
        });
        gs.add_edge(Edge {
            source: 1245,
            target: 1159,
        });
        gs.add_edge(Edge {
            source: 1271,
            target: 1159,
        });
        gs.add_edge(Edge {
            source: 1274,
            target: 1159,
        });
        gs.add_edge(Edge {
            source: 1197,
            target: 1161,
        });
        gs.add_edge(Edge {
            source: 1161,
            target: 1161,
        });
        gs.add_edge(Edge {
            source: 1162,
            target: 1162,
        });
        gs.add_edge(Edge {
            source: 1226,
            target: 1162,
        });
        gs.add_edge(Edge {
            source: 1268,
            target: 1162,
        });
        gs.add_edge(Edge {
            source: 1200,
            target: 1163,
        });
        gs.add_edge(Edge {
            source: 1163,
            target: 1163,
        });
        gs.add_edge(Edge {
            source: 1164,
            target: 1164,
        });
        gs.add_edge(Edge {
            source: 1165,
            target: 1165,
        });
        gs.add_edge(Edge {
            source: 1166,
            target: 1166,
        });
        gs.add_edge(Edge {
            source: 917,
            target: 1168,
        });
        gs.add_edge(Edge {
            source: 1168,
            target: 1168,
        });
        gs.add_edge(Edge {
            source: 1240,
            target: 1168,
        });
        gs.add_edge(Edge {
            source: 1272,
            target: 1168,
        });
        gs.add_edge(Edge {
            source: 1322,
            target: 1168,
        });
        gs.add_edge(Edge {
            source: 1169,
            target: 1169,
        });
        gs.add_edge(Edge {
            source: 1242,
            target: 1169,
        });
        gs.add_edge(Edge {
            source: 1273,
            target: 1169,
        });
        gs.add_edge(Edge {
            source: 1202,
            target: 1170,
        });
        gs.add_edge(Edge {
            source: 1170,
            target: 1170,
        });
        gs.add_edge(Edge {
            source: 1171,
            target: 1171,
        });
        gs.add_edge(Edge {
            source: 1204,
            target: 1172,
        });
        gs.add_edge(Edge {
            source: 1172,
            target: 1172,
        });
        gs.add_edge(Edge {
            source: 1307,
            target: 1172,
        });
        gs.add_edge(Edge {
            source: 1205,
            target: 1074,
        });
        gs.add_edge(Edge {
            source: 1070,
            target: 1074,
        });
        gs.add_edge(Edge {
            source: 1074,
            target: 1074,
        });
        gs.add_edge(Edge {
            source: 1075,
            target: 1075,
        });
        gs.add_edge(Edge {
            source: 1080,
            target: 1080,
        });
        gs.add_edge(Edge {
            source: 1206,
            target: 1085,
        });
        gs.add_edge(Edge {
            source: 1085,
            target: 1085,
        });
        gs.add_edge(Edge {
            source: 1087,
            target: 1087,
        });
        gs.add_edge(Edge {
            source: 1089,
            target: 1089,
        });
        gs.add_edge(Edge {
            source: 1173,
            target: 1173,
        });
        gs.add_edge(Edge {
            source: 1175,
            target: 1175,
        });
        gs.add_edge(Edge {
            source: 1176,
            target: 1176,
        });
        gs.add_edge(Edge {
            source: 949,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1207,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1106,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1123,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1215,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1247,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1250,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1251,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1254,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1275,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1278,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1279,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1282,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1303,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1318,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1330,
            target: 1106,
        });
        gs.add_edge(Edge {
            source: 1107,
            target: 1107,
        });
        gs.add_edge(Edge {
            source: 1210,
            target: 1108,
        });
        gs.add_edge(Edge {
            source: 1212,
            target: 1108,
        });
        gs.add_edge(Edge {
            source: 1108,
            target: 1108,
        });
        gs.add_edge(Edge {
            source: 1109,
            target: 1109,
        });
        gs.add_edge(Edge {
            source: 969,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1213,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1115,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1248,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1276,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1323,
            target: 1115,
        });
        gs.add_edge(Edge {
            source: 1214,
            target: 1117,
        });
        gs.add_edge(Edge {
            source: 1216,
            target: 1117,
        });
        gs.add_edge(Edge {
            source: 1117,
            target: 1117,
        });
        gs.add_edge(Edge {
            source: 1249,
            target: 1117,
        });
        gs.add_edge(Edge {
            source: 1277,
            target: 1117,
        });
        gs.add_edge(Edge {
            source: 1119,
            target: 1119,
        });
        gs.add_edge(Edge {
            source: 1130,
            target: 1130,
        });
        gs.add_edge(Edge {
            source: 1132,
            target: 1132,
        });
        gs.add_edge(Edge {
            source: 1217,
            target: 1133,
        });
        gs.add_edge(Edge {
            source: 1133,
            target: 1133,
        });
        gs.add_edge(Edge {
            source: 1252,
            target: 1133,
        });
        gs.add_edge(Edge {
            source: 1280,
            target: 1133,
        });
        gs.add_edge(Edge {
            source: 1135,
            target: 1135,
        });
        gs.add_edge(Edge {
            source: 1253,
            target: 1135,
        });
        gs.add_edge(Edge {
            source: 1281,
            target: 1135,
        });
        gs.add_edge(Edge {
            source: 1136,
            target: 1136,
        });
        gs.add_edge(Edge {
            source: 1137,
            target: 1137,
        });
        gs.add_edge(Edge {
            source: 1138,
            target: 1138,
        });
        gs.add_edge(Edge {
            source: 1219,
            target: 1139,
        });
        gs.add_edge(Edge {
            source: 1139,
            target: 1139,
        });
        gs.add_edge(Edge {
            source: 1308,
            target: 1139,
        });
        gs.add_edge(Edge {
            source: 1220,
            target: 1104,
        });
        gs.add_edge(Edge {
            source: 1102,
            target: 1104,
        });
        gs.add_edge(Edge {
            source: 1104,
            target: 1104,
        });
        gs.add_edge(Edge {
            source: 1110,
            target: 1110,
        });
        gs.add_edge(Edge {
            source: 1111,
            target: 1111,
        });
        gs.add_edge(Edge {
            source: 1112,
            target: 1112,
        });
        gs.add_edge(Edge {
            source: 1113,
            target: 1113,
        });
        gs.add_edge(Edge {
            source: 1221,
            target: 1140,
        });
        gs.add_edge(Edge {
            source: 1140,
            target: 1140,
        });
        gs.add_edge(Edge {
            source: 1141,
            target: 1141,
        });
        gs.add_edge(Edge {
            source: 1142,
            target: 1142,
        });
        gs.add_edge(Edge {
            source: 1143,
            target: 1143,
        });
        gs.add_edge(Edge {
            source: 1144,
            target: 1144,
        });
        gs.add_edge(Edge {
            source: 1145,
            target: 1145,
        });
        gs.add_edge(Edge {
            source: 1146,
            target: 1146,
        });
        gs.add_edge(Edge {
            source: 1147,
            target: 1147,
        });
        gs.add_edge(Edge {
            source: 988,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 997,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1051,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1064,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1223,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1178,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1218,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1255,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1258,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1283,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1286,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1304,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1319,
            target: 1178,
        });
        gs.add_edge(Edge {
            source: 1179,
            target: 1179,
        });
        gs.add_edge(Edge {
            source: 1225,
            target: 968,
        });
        gs.add_edge(Edge {
            source: 966,
            target: 968,
        });
        gs.add_edge(Edge {
            source: 968,
            target: 968,
        });
        gs.add_edge(Edge {
            source: 970,
            target: 970,
        });
        gs.add_edge(Edge {
            source: 1180,
            target: 1180,
        });
        gs.add_edge(Edge {
            source: 1181,
            target: 1181,
        });
        gs.add_edge(Edge {
            source: 1006,
            target: 971,
        });
        gs.add_edge(Edge {
            source: 1228,
            target: 971,
        });
        gs.add_edge(Edge {
            source: 1229,
            target: 971,
        });
        gs.add_edge(Edge {
            source: 971,
            target: 971,
        });
        gs.add_edge(Edge {
            source: 972,
            target: 972,
        });
        gs.add_edge(Edge {
            source: 1230,
            target: 973,
        });
        gs.add_edge(Edge {
            source: 973,
            target: 973,
        });
        gs.add_edge(Edge {
            source: 1257,
            target: 973,
        });
        gs.add_edge(Edge {
            source: 1285,
            target: 973,
        });
        gs.add_edge(Edge {
            source: 974,
            target: 974,
        });
        gs.add_edge(Edge {
            source: 1231,
            target: 960,
        });
        gs.add_edge(Edge {
            source: 1233,
            target: 960,
        });
        gs.add_edge(Edge {
            source: 1234,
            target: 960,
        });
        gs.add_edge(Edge {
            source: 958,
            target: 960,
        });
        gs.add_edge(Edge {
            source: 960,
            target: 960,
        });
        gs.add_edge(Edge {
            source: 961,
            target: 961,
        });
        gs.add_edge(Edge {
            source: 977,
            target: 977,
        });
        gs.add_edge(Edge {
            source: 979,
            target: 979,
        });
        gs.add_edge(Edge {
            source: 981,
            target: 981,
        });
        gs.add_edge(Edge {
            source: 1184,
            target: 1184,
        });
        gs.add_edge(Edge {
            source: 1009,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1236,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 984,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1256,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1284,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1309,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1325,
            target: 984,
        });
        gs.add_edge(Edge {
            source: 1238,
            target: 985,
        });
        gs.add_edge(Edge {
            source: 985,
            target: 985,
        });
        gs.add_edge(Edge {
            source: 1239,
            target: 986,
        });
        gs.add_edge(Edge {
            source: 986,
            target: 986,
        });
        gs.add_edge(Edge {
            source: 1331,
            target: 986,
        });
        gs.add_edge(Edge {
            source: 989,
            target: 989,
        });
        gs.add_edge(Edge {
            source: 1237,
            target: 991,
        });
        gs.add_edge(Edge {
            source: 991,
            target: 991,
        });
        gs.add_edge(Edge {
            source: 992,
            target: 992,
        });
        gs.add_edge(Edge {
            source: 1186,
            target: 1186,
        });
        gs.add_edge(Edge {
            source: 1014,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1072,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1241,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1044,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1045,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1222,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1259,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1262,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1287,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1290,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1305,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1320,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1332,
            target: 1045,
        });
        gs.add_edge(Edge {
            source: 1187,
            target: 1187,
        });
        gs.add_edge(Edge {
            source: 1188,
            target: 1188,
        });
        gs.add_edge(Edge {
            source: 1190,
            target: 1190,
        });
        gs.add_edge(Edge {
            source: 1017,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1243,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1191,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1260,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1288,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1324,
            target: 1191,
        });
        gs.add_edge(Edge {
            source: 1244,
            target: 1194,
        });
        gs.add_edge(Edge {
            source: 1194,
            target: 1194,
        });
        gs.add_edge(Edge {
            source: 1261,
            target: 1194,
        });
        gs.add_edge(Edge {
            source: 1289,
            target: 1194,
        });
        gs.add_edge(Edge {
            source: 1333,
            target: 1194,
        });
        gs.add_edge(Edge {
            source: 1195,
            target: 1195,
        });
        gs.add_edge(Edge {
            source: 1246,
            target: 1196,
        });
        gs.add_edge(Edge {
            source: 1196,
            target: 1196,
        });
        gs.add_edge(Edge {
            source: 1198,
            target: 1198,
        });
        gs.add_edge(Edge {
            source: 1199,
            target: 1199,
        });
        gs.add_edge(Edge {
            source: 1201,
            target: 1201,
        });

        gs.calculate_statistics();
        assert_eq!(true, gs.get_statistics().is_some());
        let stats = gs.get_statistics().unwrap();
        assert_eq!(true, stats.cyclic);
    }
}
