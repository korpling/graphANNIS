use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::operator::BinaryOperator;
use crate::annis::operator::BinaryOperatorBase;
use crate::annis::operator::BinaryOperatorIndex;
use crate::annis::operator::BinaryOperatorSpec;
use crate::AnnotationGraph;
use crate::{annis::operator::EstimationType, graph::Match, model::AnnotationComponent};
use graphannis_core::graph::DEFAULT_ANNO_KEY;
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct RightAlignmentSpec;

#[derive(Clone)]
pub struct RightAlignment<'a> {
    tok_helper: TokenHelper<'a>,
}

impl BinaryOperatorSpec for RightAlignmentSpec {
    fn necessary_components(&self, db: &AnnotationGraph) -> HashSet<AnnotationComponent> {
        let mut v = HashSet::default();
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let optional_op = RightAlignment::new(db);
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl<'a> RightAlignment<'a> {
    pub fn new(graph: &'a AnnotationGraph) -> Option<RightAlignment<'a>> {
        let tok_helper = TokenHelper::new(graph)?;

        Some(RightAlignment { tok_helper })
    }
}

impl<'a> std::fmt::Display for RightAlignment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_r_")
    }
}

impl<'a> BinaryOperatorBase for RightAlignment<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(lhs_token), Some(rhs_token)) = (
            self.tok_helper.right_token_for(lhs.node),
            self.tok_helper.right_token_for(rhs.node),
        ) {
            lhs_token == rhs_token
        } else {
            false
        }
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator<'b>(&self, graph: &'b AnnotationGraph) -> Option<BinaryOperator<'b>> {
        let tok_helper = TokenHelper::new(graph)?;

        Some(BinaryOperator::Index(Box::new(RightAlignment {
            tok_helper,
        })))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(stats_right) = self.tok_helper.get_gs_right_token_().get_statistics() {
            let aligned_nodes_per_token: f64 = stats_right.inverse_fan_out_99_percentile as f64;
            return EstimationType::Selectivity(
                aligned_nodes_per_token / (stats_right.nodes as f64),
            );
        }

        EstimationType::Selectivity(0.1)
    }
}

impl<'a> BinaryOperatorIndex for RightAlignment<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        let mut aligned = Vec::default();

        if let Some(lhs_token) = self.tok_helper.right_token_for(lhs.node) {
            aligned.push(Match {
                node: lhs_token,
                anno_key: DEFAULT_ANNO_KEY.clone(),
            });
            aligned.extend(
                self.tok_helper
                    .get_gs_right_token_()
                    .get_ingoing_edges(lhs_token)
                    .map(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    }),
            );
        }

        Box::from(aligned.into_iter())
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
