use Match;
use query::disjunction::Disjunction;
use query::conjunction::Conjunction;
use nodesearch::NodeSearch;

use std::collections::BTreeMap;

pub enum Error {
    ImpossiblePlan,
}

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr : usize,
    pub lhs: Option<Box<Desc>>,
    pub rhs: Option<Box<Desc>>,
    pub node_pos : BTreeMap<usize, usize>,
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


        let mut lhs = if let Some(d) = lhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        let mut rhs = if let Some(d) = rhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        // merge both node positions
        let mut node_pos = BTreeMap::new();
        let offset = if let Some(ref mut lhs) = lhs {
            node_pos.append(&mut lhs.node_pos);
            node_pos.len()
        } else {
            0
        };
        if let Some(ref mut rhs) = rhs {
            for e in rhs.node_pos.iter() {
                // the RHS has an offset after the join
                node_pos.insert(e.0.clone(), e.1 + offset);
            }
            // move values to new map, the old should be empty (as with the append() function)
            rhs.node_pos.clear();
        }


        Desc {
            component_nr,
            lhs,
            rhs,
            node_pos,
        }
    }
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;

    fn as_nodesearch(&self) -> Option<&NodeSearch> {
        None
    }

    fn get_desc(&self) -> Option<&Desc> {
        None
    }

}

pub struct ExecutionPlan {
    root: Box<Iterator<Item = Vec<Match>>>,
}

impl ExecutionPlan {
    
    pub fn from_disjunction(query : &Disjunction) -> Result<ExecutionPlan, Error> {
        let mut plans : Vec<Box<ExecutionNode<Item=Vec<Match>>>> = Vec::new();
        for alt in query.alternatives.iter() {
            let p = alt.make_exec_node();
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
