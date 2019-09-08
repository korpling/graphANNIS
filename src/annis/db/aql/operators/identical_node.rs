use crate::annis::db::{Graph, Match};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKeyID, Component};
use std;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalNodeSpec;

impl BinaryOperatorSpec for IdenticalNodeSpec {
    fn necessary_components(&self, _db: &Graph) -> HashSet<Component> {
        HashSet::default()
    }

    fn create_operator(&self, _db: &Graph) -> Option<Box<dyn BinaryOperator>> {
        Some(Box::new(IdenticalNode {}))
    }
}

#[derive(Clone, Debug)]
pub struct IdenticalNode;

impl std::fmt::Display for IdenticalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_ident_")
    }
}

impl BinaryOperator for IdenticalNode {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
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

    fn get_inverse_operator(&self) -> Option<Box<dyn BinaryOperator>> {
        Some(Box::new(self.clone()))
    }
}
