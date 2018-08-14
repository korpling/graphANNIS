use aql::operators::edge_op::EdgeAnnoSearchSpec;
use {Component, Match};
use graphdb::GraphDB;
use std;

pub enum EstimationType {
    SELECTIVITY(f64),
    MAX,
    MIN,
}

pub trait Operator: std::fmt::Display + Send + Sync {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>>;

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool;

    fn is_reflexive(&self) -> bool {
        true
    }
    
    fn get_inverse_operator(&self) -> Option<Box<Operator>> {
        None
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::SELECTIVITY(0.1)
    }

    fn edge_anno_selectivity(&self) -> Option<f64> {
        None
    }
}

pub trait OperatorSpec: std::fmt::Debug {
    fn necessary_components(&self, db: &GraphDB) -> Vec<Component>;

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>>;

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        None
    }
}

