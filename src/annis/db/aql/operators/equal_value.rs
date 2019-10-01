use crate::annis::db::exec::nodesearch::NodeSearchSpec;
use crate::annis::db::{AnnotationStorage, Graph, Match, ValueSearch, ANNIS_NS, TOK};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKey, Component, NodeID};
use std;
use std::collections::HashSet;
use std::sync::Arc;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct EqualValueSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
    pub negated: bool,
}

impl BinaryOperatorSpec for EqualValueSpec {
    fn necessary_components(&self, _db: &Graph) -> HashSet<Component> {
        HashSet::default()
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<dyn BinaryOperator>> {
        Some(Box::new(EqualValue {
            node_annos: db.node_annos.clone(),
            spec_left: self.spec_left.clone(),
            spec_right: self.spec_right.clone(),
            tok_key: db.get_token_key(),
            negated: self.negated,
        }))
    }

    fn is_binding(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct EqualValue {
    node_annos: Arc<AnnotationStorage<NodeID>>,
    tok_key: AnnoKey,
    spec_left: NodeSearchSpec,
    spec_right: NodeSearchSpec,
    negated: bool,
}

impl std::fmt::Display for EqualValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.negated {
            write!(f, "!=")
        } else {
            write!(f, "==")
        }
    }
}

impl EqualValue {
    fn value_for_match(&self, m: &Match, spec: &NodeSearchSpec) -> Option<Cow<str>> {
        match spec {
            NodeSearchSpec::ExactValue { .. }
            | NodeSearchSpec::NotExactValue { .. }
            | NodeSearchSpec::RegexValue { .. }
            | NodeSearchSpec::NotRegexValue { .. } => self
                .node_annos
                .get_value_for_item(&m.node, &m.anno_key),
            NodeSearchSpec::AnyToken
            | NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => {
                self.node_annos.get_value_for_item(&m.node, &self.tok_key)
            }
            NodeSearchSpec::AnyNode => None,
        }
    }

    fn anno_def_for_spec(spec: &NodeSearchSpec) -> Option<(Option<String>, String)> {
        match spec {
            NodeSearchSpec::ExactValue { ns, name, .. }
            | NodeSearchSpec::NotExactValue { ns, name, .. }
            | NodeSearchSpec::RegexValue { ns, name, .. }
            | NodeSearchSpec::NotRegexValue { ns, name, .. } => Some((ns.clone(), name.clone())),
            NodeSearchSpec::AnyToken
            | NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => {
                let ns = Some(ANNIS_NS.to_string());
                let name = TOK.to_string();
                Some((ns, name))
            }
            NodeSearchSpec::AnyNode => None,
        }
    }
}

impl BinaryOperator for EqualValue {
    fn retrieve_matches<'a>(&'a self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let lhs = lhs.clone();
        if let Some(lhs_val) = self.value_for_match(&lhs, &self.spec_left) {
            let lhs_val = lhs_val.to_string();
            let val_search = if self.negated {
                ValueSearch::NotSome(lhs_val)
            } else {
                ValueSearch::Some(lhs_val)
            };

            if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_right) {
                let rhs_candidates: Vec<Match> = self
                    .node_annos
                    .exact_anno_search(ns, name, val_search)
                    .collect();
                return Box::new(rhs_candidates.into_iter());
            }
        }
        Box::new(std::iter::empty())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let lhs_val = self.value_for_match(lhs, &self.spec_left);
        let rhs_val = self.value_for_match(rhs, &self.spec_right);

        if let (Some(lhs_val), Some(rhs_val)) = (lhs_val, rhs_val) {
            if self.negated {
                lhs_val != rhs_val
            } else {
                lhs_val == rhs_val
            }
        } else {
            false
        }
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_left) {
            if let Some(most_frequent_value_left) =
                self.node_annos.guess_most_frequent_value(ns, name)
            {
                if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_right) {
                    let guessed_count_right = self.node_annos.guess_max_count(
                        ns.clone(),
                        name.clone(),
                        &most_frequent_value_left,
                        &most_frequent_value_left,
                    );

                    let total_annos = self.node_annos.number_of_annotations_by_name(ns, name);
                    let sel = guessed_count_right as f64 / total_annos as f64;
                    if self.negated {
                        return EstimationType::SELECTIVITY(1.0 - sel);
                    } else {
                        return EstimationType::SELECTIVITY(sel);
                    }
                }
            }
        }
        // fallback to default
        EstimationType::SELECTIVITY(0.5)
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        Some(Box::from(self.clone()))
    }
}
