use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::{Graph, Match, DEFAULT_ANNO_KEY};
use crate::annis::operator::BinaryOperator;
use crate::annis::operator::BinaryOperatorSpec;
use crate::annis::operator::EstimationType;
use crate::annis::types::Component;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct LeftAlignmentSpec;

#[derive(Clone)]
pub struct LeftAlignment<'a> {
    tok_helper: &'a TokenHelper,
}

impl BinaryOperatorSpec for LeftAlignmentSpec {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component> {
        let mut v = HashSet::default();
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(&self, db: &'a Graph) -> Option<Box<dyn BinaryOperator + 'a>> {
        let optional_op = LeftAlignment::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl<'a> LeftAlignment<'a> {
    pub fn new(graph: &'a Graph) -> Option<LeftAlignment<'a>> {
        let tok_helper = graph.get_token_helper()?;

        Some(LeftAlignment { tok_helper })
    }
}

impl<'a> std::fmt::Display for LeftAlignment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_l_")
    }
}

impl<'a> BinaryOperator for LeftAlignment<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        let mut aligned = Vec::default();

        if let Some(lhs_token) = self.tok_helper.left_token_for(lhs.node) {
            aligned.push(Match {
                node: lhs_token,
                anno_key: DEFAULT_ANNO_KEY.clone(),
            });
            aligned.extend(
                self.tok_helper
                    .get_gs_left_token()
                    .get_ingoing_edges(lhs_token)
                    .map(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    }),
            );
        }

        Box::from(aligned.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(lhs_token), Some(rhs_token)) = (
            self.tok_helper.left_token_for(lhs.node),
            self.tok_helper.left_token_for(rhs.node),
        ) {
            return lhs_token == rhs_token;
        } else {
            return false;
        }
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator<'b>(&self, graph: &'b Graph) -> Option<Box<dyn BinaryOperator + 'b>> {
        let tok_helper = graph.get_token_helper()?;

        Some(Box::new(LeftAlignment { tok_helper }))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_left) = self.tok_helper.get_gs_left_token().get_statistics() {
            let aligned_nodes_per_token: f64 = stats_left.inverse_fan_out_99_percentile as f64;
            return EstimationType::SELECTIVITY(
                aligned_nodes_per_token / (stats_left.nodes as f64),
            );
        }

        EstimationType::SELECTIVITY(0.1)
    }
}
