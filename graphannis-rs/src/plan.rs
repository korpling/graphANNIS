use Match;
use graphdb::GraphDB;
use query::disjunction::Disjunction;
use exec::ExecutionNode;

pub enum Error {
    ImpossiblePlan,
}

pub struct ExecutionPlan {
    root: Box<Iterator<Item = Vec<Match>>>,
}

impl ExecutionPlan {
    
    pub fn from_disjunction(mut query : Disjunction, db : &GraphDB) -> Result<ExecutionPlan, Error> {
        let mut plans : Vec<Box<ExecutionNode<Item=Vec<Match>>>> = Vec::new();
        for alt in query.alternatives.drain(..) {
            let p = alt.make_exec_node(db);
            if let Ok(p) = p {
                plans.push(p);
            }
        }

        if plans.is_empty() {
            return Err(Error::ImpossiblePlan);
        } else {
            let it = plans.into_iter().flat_map(|p| p);
            let box_it : Box<Iterator<Item = Vec<Match>>> = Box::new(it);
            return Ok(ExecutionPlan {root: box_it});
        }
    }
}

impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = self.root.next();
        // TODO: re-organize the match positions
        return n;
    }
}
