use Match;

pub struct Cost {
    pub output : usize,
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;

    fn get_lhs(&self) -> Option<&ExecutionNode<Item = Vec<Match>>> {
        None
    }
    fn get_rhs(&self) -> Option<&ExecutionNode<Item = Vec<Match>>> {
        None
    }

    fn get_cost(&self) -> Option<&Cost> {
        None
    }
}


pub struct ExecutionPlan {
    root: Box<ExecutionNode<Item = Vec<Match>>>,
}

impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = self.root.as_iter().next();
        // TODO: re-organize the match positions
        return n;
    }
}
