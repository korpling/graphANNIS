use Match;

#[derive(Debug, Clone)]
pub struct Desc {
    pub output : usize,
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;

    fn get_lhs_desc(&self) -> Option<&Desc> {
        None
    }
    fn get_rhs_desc(&self) -> Option<&Desc> {
        None
    }

    fn get_desc(&self) -> Option<&Desc> {
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
