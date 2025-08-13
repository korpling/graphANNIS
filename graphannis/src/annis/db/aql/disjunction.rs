use super::conjunction::Conjunction;
use crate::{AnnotationGraph, annis::db::aql::model::AnnotationComponentType};
use graphannis_core::types::Component;
use std::collections::HashSet;

/// A disjunction is a parsed and normalized AQL query.
pub struct Disjunction {
    pub(crate) alternatives: Vec<Conjunction>,
}

impl Disjunction {
    pub(crate) fn new(alternatives: Vec<Conjunction>) -> Disjunction {
        Disjunction { alternatives }
    }

    pub(crate) fn necessary_components(
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

    pub(crate) fn get_variable_pos(&self, variable: &str) -> Option<usize> {
        for alt in &self.alternatives {
            if let Ok(var_pos) = alt.resolve_variable_pos(variable, None) {
                return Some(var_pos);
            }
        }
        None
    }

    /// Return the variable name for a node number. The node number is the
    /// position of an AQL query node in the disjunction.
    pub(crate) fn get_variable_by_node_nr(&self, node_nr: usize) -> Option<String> {
        for alt in &self.alternatives {
            if let Some(var) = alt.get_variable_by_node_nr(node_nr) {
                return Some(var);
            }
        }
        None
    }

    pub(crate) fn is_included_in_output(&self, variable: &str) -> bool {
        for alt in &self.alternatives {
            if alt.is_included_in_output(variable) {
                return true;
            }
        }
        false
    }
}
