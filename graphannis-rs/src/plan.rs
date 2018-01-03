use Match;
use graphdb::GraphDB;
use query::disjunction::Disjunction;
use exec::ExecutionNode;

pub enum Error {
    ImpossiblePlan,
}

pub struct ExecutionPlan<'a> {
    root: Box<Iterator<Item = Vec<Match>> + 'a>,
}

impl<'a> ExecutionPlan<'a> {
    
    pub fn from_disjunction(mut query : Disjunction<'a>, db : &'a GraphDB) -> Result<ExecutionPlan<'a>, Error> {
        let mut plans : Vec<Box<ExecutionNode<Item=Vec<Match>>+'a>> = Vec::new();
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

impl<'a> Iterator for ExecutionPlan<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = self.root.next();
        // TODO: re-organize the match positions
        return n;
    }
}
