use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    graph::{GraphStorage, Match},
    model::AnnotationComponentType,
    AnnotationGraph,
};
use graphannis_core::{
    annostorage::MatchGroup,
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY},
    types::Component,
};

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalCoverageSpec;

#[derive(Clone)]
pub struct IdenticalCoverage<'a> {
    gs_left: Arc<dyn GraphStorage>,
    gs_order: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
}

lazy_static! {
    static ref COMPONENT_LEFT: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::LeftToken,
            ANNIS_NS.into(),
            "".into(),
        )
    };
    static ref COMPONENT_ORDER: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

impl BinaryOperatorSpec for IdenticalCoverageSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut v = HashSet::new();
        v.insert(COMPONENT_LEFT.clone());
        v.insert(COMPONENT_ORDER.clone());
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let optional_op = IdenticalCoverage::new(db);
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<'a> IdenticalCoverage<'a> {
    pub fn new(db: &'a AnnotationGraph) -> Option<IdenticalCoverage<'a>> {
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT)?;
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER)?;

        Some(IdenticalCoverage {
            gs_left,
            gs_order,
            tok_helper: TokenHelper::new(db)?,
        })
    }
}

impl<'a> std::fmt::Display for IdenticalCoverage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_=_")
    }
}

impl<'a> BinaryOperatorBase for IdenticalCoverage<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        let start_lhs = self.tok_helper.left_token_for(lhs.node);
        let end_lhs = self.tok_helper.right_token_for(lhs.node);

        let start_rhs = self.tok_helper.left_token_for(rhs.node);
        let end_rhs = self.tok_helper.right_token_for(rhs.node);

        if start_lhs.is_none() || end_lhs.is_none() || start_rhs.is_none() || end_rhs.is_none() {
            return false;
        }

        start_lhs.unwrap() == start_rhs.unwrap() && end_lhs.unwrap() == end_rhs.unwrap()
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator<'b>(&self, graph: &'b AnnotationGraph) -> Option<BinaryOperator<'b>> {
        Some(BinaryOperator::Index(Box::new(IdenticalCoverage {
            gs_left: self.gs_left.clone(),
            gs_order: self.gs_order.clone(),
            tok_helper: TokenHelper::new(graph)?,
        })))
    }

    fn estimation_type(&self) -> EstimationType {
        if let Some(order_stats) = self.gs_order.get_statistics() {
            let num_of_token = order_stats.nodes as f64;

            // Assume two nodes have same identical coverage if they have the same
            // left covered token and the same length (right covered token is not independent
            // of the left one, this is why we should use length).
            // The probability for the same length is taken is assumed to be 1.0, histograms
            // of the distribution would help here.

            EstimationType::Selectivity(1.0 / num_of_token)
        } else {
            EstimationType::Selectivity(0.1)
        }
    }
}

impl<'a> BinaryOperatorIndex for IdenticalCoverage<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        let n_left = self.tok_helper.left_token_for(lhs.node);
        let n_right = self.tok_helper.right_token_for(lhs.node);

        let mut result = MatchGroup::new();

        if let (Some(n_left), Some(n_right)) = (n_left, n_right) {
            if n_left == n_right {
                // covered range is exactly one token, add token itself
                result.push(Match {
                    node: n_left,
                    anno_key: DEFAULT_ANNO_KEY.clone(),
                });
            }

            // find left-aligned non-token
            let v = self.gs_left.get_ingoing_edges(n_left);
            for c in v {
                // check if also right-aligned
                if let Some(c_right) = self.tok_helper.right_token_for(c) {
                    if n_right == c_right {
                        result.push(Match {
                            node: c,
                            anno_key: DEFAULT_ANNO_KEY.clone(),
                        });
                    }
                }
            }
        }

        Box::new(result.into_iter())
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
