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

    /// Return true if the query node with the given node number inside the given
    /// alternative is supposed to be included in a match output. The node number
    /// is the one inside the alternative and not the one in the whole query.
    pub(crate) fn is_included_in_output(&self, alternative: usize, node_nr: usize) -> bool {
        self.alternatives
            .get(alternative)
            .is_some_and(|alt| alt.is_included_in_output(node_nr))
    }
}
