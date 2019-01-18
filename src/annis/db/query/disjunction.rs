use super::conjunction::Conjunction;
use crate::annis::db::Graph;
use crate::annis::types::Component;

pub struct Disjunction<'a> {
    pub alternatives: Vec<Conjunction<'a>>,
}

impl<'a> Disjunction<'a> {
    pub fn new(alternatives: Vec<Conjunction<'a>>) -> Disjunction<'a> {
        Disjunction { alternatives }
    }

    pub fn necessary_components(&self, db: &Graph) -> Vec<Component> {
        let mut result = vec![];

        for alt in &self.alternatives {
            let mut c = alt.necessary_components(db);
            result.append(&mut c);
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
}
