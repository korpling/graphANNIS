use Match;
use graphdb::GraphDB;
use query::disjunction::Disjunction;
use exec::{ExecutionNode, Desc};
use std;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    ImpossibleSearch,
}

pub struct ExecutionPlan<'a> {
    root: Box<Iterator<Item = Vec<Match>> + 'a>,
    descriptions : Vec<Option<Desc>>,
}

impl<'a> ExecutionPlan<'a> {
    
    pub fn from_disjunction(mut query : Disjunction<'a>, db : &'a GraphDB) -> Result<ExecutionPlan<'a>, Error> {
        let mut plans : Vec<Box<ExecutionNode<Item=Vec<Match>>+'a>> = Vec::new();
        let mut descriptions : Vec<Option<Desc>> = Vec::new();
        for alt in query.alternatives.drain(..) {
            let p = alt.make_exec_node(db);
            if let Ok(p) = p {
                descriptions.push(p.get_desc().cloned());
                plans.push(p);
            }
        }

        if plans.is_empty() {
            return Err(Error::ImpossibleSearch);
        } else {
            let it = plans.into_iter().flat_map(|p| p);
            let box_it : Box<Iterator<Item = Vec<Match>>> = Box::new(it);
            return Ok(ExecutionPlan {root: box_it, descriptions});
        }
    }
}

impl<'a> std::fmt::Display for ExecutionPlan<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for d in self.descriptions.iter() {
            if let &Some(ref d) = d {
                write!(f, "{}", d.debug_string(""))?;
            } else {
                write!(f, "<no description>")?;
            }
        }
        Ok(())
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
