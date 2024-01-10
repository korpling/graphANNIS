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

    /// Return the variable name for a given position in the match output list.
    ///
    /// Optional nodes that are not part of the output are ignored. If there are
    /// no optional nodes, this corresponds to the index of the node in the
    /// query.
    pub(crate) fn get_variable_by_pos(&self, pos: usize) -> Option<String> {
        for alt in &self.alternatives {
            if let Some(var) = alt.get_variable_by_pos(pos) {
                return Some(var);
            }
        }
        None
    }
}
