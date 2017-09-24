use super::*;
use annis::annostorage::AnnoStorage;
use annis::Edge;

use std::collections::BTreeSet;
use std::collections::Bound::*;

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

impl ReadableGraphStorage for AdjacencyListStorage {
    fn find_connected(
        &self,
        source: &NodeID,
        min_distance: u32,
        max_distance: u32,
    ) -> Box<Iterator<Item = NodeID>> {
        unimplemented!();
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> u32 {
        unimplemented!();
    }
    fn is_connected(&self, source: &NodeID, target: &NodeID, min_distance: u32, max_distance: u32) {
        unimplemented!();
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
    }

}
