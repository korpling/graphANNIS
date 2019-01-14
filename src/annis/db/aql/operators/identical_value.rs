use crate::annis::db::{AnnoStorage, AnnotationStorage, Graph, Match, ValueSearch, ANNIS_NS, TOK};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKey, Component, NodeID};
use std;
use std::sync::Arc;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum Type {
    TokenText,
    AnnotationValue { ns: Option<String>, name: String },
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

    fn create_operator<'a>(&self, db: &Graph) -> Option<Box<dyn BinaryOperator>> {
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

impl IdenticalValue {
    fn value_for_match(&self, m: &Match, t: &Type) -> Option<&str> {
        match t {
            Type::AnnotationValue { .. } => self
                .node_annos
                .get_value_for_item_by_id(&m.node, m.anno_key),
            Type::TokenText => self.node_annos.get_value_for_item(&m.node, &self.tok_key),
        }
    }
}

impl BinaryOperator for IdenticalValue {
    fn retrieve_matches<'a>(&'a self, lhs: &Match) -> Box<Iterator<Item = Match> > {
        let lhs = lhs.clone();
        if let Some(lhs_val) = self.value_for_match(&lhs, &self.lhs_type) {
                 let lhs_val = lhs_val.to_owned();
            
                 let rhs_candidates : Vec<Match> = match &self.lhs_type {
                     Type::TokenText => {
                         self.node_annos.exact_anno_search(Some(ANNIS_NS.to_string()), TOK.to_string(), ValueSearch::Some(lhs_val))
                     }
                     Type::AnnotationValue {ns, name} => {
                         self.node_annos.exact_anno_search(ns.to_owned(), name.to_owned(), ValueSearch::Some(lhs_val.to_string()))
                     }
                 }.collect();
                 Box::new(rhs_candidates.into_iter())
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let lhs_val = self.value_for_match(lhs, &self.lhs_type);
        let rhs_val = self.value_for_match(rhs, &self.rhs_type);

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
        Some(Box::from(self.clone()))
    }
}
