use crate::annis::db::exec::nodesearch::NodeSearchSpec;
use crate::annis::db::exec::CostEstimate;
use crate::{
    annis::{
        db::aql::model::{AnnotationComponentType, TOK, TOKEN_KEY},
        operator::*,
    },
    errors::Result,
    graph::Match,
};
use crate::{try_as_boxed_iter, AnnotationGraph};
use graphannis_core::annostorage::NodeAnnotationStorage;
use graphannis_core::{annostorage::ValueSearch, graph::ANNIS_NS, types::Component};
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct EqualValueSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
    pub negated: bool,
}

impl BinaryOperatorSpec for EqualValueSpec {
    fn necessary_components(
        &self,
        _db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::default()
    }

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        _cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>> {
        Ok(BinaryOperator::Index(Box::new(EqualValue {
            node_annos: db.get_node_annos(),
            spec_left: self.spec_left.clone(),
            spec_right: self.spec_right.clone(),
            negated: self.negated,
        })))
    }

    fn is_binding(&self) -> bool {
        false
    }

    #[cfg(test)]
    fn into_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any> {
        self
    }

    #[cfg(test)]
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
pub struct EqualValue<'a> {
    node_annos: &'a dyn NodeAnnotationStorage,
    spec_left: NodeSearchSpec,
    spec_right: NodeSearchSpec,
    negated: bool,
}

impl<'a> std::fmt::Display for EqualValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.negated {
            write!(f, "!=")
        } else {
            write!(f, "==")
        }
    }
}

impl<'a> EqualValue<'a> {
    fn value_for_match(&self, m: &Match, spec: &NodeSearchSpec) -> Result<Option<Cow<str>>> {
        match spec {
            NodeSearchSpec::ExactValue { .. }
            | NodeSearchSpec::NotExactValue { .. }
            | NodeSearchSpec::RegexValue { .. }
            | NodeSearchSpec::NotRegexValue { .. } => {
                let val = self.node_annos.get_value_for_item(&m.node, &m.anno_key)?;
                Ok(val)
            }
            NodeSearchSpec::AnyToken
            | NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => {
                let val = self.node_annos.get_value_for_item(&m.node, &TOKEN_KEY)?;
                Ok(val)
            }
            NodeSearchSpec::AnyNode => Ok(None),
        }
    }

    fn anno_def_for_spec(spec: &NodeSearchSpec) -> Option<(Option<&str>, &str)> {
        match spec {
            NodeSearchSpec::ExactValue { ns, name, .. }
            | NodeSearchSpec::NotExactValue { ns, name, .. }
            | NodeSearchSpec::RegexValue { ns, name, .. }
            | NodeSearchSpec::NotRegexValue { ns, name, .. } => {
                Some((ns.as_ref().map(String::as_str), name))
            }
            NodeSearchSpec::AnyToken
            | NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => {
                let ns = Some(ANNIS_NS);
                let name = TOK;
                Some((ns, name))
            }
            NodeSearchSpec::AnyNode => None,
        }
    }
}

impl<'a> BinaryOperatorBase for EqualValue<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let lhs_val = self.value_for_match(lhs, &self.spec_left)?;
        let rhs_val = self.value_for_match(rhs, &self.spec_right)?;

        let result = if let (Some(lhs_val), Some(rhs_val)) = (lhs_val, rhs_val) {
            if self.negated {
                lhs_val != rhs_val
            } else {
                lhs_val == rhs_val
            }
        } else {
            false
        };
        Ok(result)
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_left) {
            if let Some(most_frequent_value_left) =
                self.node_annos.guess_most_frequent_value(ns, name)?
            {
                if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_right) {
                    let guessed_count_right = self.node_annos.guess_max_count(
                        ns,
                        name,
                        &most_frequent_value_left,
                        &most_frequent_value_left,
                    )?;

                    let total_annos = self.node_annos.number_of_annotations_by_name(ns, name)?;
                    let sel = guessed_count_right as f64 / total_annos as f64;
                    if self.negated {
                        return Ok(EstimationType::Selectivity(1.0 - sel));
                    } else {
                        return Ok(EstimationType::Selectivity(sel));
                    }
                }
            }
        }
        // fallback to default
        Ok(EstimationType::Selectivity(0.5))
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        let inverse = BinaryOperator::Index(Box::from(EqualValue {
            node_annos: graph.get_node_annos(),
            spec_left: self.spec_left.clone(),
            spec_right: self.spec_right.clone(),
            negated: self.negated,
        }));
        Ok(Some(inverse))
    }
}

impl<'a> BinaryOperatorIndex for EqualValue<'a> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>> + 'b> {
        let lhs = lhs.clone();
        let lhs_val = try_as_boxed_iter!(self.value_for_match(&lhs, &self.spec_left));
        if let Some(lhs_val) = lhs_val {
            let val_search: ValueSearch<&str> = if self.negated {
                ValueSearch::NotSome(&lhs_val)
            } else {
                ValueSearch::Some(&lhs_val)
            };

            if let Some((ns, name)) = EqualValue::anno_def_for_spec(&self.spec_right) {
                let rhs_candidates = self
                    .node_annos
                    .exact_anno_search(ns, name, val_search)
                    .map(|m| m.map_err(|e| e.into()));
                return Box::new(rhs_candidates);
            }
        }
        Box::new(std::iter::empty())
    }
    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
