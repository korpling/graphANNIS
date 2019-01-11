use crate::annis::db::AnnoStorage;
use crate::annis::db::{Graph, Match};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKey, AnnoKeyID, Component, NodeID};
use std;
use std::sync::Arc;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum Type {
    TokenText,
    AnnotationValue,
}

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalValueSpec {
    pub lhs_type: Type,
    pub rhs_type: Type,
}

impl BinaryOperatorSpec for IdenticalValueSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        vec![]
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<BinaryOperator>> {
        Some(Box::new(IdenticalValue {
            node_annos: db.node_annos.clone(),
            lhs_type: self.lhs_type.clone(),
            rhs_type: self.rhs_type.clone(),
            tok_key: db.get_token_key(),
        }))
    }
}

#[derive(Clone)]
pub struct IdenticalValue {
    node_annos: Arc<AnnoStorage<NodeID>>,
    tok_key: AnnoKey,
    lhs_type: Type,
    rhs_type: Type,
}

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
        let lhs_val = match self.lhs_type {
            Type::AnnotationValue => {
                self.node_annos.get_value_for_item_by_id(&lhs.node, lhs.anno_key)
            } 
            Type::TokenText => {
                self.node_annos.get_value_for_item(&lhs.node, &self.tok_key)
            }
        };
        let rhs_val = match self.rhs_type {
            Type::AnnotationValue => {
                self.node_annos.get_value_for_item_by_id(&rhs.node, rhs.anno_key)
            } 
            Type::TokenText => {
                self.node_annos.get_value_for_item(&rhs.node, &self.tok_key)
            }
        };

        if let (Some(lhs_val), Some(rhs_val)) = (lhs_val, rhs_val) {
            lhs_val == rhs_val
        } else {
            false
        }
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::MIN
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        Some(Box::new(self.clone()))
    }
}
