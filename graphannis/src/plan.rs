use Match;
use graphdb::GraphDB;
use query::conjunction;
use query::disjunction::Disjunction;
use exec::{ExecutionNode, Desc};
use std;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    ImpossibleSearch(Vec<conjunction::Error>),
}

pub struct ExecutionPlan<'a> {
    plans : Vec<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    current_plan: usize,
    descriptions : Vec<Option<Desc>>,
}

impl<'a> ExecutionPlan<'a> {
    
    pub fn from_disjunction(mut query : Disjunction<'a>, db : &'a GraphDB) -> Result<ExecutionPlan<'a>, Error> {
        let mut plans : Vec<Box<ExecutionNode<Item=Vec<Match>>+'a>> = Vec::new();
        let mut descriptions : Vec<Option<Desc>> = Vec::new();
        let mut errors : Vec<conjunction::Error> = Vec::new();
        for alt in query.alternatives.drain(..) {
            let p = alt.make_exec_node(db);
            if let Ok(p) = p {
                descriptions.push(p.get_desc().cloned());
                plans.push(p);
            } else if let Err(e) = p {
                errors.push(e);
            }
        }

        if plans.is_empty() {
            return Err(Error::ImpossibleSearch(errors));
        } else {
            return Ok(ExecutionPlan {plans, current_plan: 0, descriptions});
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
        while self.current_plan < self.plans.len() {
            let n = self.plans[self.current_plan].next();
            if let Some(tmp) = n {
                if let Some(ref desc) = self.descriptions[self.current_plan] {
                    let desc : &Desc = desc;
                    // re-order the matched nodes by the original node position of the query
                    let mut result : Vec<Match> = Vec::new();
                    result.reserve(tmp.len());
                    for i in 0..tmp.len() {
                        if let Some(mapped_pos) = desc.node_pos.get(&i) {
                            result.push(tmp[mapped_pos.clone()].clone());
                        } else {
                            result.push(tmp[i].clone());
                        }
                    }
                    return Some(result);
                } else {
                    return Some(tmp);
                }
            } else {
                self.current_plan += 1;
            } 
        }
        // all possible plans exhausted
        return None;
    }
}
