use super::*;
use annostorage::AnnoStorage;
use Edge;
use dfs::CycleSafeDFS;

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::Bound::*;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone)]
pub struct AdjacencyListStorage {
    edges: BTreeSet<Edge>,
    inverse_edges: BTreeSet<Edge>,
    annos: AnnoStorage<Edge>,
}

impl AdjacencyListStorage {
    pub fn new() -> AdjacencyListStorage {
        AdjacencyListStorage {
            edges: BTreeSet::new(),
            inverse_edges: BTreeSet::new(),
            annos: AnnoStorage::new(),
        }
    }
}

impl EdgeContainer for AdjacencyListStorage {
    fn get_outgoing_edges(&self, source : &NodeID) -> Vec<NodeID> {
        let start_key = Edge{source: source.clone(), target: NodeID::min_value()};
        let end_key = Edge {source: source.clone(), target: NodeID::max_value()};

        Vec::from_iter(
            self.edges.range(start_key..end_key)
            .map(|e| {
                e.target
            })
        )
    }

    fn get_edge_annos(&self, edge : &Edge) -> Vec<Annotation> {
        self.annos.get_all(edge)
    }
}

impl ReadableGraphStorage for AdjacencyListStorage {
    fn find_connected<'a>(
        &'a self,
        source: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {

        let it = CycleSafeDFS::<'a>::new(self, source, min_distance, max_distance)
            .map(|x| {x.0})
            .scan(HashSet::<NodeID>::new(), |visited, n| {
                match visited.insert(n) {
                    true => Some(n),
                    false => None,
                }
            });
        Box::new(it)
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize> {
        let mut it = CycleSafeDFS::new(self, source, usize::min_value(), usize::max_value())
        .filter(|x| *target == x.0 )
        .map(|x| x.1);

        return it.next();

    }
    fn is_connected(&self, source: &NodeID, target: &NodeID, min_distance: usize, max_distance: usize) -> bool {
        let mut it = CycleSafeDFS::new(self, source, min_distance, max_distance)
        .filter(|x| *target == x.0 );
        
        return it.next().is_some();
    }

    fn copy(&mut self, other : &ReadableGraphStorage) {
        unimplemented!();
    }

    fn as_graphstorage(&self) -> &GraphStorage {
        return self;
    }
}

impl WriteableGraphStorage for AdjacencyListStorage {
    fn add_edge(&mut self, edge: Edge) {
        if edge.source != edge.target {
            self.inverse_edges.insert(edge.inverse());
            self.edges.insert(edge);
            // TODO: invalid graph statistics
        }
    }
    fn add_edge_annotation(&mut self, edge: Edge, anno: Annotation) {
        if self.edges.contains(&edge) {
            self.annos.insert(edge, anno);
        }
    }

    fn delete_edge(&mut self, edge: &Edge) {
        self.edges.remove(edge);
        self.inverse_edges.remove(&edge.inverse());

        let annos = self.annos.get_all(edge);
        for a in annos {
            self.annos.remove(edge, &a.key);
        }
    }
    fn delete_edge_annotation(&mut self, edge: &Edge, anno_key: &AnnoKey) {
        self.annos.remove(edge, anno_key);
    }
    fn delete_node(&mut self, node: &NodeID) {
        // find all both ingoing and outgoing edges
        let mut to_delete = std::collections::LinkedList::<Edge>::new();

        let range_start = Edge {
            source: node.clone(),
            target: NodeID::min_value(),
        };
        let range_end = Edge {
            source: node.clone(),
            target: NodeID::max_value(),
        };

        for e in self.edges
            .range((Included(range_start.clone()), Included(range_end.clone())))
        {
            to_delete.push_back(e.clone());
        }

        for e in self.inverse_edges
            .range((Included(range_start), Included(range_end)))
        {
            to_delete.push_back(e.clone());
        }

        for e in to_delete {
            self.delete_edge(&e);
        }
    }
}

impl GraphStorage for AdjacencyListStorage {
    fn as_readable(&self) -> &ReadableGraphStorage {self}
    fn as_writeable(&mut self) -> Option<&mut WriteableGraphStorage> {Some(self)}
}


#[cfg(test)]
mod tests {

    use super::*;

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

        assert_eq!(vec![2, 3], gs.get_outgoing_edges(&1));
        assert_eq!(vec![4,5], gs.get_outgoing_edges(&3));
        assert_eq!(0, gs.get_outgoing_edges(&6).len());
        assert_eq!(vec![4], gs.get_outgoing_edges(&2));

        let mut reachable : Vec<NodeID> = gs.find_connected(&1, 1, 100).collect();
        reachable.sort();
        assert_eq!(vec![2,3,4,5,6,7], reachable);

        let mut reachable : Vec<NodeID> = gs.find_connected(&3, 2, 100).collect();
        reachable.sort();
        assert_eq!(vec![6,7], reachable);

        let mut reachable : Vec<NodeID> = gs.find_connected(&1, 2, 4).collect();
        reachable.sort();
        assert_eq!(vec![4,5,6,7], reachable);

        let reachable : Vec<NodeID> = gs.find_connected(&7, 1, 100).collect();
        assert_eq!(true, reachable.is_empty());

        
    }

}
