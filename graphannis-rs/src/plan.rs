use Match;

pub trait ExecutionNode {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>>;
}


pub struct ExecutionPlan {
    root: Box<ExecutionNode>,
}

impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = self.root.as_iter().next();
        // TODO: re-organize the match positions
        return n;
    }
}
