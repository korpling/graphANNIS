use {Match};
use operator::OperatorSpec;
use super::disjunction::Disjunction;
use std::iter::Iterator;

struct OperatorEntry {
    op : Box<OperatorSpec>,
    idx_left : usize,
    idx_right : usize,
    original_order : usize,
}

pub struct Conjunction {
    nodes : Vec<Box<Iterator<Item = Vec<Match>>>>,
    operators : Vec<OperatorEntry>,
}

impl Conjunction {
    pub fn new() -> Conjunction {
        Conjunction {
            nodes: vec![],
            operators: vec![],
        }
    }

    pub fn into_disjunction(self) -> Disjunction {
        Disjunction::new(self)
    }

    pub fn add_node(&mut self, node : Box<Iterator<Item = Vec<Match>>>) -> usize {
        let idx = self.nodes.len();

        // TODO allow wrapping with an "any node anno" search
        self.nodes.push(node);

        idx
    }

    pub fn add_operator(&mut self, op : Box<OperatorSpec>, idx_left : usize, idx_right : usize) {
        let original_order = self.operators.len();
        self.operators.push(OperatorEntry {
            op,
            idx_left,
            idx_right,
            original_order,
        });

    }
}