use {Annotation, Component, Match};
use graphstorage::GraphStorage;
use graphdb::GraphDB;
use operator::{OperatorSpec, Operator};
use std::rc::Rc;

#[derive(Clone)]
pub struct AbstractEdgeOpSpec {
    pub components: Vec<Component>,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno : Option<Annotation>,
}

pub struct AbstractEdgeOp {
    gs: Vec<Rc<GraphStorage>>,
    spec: AbstractEdgeOpSpec,
}

impl AbstractEdgeOp {
    pub fn new(db: &GraphDB, spec: AbstractEdgeOpSpec) -> Option<AbstractEdgeOp> {
     
        let mut gs : Vec<Rc<GraphStorage>> = Vec::new();
        for c in spec.components.iter() {
            gs.push(db.get_graphstorage(c)?);
        }
        Some(AbstractEdgeOp {
            gs,
            spec,
        })
    }
}

impl OperatorSpec for AbstractEdgeOpSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.components.clone()
    }

    fn create_operator<'b>(&self, db : &'b GraphDB) -> Option<Box<Operator + 'b>> {
        
        let optional_op = AbstractEdgeOp::new(db, self.clone());
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl Operator for AbstractEdgeOp {

    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {
        unimplemented!()
        
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        unimplemented!()
    }
}