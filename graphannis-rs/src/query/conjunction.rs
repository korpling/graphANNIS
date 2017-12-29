use {Match};
use operator::OperatorSpec;
use std::iter::Iterator;

pub struct Conjunction {
    nodes : Vec<Box<Iterator<Item = Vec<Match>>>>,
    operators : Vec<Box<OperatorSpec>>,
}

impl Conjunction {
    pub fn new() -> Conjunction {
        Conjunction {
            nodes: vec![],
            operators: vec![],
        }
    }

    pub fn add_node(&mut self, node : Box<Iterator<Item = Vec<Match>>>) -> usize {
        let idx = self.nodes.len();

        // TODO allow wrapping with an "any node anno" search
        self.nodes.push(node);

        idx
    }
}