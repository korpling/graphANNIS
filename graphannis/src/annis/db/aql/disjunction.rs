use super::conjunction::Conjunction;
use crate::{annis::db::aql::model::AnnotationComponentType, AnnotationGraph};
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

    /// Returns true if the node at given position in the query should be included in the output for all alternatives.
    pub(crate) fn position_included_in_output(&self, idx: usize) -> bool {
        for alt in &self.alternatives {
            if !alt.position_included_in_output(idx) {
                return false;
            }
        }
        true
    }
}
