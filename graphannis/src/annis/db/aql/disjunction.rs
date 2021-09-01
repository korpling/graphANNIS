use super::conjunction::Conjunction;
use crate::{annis::db::aql::model::AnnotationComponentType, AnnotationGraph};
use graphannis_core::types::Component;
use std::collections::HashSet;

pub struct Disjunction {
    pub alternatives: Vec<Conjunction>,
}

impl Disjunction {
    pub fn new(alternatives: Vec<Conjunction>) -> Disjunction {
        Disjunction { alternatives }
    }

    pub fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut result = HashSet::default();

        for alt in &self.alternatives {
            let c = alt.necessary_components(db);
            result.extend(c);
        }

        result
    }

    pub fn get_variable_pos(&self, variable: &str) -> Option<usize> {
        for alt in &self.alternatives {
            if let Ok(var_pos) = alt.resolve_variable_pos(variable, None) {
                return Some(var_pos);
            }
        }
        None
    }

    pub fn get_variable_by_pos(&self, pos: usize) -> Option<String> {
        for alt in &self.alternatives {
            if let Some(var) = alt.get_variable_by_pos(pos) {
                return Some(var);
            }
        }
        None
    }

    pub fn is_included_in_output(&self, variable: &str) -> bool {
        for alt in &self.alternatives {
            if alt.is_included_in_output(variable) {
                return true;
            }
        }
        false
    }
}
