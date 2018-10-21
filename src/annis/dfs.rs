use annis::db::graphstorage::EdgeContainer;
use annis::types::NodeID;
use rustc_hash::FxHashSet;

pub struct CycleSafeDFS<'a> {
    min_distance: usize,
    max_distance: usize,
    inverse: bool,
    container: &'a EdgeContainer,

    stack: Vec<(NodeID, usize)>,
    path: Vec<NodeID>,
    nodes_in_path: FxHashSet<NodeID>,
    last_distance: usize,
    cycle_detected: bool,
}

pub struct DFSStep {
    pub node: NodeID,
    pub distance: usize,
}

impl<'a> CycleSafeDFS<'a> {
    pub fn new(
        container: &'a EdgeContainer,
        node: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> CycleSafeDFS<'a> {
        let mut stack = vec![];
        stack.push((node.clone(), 0));

        let path = vec![];
        let nodes_in_path = FxHashSet::default();

        CycleSafeDFS {
            min_distance,
            max_distance,
            inverse: false,
            container,
            stack,
            path,
            nodes_in_path,
            last_distance: 0,
            cycle_detected: false,
        }
    }

    pub fn new_inverse(
        container: &'a EdgeContainer,
        node: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> CycleSafeDFS<'a> {
        let mut stack = vec![];
        stack.push((node.clone(), 0));

        let path = vec![];
        let nodes_in_path = FxHashSet::default();

        CycleSafeDFS {
            min_distance,
            max_distance,
            inverse: true,
            container,
            stack,
            path,
            nodes_in_path,
            last_distance: 0,
            cycle_detected: true,
        }
    }

    pub fn is_cyclic(&self) -> bool {
        self.cycle_detected
    }

    fn enter_node(&mut self, entry: (NodeID, usize)) -> bool {
        let node = entry.0;
        let dist = entry.1;

        trace!("enter node {}", node);
        // test if subgraph was completed
        if self.last_distance >= dist {
            // remove all entries below the parent node from the path
            for i in dist..self.path.len() {
                trace!("truncating {} from path", &self.path[i]);
                self.nodes_in_path.remove(&self.path[i]);
            }
            self.path.truncate(dist);
        }
        // test for cycle
        if self.nodes_in_path.contains(&node) {
            trace!("cycle detected for node {} with distance {}", &node, dist);
            self.last_distance = dist;
            self.cycle_detected = true;
            false
        } else {
            self.path.push(node.clone());
            self.nodes_in_path.insert(node);
            self.last_distance = dist;
            trace!("removing from stack");
            self.stack.pop();

            // check if distance is in valid range
            let found = dist >= self.min_distance && dist <= self.max_distance;

            if dist < self.max_distance {
                if self.inverse {
                    // add all parent nodes to the stack
                    for o in self.container.get_ingoing_edges(&node) {
                        self.stack.push((o, dist + 1));
                        trace!("adding {} to stack with new size {}", o, self.stack.len());
                    }
                } else {
                    // add all child nodes to the stack
                    for o in self.container.get_outgoing_edges(&node) {
                        self.stack.push((o, dist + 1));
                        trace!("adding {} to stack with new size {}", o, self.stack.len());
                    }
                }
            }
            trace!(
                "enter_node finished with result {} for node {}",
                found,
                node
            );
            found
        }
    }
}

impl<'a> Iterator for CycleSafeDFS<'a> {
    type Item = DFSStep;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result: Option<DFSStep> = None;
        while result.is_none() && !self.stack.is_empty() {
            let top = self.stack.last().unwrap().clone();
            if self.enter_node(top) {
                result = Some(DFSStep {
                    node: top.0,
                    distance: top.1,
                });
            }
        }

        result
    }
}
