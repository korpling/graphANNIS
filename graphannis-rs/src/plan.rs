use Match;
use query::disjunction::Disjunction;
use query::conjunction::Conjunction;

pub enum Error {
    ImpossiblePlan,
}

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr : usize,
    pub lhs: Option<Box<Desc>>,
    pub rhs: Option<Box<Desc>>,
}

impl Desc {
    pub fn join(lhs : Option<&Desc>, rhs : Option<&Desc>) -> Desc {
        let component_nr = if let Some(d) = lhs  {
            d.component_nr
        } else if let Some(d) = rhs {
            d.component_nr
        } else {
            0
        };

        let lhs = if let Some(d) = lhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        let rhs = if let Some(d) = rhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        Desc {
            component_nr,
            lhs,
            rhs,
        }
    }
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;

    fn get_desc(&self) -> Option<&Desc> {
        None
    }
}



pub struct ExecutionPlan {
    root: Box<Iterator<Item = Vec<Match>>>,
}

impl ExecutionPlan {
    pub fn from_conjunction(query : &Conjunction) -> Result<ExecutionPlan, Error> {
        unimplemented!()
    }
    pub fn from_disjunction(query : &Disjunction) -> Result<ExecutionPlan, Error> {
        let mut plans : Vec<ExecutionPlan> = Vec::new();
        for alt in query.alternatives.iter() {
            let p = ExecutionPlan::from_conjunction(alt);
            if let Ok(p) = p {
                plans.push(p);
            }
        }

        if plans.is_empty() {
            return Err(Error::ImpossiblePlan);
        } else {
            let it = plans.into_iter().flat_map(|p| p.root);
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
