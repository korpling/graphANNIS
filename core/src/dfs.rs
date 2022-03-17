use crate::{errors::Result, graph::storage::EdgeContainer, types::NodeID};
use rustc_hash::FxHashSet;

pub struct CycleSafeDFS<'a> {
    min_distance: usize,
    max_distance: usize,
    inverse: bool,
    container: &'a dyn EdgeContainer,

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
        container: &'a dyn EdgeContainer,
        node: NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> CycleSafeDFS<'a> {
        CycleSafeDFS {
            min_distance,
            max_distance,
            inverse: false,
            container,
            stack: vec![(node, 0)],
            path: Vec::default(),
            nodes_in_path: FxHashSet::default(),
            last_distance: 0,
            cycle_detected: false,
        }
    }

    pub fn new_inverse(
        container: &'a dyn EdgeContainer,
        node: NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> CycleSafeDFS<'a> {
        CycleSafeDFS {
            min_distance,
            max_distance,
            inverse: true,
            container,
            stack: vec![(node, 0)],
            path: Vec::default(),
            nodes_in_path: FxHashSet::default(),
            last_distance: 0,
            cycle_detected: true,
        }
    }

    pub fn is_cyclic(&self) -> bool {
        self.cycle_detected
    }

    fn enter_node(&mut self, entry: (NodeID, usize)) -> Result<bool> {
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
            trace!("removing from stack because of cycle");
            self.stack.pop();
            Ok(false)
        } else {
            self.path.push(node);
            self.nodes_in_path.insert(node);
            self.last_distance = dist;
            trace!("removing from stack");
            self.stack.pop();

            // check if distance is in valid range
            let found = dist >= self.min_distance && dist <= self.max_distance;

            if dist < self.max_distance {
                if self.inverse {
                    // add all parent nodes to the stack
                    for o in self.container.get_ingoing_edges(node) {
                        let o = o?;
                        self.stack.push((o, dist + 1));
                        trace!("adding {} to stack with new size {}", o, self.stack.len());
                    }
                } else {
                    // add all child nodes to the stack
                    for o in self.container.get_outgoing_edges(node) {
                        let o = o?;
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
            Ok(found)
        }
    }
}

impl<'a> Iterator for CycleSafeDFS<'a> {
    type Item = Result<DFSStep>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result: Option<Result<DFSStep>> = None;
        while result.is_none() && !self.stack.is_empty() {
            let top = *self.stack.last().unwrap();

            match self.enter_node(top) {
                Ok(entered) => {
                    if entered {
                        result = Some(Ok(DFSStep {
                            node: top.0,
                            distance: top.1,
                        }));
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }

        result
    }
}
