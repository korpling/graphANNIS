use super::*;

use crate::{
    annostorage::ondisk::AnnoStorageImpl,
    dfs::CycleSafeDFS,
    errors::Result,
    util::disk_collections::{DiskMap, EvictionStrategy, DEFAULT_BLOCK_CACHE_CAPACITY},
};
use itertools::Itertools;
use rustc_hash::FxHashSet;
use std::collections::BTreeSet;
use std::ops::Bound;
use transient_btree_index::BtreeConfig;

pub const SERIALIZATION_ID: &str = "DiskAdjacencyListV1";

/// Repeatedly call the given function and get the result, or panic if the error is permanent.
fn get_or_panic<F, R>(f: F) -> R
where
    F: Fn() -> Result<R>,
{
    let mut last_err = None;
    for _ in 0..5 {
        match f() {
            Ok(result) => return result,
            Err(e) => last_err = Some(e),
        }
        // In case this is an intermediate error, wait some time before trying again
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    panic!("Accessing the disk-database failed. This is a non-recoverable error since it means something serious is wrong with the disk or file system.\nCause:\n{:?}", last_err.unwrap())
}

#[derive(MallocSizeOf)]
pub struct DiskAdjacencyListStorage {
    #[ignore_malloc_size_of = "is stored on disk"]
    edges: DiskMap<Edge, bool>,
    #[ignore_malloc_size_of = "is stored on disk"]
    inverse_edges: DiskMap<Edge, bool>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

fn get_fan_outs(edges: &DiskMap<Edge, bool>) -> Vec<usize> {
    let mut fan_outs: Vec<usize> = Vec::new();
    if !get_or_panic(|| edges.is_empty()) {
        let it = get_or_panic(|| edges.iter());
        for (_, targets) in &it.group_by(|(e, _)| e.source) {
            fan_outs.push(targets.count());
        }
    }
    // order the fan-outs
    fan_outs.sort_unstable();

    fan_outs
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
    fn get_outgoing_edges<'a>(&'a self, node: NodeID) -> Box<dyn Iterator<Item = NodeID> + 'a> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::min_value(),
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::max_value(),
        };
        Box::new(
            self.edges
                .range(lower_bound..upper_bound)
                .map(|(e, _)| e.target),
        )
    }

    fn has_outgoing_edges(&self, node: NodeID) -> bool {
        let lower_bound = Edge {
            source: node,
            target: NodeID::min_value(),
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::max_value(),
        };
        self.edges.range(lower_bound..upper_bound).next().is_some()
    }

    fn get_ingoing_edges<'a>(&'a self, node: NodeID) -> Box<dyn Iterator<Item = NodeID> + 'a> {
        let lower_bound = Edge {
            source: node,
            target: NodeID::min_value(),
        };
        let upper_bound = Edge {
            source: node,
            target: NodeID::max_value(),
        };
        Box::new(
            self.inverse_edges
                .range(lower_bound..upper_bound)
                .map(|(e, _)| e.target),
        )
    }
    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = NodeID> + 'a> {
        let it = get_or_panic(|| self.edges.iter())
            .map(|(e, _)| e.source)
            .unique();

        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl GraphStorage for DiskAdjacencyListStorage {
    fn get_anno_storage(&self) -> &dyn AnnotationStorage<Edge> {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        SERIALIZATION_ID.to_owned()
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        // Read stats
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::open(&stats_path)?;
        let input = std::io::BufReader::new(f_stats);
        let stats = bincode::deserialize_from(input)?;

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
        // Write stats with bincode
        let stats_path = location.join("edge_stats.bin");
        let f_stats = std::fs::File::create(&stats_path)?;
        let mut writer = std::io::BufWriter::new(f_stats);
        bincode::serialize_into(&mut writer, &self.stats)?;

        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let it = CycleSafeDFS::<'a>::new(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn find_connected_inverse<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: Bound<usize>,
    ) -> Box<dyn Iterator<Item = NodeID> + 'a> {
        let mut visited = FxHashSet::<NodeID>::default();
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };

        let it = CycleSafeDFS::<'a>::new_inverse(self, node, min_distance, max_distance)
            .map(|x| x.node)
            .filter(move |n| visited.insert(*n));
        Box::new(it)
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Option<usize> {
        let mut it = CycleSafeDFS::new(self, source, usize::min_value(), usize::max_value())
            .filter(|x| target == x.node)
            .map(|x| x.distance);

        it.next()
    }
    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> bool {
        let max_distance = match max_distance {
            Bound::Unbounded => usize::max_value(),
            Bound::Included(max_distance) => max_distance,
            Bound::Excluded(max_distance) => max_distance + 1,
        };
        let mut it = CycleSafeDFS::new(self, source, min_distance, max_distance)
            .filter(|x| target == x.node);

        it.next().is_some()
    }

    fn copy(
        &mut self,
        _node_annos: &dyn AnnotationStorage<NodeID>,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.clear()?;

        for source in orig.source_nodes() {
            for target in orig.get_outgoing_edges(source) {
                let e = Edge { source, target };
                self.add_edge(e.clone())?;
                for a in orig.get_anno_storage().get_annotations_for_item(&e) {
                    self.add_edge_annotation(e.clone(), a)?;
                }
            }
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics();
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

        let annos = self.annos.get_annotations_for_item(edge);
        for a in annos {
            self.annos.remove_annotation_for_item(edge, &a.key)?;
        }

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
            to_delete.push_back(Edge {
                source: node,
                target,
            });
        }

        for source in self.get_ingoing_edges(node) {
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
            for (e, _) in get_or_panic(|| self.edges.iter()) {
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

        let edges_empty = get_or_panic(|| self.edges.is_empty());

        if !edges_empty {
            for (e, _) in get_or_panic(|| self.edges.iter()) {
                roots.remove(&e.target);
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
        if roots.is_empty() && !edges_empty {
            // if we have edges but no roots at all there must be a cycle
            stats.cyclic = true;
        } else {
            for root_node in &roots {
                let mut dfs = CycleSafeDFS::new(self, *root_node, 0, usize::max_value());
                for step in &mut dfs {
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

    fn clear(&mut self) -> Result<()> {
        self.annos.clear()?;
        self.edges.clear();
        self.inverse_edges.clear();
        self.stats = None;
        Ok(())
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

        let mut gs = DiskAdjacencyListStorage::new().unwrap();
        gs.add_edge(Edge {
            source: 1,
            target: 2,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 2,
            target: 3,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 3,
            target: 4,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1,
            target: 3,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 4,
            target: 5,
        })
        .unwrap();

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
        let mut gs = DiskAdjacencyListStorage::new().unwrap();

        gs.add_edge(Edge {
            source: 1,
            target: 2,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 2,
            target: 4,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1,
            target: 3,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 3,
            target: 5,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 5,
            target: 7,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 5,
            target: 6,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 3,
            target: 4,
        })
        .unwrap();

        assert_eq!(
            vec![2, 3],
            gs.get_outgoing_edges(1).sorted().collect::<Vec<NodeID>>()
        );
        assert_eq!(
            vec![4, 5],
            gs.get_outgoing_edges(3).sorted().collect::<Vec<NodeID>>()
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
        let mut gs = DiskAdjacencyListStorage::new().unwrap();

        gs.add_edge(Edge {
            source: 1,
            target: 2,
        })
        .unwrap();

        gs.add_edge(Edge {
            source: 2,
            target: 3,
        })
        .unwrap();

        gs.add_edge(Edge {
            source: 3,
            target: 4,
        })
        .unwrap();

        gs.add_edge(Edge {
            source: 4,
            target: 5,
        })
        .unwrap();

        gs.add_edge(Edge {
            source: 5,
            target: 2,
        })
        .unwrap();

        gs.calculate_statistics();
        assert_eq!(true, gs.get_statistics().is_some());
        let stats = gs.get_statistics().unwrap();
        assert_eq!(true, stats.cyclic);
    }

    #[test]
    fn multi_branch_cycle_statistics() {
        let mut gs = DiskAdjacencyListStorage::new().unwrap();

        gs.add_edge(Edge {
            source: 903,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 904,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1174,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1295,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1310,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1334,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1335,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1336,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1337,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1338,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1339,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1340,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1341,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1342,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1343,
            target: 1343,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 903,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 904,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1174,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1295,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1310,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1334,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1335,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1336,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1337,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1338,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1339,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1340,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1341,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1342,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1343,
            target: 1342,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 903,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 904,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1174,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1295,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1310,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1334,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1335,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1336,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1337,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1338,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1339,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1340,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1341,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1342,
            target: 1339,
        })
        .unwrap();
        gs.add_edge(Edge {
            source: 1343,
            target: 1339,
        })
        .unwrap();

        gs.calculate_statistics();
        assert_eq!(true, gs.get_statistics().is_some());
        let stats = gs.get_statistics().unwrap();
        assert_eq!(true, stats.cyclic);
    }
}
