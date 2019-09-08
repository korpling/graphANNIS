use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::Graph;
use crate::annis::db::Match;
use crate::annis::operator::BinaryOperator;
use crate::annis::operator::BinaryOperatorSpec;
use crate::annis::operator::EstimationType;
use crate::annis::types::{AnnoKeyID, Component};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct RightAlignmentSpec;

#[derive(Clone)]
pub struct RightAlignment {
    tok_helper: TokenHelper,
}

impl BinaryOperatorSpec for RightAlignmentSpec {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component> {
        let mut v = HashSet::default();
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<dyn BinaryOperator>> {
        let optional_op = RightAlignment::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl RightAlignment {
    pub fn new(db: &Graph) -> Option<RightAlignment> {
        let tok_helper = TokenHelper::new(db)?;

        Some(RightAlignment { tok_helper })
    }
}

impl std::fmt::Display for RightAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_r_")
    }
}

impl BinaryOperator for RightAlignment {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        let mut aligned = Vec::default();

        if let Some(lhs_token) = self.tok_helper.right_token_for(lhs.node) {
            aligned.push(Match {
                node: lhs_token,
                anno_key: AnnoKeyID::default(),
            });
            aligned.extend(
                self.tok_helper
                    .get_gs_right_token_()
                    .get_ingoing_edges(lhs_token)
                    .map(|n| Match {
                        node: n,
                        anno_key: AnnoKeyID::default(),
                    }),
            );
        }

        Box::from(aligned.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(lhs_token), Some(rhs_token)) = (
            self.tok_helper.right_token_for(lhs.node),
            self.tok_helper.right_token_for(rhs.node),
        ) {
            return lhs_token == rhs_token;
        } else {
            return false;
        }
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator(&self) -> Option<Box<dyn BinaryOperator>> {
        Some(Box::new(self.clone()))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_right) = self.tok_helper.get_gs_right_token_().get_statistics() {
            let aligned_nodes_per_token: f64 = stats_right.inverse_fan_out_99_percentile as f64;
            return EstimationType::SELECTIVITY(
                aligned_nodes_per_token / (stats_right.nodes as f64),
            );
        }

        EstimationType::SELECTIVITY(0.1)
    }
}
