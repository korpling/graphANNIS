use std::rc::Rc;
use Match;

enum ExecutionNode {
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
        unimplemented!();
        /* match self.root {
            ExecutionNode::Join{it, ..} => it.next(),
            ExecutionNode::Base{it, ..} => it.next(),
        } */
    }
}