use nodesearch::NodeSearch;
use operator::OperatorSpec;
use super::disjunction::Disjunction;

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
}