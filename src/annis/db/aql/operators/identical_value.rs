use crate::annis::db::{Graph, Match};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKeyID, Component};
use std;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalValueSpec {
    pub key : AnnoKeyID
}

impl BinaryOperatorSpec for IdenticalValueSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        vec![]
    }

    fn create_operator(&self, _db: &Graph) -> Option<Box<BinaryOperator>> {
        Some(Box::new(IdenticalValue {}))
    }
}

#[derive(Clone, Debug)]
pub struct IdenticalValue;

impl std::fmt::Display for IdenticalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "==")
    }
}

impl BinaryOperator for IdenticalValue {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        Box::new(std::iter::once(Match {
            node: lhs.node,
            anno_key: AnnoKeyID::default(),
        }))
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        lhs.node == rhs.node
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::MIN
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        Some(Box::new(self.clone()))
    }
}
