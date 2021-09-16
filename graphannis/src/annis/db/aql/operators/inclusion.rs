use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::AnnotationGraph;
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    graph::{GraphStorage, Match},
    model::AnnotationComponentType,
};
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY},
    types::Component,
};

use std::any::Any;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct InclusionSpec;

pub struct Inclusion<'a> {
    gs_order: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
}

lazy_static! {
    static ref COMPONENT_ORDER: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

impl BinaryOperatorSpec for InclusionSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut v = HashSet::default();
        v.insert(COMPONENT_ORDER.clone());
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let optional_op = Inclusion::new(db);
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl<'a> Inclusion<'a> {
    pub fn new(db: &'a AnnotationGraph) -> Option<Inclusion<'a>> {
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(Inclusion {
            gs_order,
            tok_helper,
        })
    }
}

impl<'a> std::fmt::Display for Inclusion<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_i_")
    }
}

impl<'a> BinaryOperatorBase for Inclusion<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let left_right_lhs = self.tok_helper.left_right_token_for(lhs.node);
        let left_right_rhs = self.tok_helper.left_right_token_for(rhs.node);
        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) = (
            left_right_lhs.0,
            left_right_lhs.1,
            left_right_rhs.0,
            left_right_rhs.1,
        ) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(start_lhs, end_lhs) {
                // path between left-most tokens exists in ORDERING component and has maximum length l
                if self.gs_order.is_connected(start_lhs, start_rhs, 0, std::ops::Bound::Included(l))
                // path between right-most tokens exists in ORDERING component and has maximum length l
                && self.gs_order.is_connected(end_rhs, end_lhs, 0, std::ops::Bound::Included(l))
                {
                    return true;
                }
            }
        }

        false
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn estimation_type(&self) -> EstimationType {
        if let (Some(stats_order), Some(stats_left)) = (
            self.gs_order.get_statistics(),
            self.tok_helper.get_gs_left_token().get_statistics(),
        ) {
            let mut sum_cov_nodes = 0;
            let mut sum_included = 0;

            let num_of_token = stats_order.nodes as f64;
            for gs_cov in self.tok_helper.get_gs_coverage().iter() {
                if let Some(stats_cov) = gs_cov.get_statistics() {
                    sum_cov_nodes += stats_cov.nodes;

                    let covered_token_per_node = stats_cov.fan_out_99_percentile;
                    let aligned_non_token =
                        covered_token_per_node * stats_left.inverse_fan_out_99_percentile;

                    sum_included += covered_token_per_node + aligned_non_token;
                }
            }
            if sum_cov_nodes == 0 {
                // only token in this corpus
                return EstimationType::Selectivity(1.0 / num_of_token);
            } else {
                return EstimationType::Selectivity((sum_included as f64) / (sum_cov_nodes as f64));
            }
        }

        EstimationType::Selectivity(0.1)
    }
}

impl<'a> BinaryOperatorIndex for Inclusion<'a> {
    #[allow(clippy::needless_collect)]
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        if let (Some(start_lhs), Some(end_lhs)) = self.tok_helper.left_right_token_for(lhs.node) {
            // span length of LHS
            if let Some(l) = self.gs_order.distance(start_lhs, end_lhs) {
                // find each token which is between the left and right border
                let result: VecDeque<Match> = self
                    .gs_order
                    .find_connected(start_lhs, 0, std::ops::Bound::Included(l))
                    .flat_map(move |t| {
                        let it_aligned = self
                            .tok_helper
                            .get_gs_left_token()
                            .get_ingoing_edges(t)
                            .filter(move |n| {
                                // right-aligned token of candidate
                                let mut end_n =
                                    self.tok_helper.get_gs_right_token_().get_outgoing_edges(*n);
                                if let Some(end_n) = end_n.next() {
                                    // path between right-most tokens exists in ORDERING component
                                    // and has maximum length l
                                    self.gs_order.is_connected(
                                        end_n,
                                        end_lhs,
                                        0,
                                        std::ops::Bound::Included(l),
                                    )
                                } else {
                                    false
                                }
                            });
                        // return the token itself and all aligned nodes
                        std::iter::once(t).chain(it_aligned)
                    })
                    .map(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    })
                    .collect();
                return Box::new(result.into_iter());
            }
        }

        Box::new(std::iter::empty())
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
