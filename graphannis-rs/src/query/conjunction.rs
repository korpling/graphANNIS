use {Match, NodeID};
use nodesearch::NodeSearch;
use operator::OperatorSpec;
use plan::ExecutionNode;
use super::disjunction::Disjunction;

use std::collections::BTreeMap;

pub enum Error {
    ImpossibleQuery,
}

struct OperatorEntry {
    op : Box<OperatorSpec>,
    idx_left : usize,
    idx_right : usize,
    original_order : usize,
}

pub struct Conjunction<'a> {
    nodes : Vec<NodeSearch<'a>>,
    operators : Vec<OperatorEntry>,
}

impl<'a> Conjunction<'a> {
    pub fn new() -> Conjunction<'a> {
        Conjunction {
            nodes: vec![],
            operators: vec![],
        }
    }

    pub fn into_disjunction(self) -> Disjunction<'a> {
        Disjunction::new(self)
    }

    pub fn add_node(&mut self, node : NodeSearch<'a>) -> usize {
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

    pub fn make_exec_node(&self) -> Result<Box<ExecutionNode<Item=Vec<Match>>>, Error> {
        let mut node2component : BTreeMap<NodeID, usize> = BTreeMap::new();
        let mut component2exec : BTreeMap<usize, Box<ExecutionNode<Item=Vec<Match>>>> = BTreeMap::new();

        // 1. add all nodes
        let mut i = 0;

        unimplemented!()
    }
}