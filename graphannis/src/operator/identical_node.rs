use super::*;

use {Component, Annotation};
use graphdb::GraphDB;
use std;

#[derive(Debug,Clone)]
pub struct IdenticalNodeSpec;

impl OperatorSpec for IdenticalNodeSpec {
    fn necessary_components(&self) -> Vec<Component> {vec![]}

    fn create_operator<'a>(&self, _db: &'a GraphDB) -> Option<Box<Operator + 'a>> {
        Some(Box::new(IdenticalNode {}))
    }
   
}

#[derive(Debug)]
pub struct IdenticalNode;

impl std::fmt::Display for IdenticalNode {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_ident_")
    }
}

impl Operator for IdenticalNode {
    fn retrieve_matches<'a>(&'a self, lhs : &Match) -> Box<Iterator<Item = Match> + 'a> {
        return Box::new(std::iter::once(
            Match{node: lhs.node.clone(), anno: Annotation::default()}
            )
        );
    }

    fn filter_match(&self, lhs : &Match, rhs : &Match) -> bool {
        return lhs.node == rhs.node;
    }

    fn estimation_type<'a>(&self, _db: &'a GraphDB) -> EstimationType {EstimationType::MIN}
}