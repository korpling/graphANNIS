use std::rc::Rc;
use Match;

pub enum ExecutionNode {
    Join {
        it: Box<Iterator<Item = Vec<Match>>>,
        component: usize,
        lhs: Box<ExecutionNode>,
        rhs: Box<ExecutionNode>,
    },
    Base {
        it: Box<Iterator<Item = Vec<Match>>>,
        component: usize,
    },
}

impl ExecutionNode {
    pub fn new_base(search: Box<Iterator<Item = Vec<Match>>>, node_nr: usize) -> ExecutionNode {
        ExecutionNode::Base { it: search, component: node_nr }
    }

    pub fn join(lhs : Box<ExecutionNode>, rhs : Box<ExecutionNode>) -> ExecutionNode {
        unimplemented!();
    }
}

pub struct ExecutionPlan {
    root: ExecutionNode,
}

impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = match self.root {
            ExecutionNode::Join { ref mut it, .. } | ExecutionNode::Base { ref mut it, .. } => {
                it.next()
            }
        };
        // TODO: re-organize the match positions
        return n;
    }
}
