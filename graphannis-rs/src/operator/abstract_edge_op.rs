use {Annotation, Component, Match, NodeID, Edge};
use graphstorage::GraphStorage;
use graphdb::GraphDB;
use operator::{OperatorSpec, Operator};
use util;
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

impl AbstractEdgeOp {
    fn check_edge_annotation(&self, gs : &GraphStorage, source : &NodeID, target : &NodeID) -> bool {
        if self.spec.edge_anno.is_none() {
            return true;
        }

        let anno_template : &Annotation = self.spec.edge_anno.as_ref().unwrap();
        if anno_template.val == 0 || anno_template.val == <NodeID>::max_value() {
            // must be a valid value
            return false;
        } else {
            // check if the edge has the correct annotation first
            for a in gs.get_edge_annos(&Edge {source: source.clone(), target: target.clone()}) {
                if util::check_annotation_equal(&anno_template, &a) {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Operator for AbstractEdgeOp {

    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {
        unimplemented!()
        
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        for e in self.gs.iter() {
            if e.is_connected(&lhs.node, &rhs.node, self.spec.min_dist, self.spec.max_dist) {
                if self.check_edge_annotation(e.as_ref(), &lhs.node, &rhs.node) {
                    return true;
                }
            }
        }
        return false;
    }
}