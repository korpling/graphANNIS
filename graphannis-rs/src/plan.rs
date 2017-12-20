use std::rc::Rc;
use Match;

pub enum ExecutionNode {
    Join {
        it: Box<Iterator<Item = Vec<Match>>>,
        lhs: Box<ExecutionNode>,
        rhs: Box<ExecutionNode>,
    },
    Base { it: Box<Iterator<Item = Vec<Match>>> },
}


pub struct ExecutionPlan {
    root: ExecutionNode,
}


impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = match self.root {
            ExecutionNode::Join{ref mut it, ..} => it.next(),
            ExecutionNode::Base{ref mut it, ..} => it.next(),
        };
        // TODO: re-organize the match positions
        return n;
    }
}