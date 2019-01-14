use crate::annis::db::exec::nodesearch::NodeSearchSpec;
use crate::annis::db::{AnnoStorage, AnnotationStorage, Graph, Match, ValueSearch, ANNIS_NS, TOK};
use crate::annis::operator::*;
use crate::annis::types::{AnnoKey, Component, NodeID};
use std;
use std::sync::Arc;


#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalValueSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
}

impl BinaryOperatorSpec for IdenticalValueSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        vec![]
    }

    fn create_operator<'a>(&self, db: &Graph) -> Option<Box<dyn BinaryOperator>> {
        Some(Box::new(IdenticalValue {
            node_annos: db.node_annos.clone(),
            spec_left: self.spec_left.clone(),
            spec_right: self.spec_right.clone(),
            tok_key: db.get_token_key(),
        }))
    }
}

#[derive(Clone)]
pub struct IdenticalValue {
    node_annos: Arc<AnnoStorage<NodeID>>,
    tok_key: AnnoKey,
    spec_left: NodeSearchSpec,
    spec_right: NodeSearchSpec,
}

impl std::fmt::Display for IdenticalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "==")
    }
}

impl IdenticalValue {
    fn value_for_match(&self, m: &Match, spec: &NodeSearchSpec) -> Option<&str> {
        match spec {
            NodeSearchSpec::ExactValue { .. }
            | NodeSearchSpec::NotExactValue { .. }
            | NodeSearchSpec::RegexValue { .. }
            | NodeSearchSpec::NotRegexValue { .. } => {
                self
                .node_annos
                .get_value_for_item_by_id(&m.node, m.anno_key)
            }
            NodeSearchSpec::AnyToken
            | NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => self.node_annos.get_value_for_item(&m.node, &self.tok_key),
            NodeSearchSpec::AnyNode => None,
        }
    }
}

impl BinaryOperator for IdenticalValue {
    fn retrieve_matches<'a>(&'a self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let lhs = lhs.clone();
        if let Some(lhs_val) = self.value_for_match(&lhs, &self.spec_left) {
            let lhs_val = lhs_val.to_owned();

            let rhs_candidates : Vec<Match> = match &self.spec_right {
                NodeSearchSpec::ExactValue { ns, name, .. }
                | NodeSearchSpec::NotExactValue { ns, name, .. }
                | NodeSearchSpec::RegexValue { ns, name, .. }
                | NodeSearchSpec::NotRegexValue { ns, name, .. } => self.node_annos.exact_anno_search(
                        ns.to_owned(),
                        name.to_owned(),
                        ValueSearch::Some(lhs_val.to_string()),
                    ),
                NodeSearchSpec::AnyToken
                | NodeSearchSpec::ExactTokenValue { .. }
                | NodeSearchSpec::NotExactTokenValue { .. }
                | NodeSearchSpec::RegexTokenValue {  .. }
                | NodeSearchSpec::NotRegexTokenValue {  .. } => self.node_annos.exact_anno_search(
                    Some(ANNIS_NS.to_string()),
                    TOK.to_string(),
                    ValueSearch::Some(lhs_val),
                ),
                NodeSearchSpec::AnyNode => Box::new(std::iter::empty()),
            }
            .collect();
            Box::new(rhs_candidates.into_iter())
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let lhs_val = self.value_for_match(lhs, &self.spec_left);
        let rhs_val = self.value_for_match(rhs, &self.spec_right);

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
