use crate::annis::db::exec::CostEstimate;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::try_as_boxed_iter;
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    errors::Result,
    graph::{GraphStorage, Match},
    model::AnnotationComponentType,
    AnnotationGraph,
};
use graphannis_core::{
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

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        _cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>> {
        let optional_op = IdenticalCoverage::new(db);
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl<'a> IdenticalCoverage<'a> {
    pub fn new(db: &'a AnnotationGraph) -> Result<IdenticalCoverage<'a>> {
        let gs_left = db.get_graphstorage(&COMPONENT_LEFT).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "LeftToken component is missing (needed by _=_ operator)".to_string(),
            )
        })?;
        let gs_order = db.get_graphstorage(&COMPONENT_ORDER).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "Ordering component is missing for (needed by _=_ operator)".to_string(),
            )
        })?;

        Ok(IdenticalCoverage {
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
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let start_lhs = self.tok_helper.left_token_for(lhs.node)?;
        let end_lhs = self.tok_helper.right_token_for(lhs.node)?;

        let start_rhs = self.tok_helper.left_token_for(rhs.node)?;
        let end_rhs = self.tok_helper.right_token_for(rhs.node)?;

        if let (Some(start_lhs), Some(end_lhs), Some(start_rhs), Some(end_rhs)) =
            (start_lhs, end_lhs, start_rhs, end_rhs)
        {
            let result = start_lhs == start_rhs && end_lhs == end_rhs;
            Ok(result)
        } else {
            Ok(false)
        }
    }

    fn is_reflexive(&self) -> bool {
        false
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        let inverse = BinaryOperator::Index(Box::new(IdenticalCoverage {
            gs_left: self.gs_left.clone(),
            gs_order: self.gs_order.clone(),
            tok_helper: TokenHelper::new(graph)?,
        }));
        Ok(Some(inverse))
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some(order_stats) = self.gs_order.get_statistics() {
            let num_of_token = order_stats.nodes as f64;

            // Assume two nodes have same identical coverage if they have the same
            // left covered token and the same length (right covered token is not independent
            // of the left one, this is why we should use length).
            // The probability for the same length is taken is assumed to be 1.0, histograms
            // of the distribution would help here.

            Ok(EstimationType::Selectivity(1.0 / num_of_token))
        } else {
            Ok(EstimationType::Selectivity(0.1))
        }
    }
}

impl<'a> BinaryOperatorIndex for IdenticalCoverage<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>>> {
        let n_left = try_as_boxed_iter!(self.tok_helper.left_token_for(lhs.node));
        let n_right = try_as_boxed_iter!(self.tok_helper.right_token_for(lhs.node));

        let mut result = Vec::new();

        if let (Some(n_left), Some(n_right)) = (n_left, n_right) {
            if n_left == n_right {
                // covered range is exactly one token, add token itself
                result.push(Ok(Match {
                    node: n_left,
                    anno_key: DEFAULT_ANNO_KEY.clone(),
                }));
            }

            // find left-aligned non-token
            for c in self.gs_left.get_ingoing_edges(n_left) {
                match c {
                    Ok(c) => {
                        // check if also right-aligned
                        match self.tok_helper.right_token_for(c) {
                            Ok(c_right) => {
                                if let Some(c_right) = c_right {
                                    if n_right == c_right {
                                        result.push(Ok(Match {
                                            node: c,
                                            anno_key: DEFAULT_ANNO_KEY.clone(),
                                        }));
                                    }
                                }
                            }
                            Err(e) => result.push(Err(e)),
                        }
                    }
                    Err(e) => result.push(Err(e.into())),
                }
            }
        }

        Box::new(result.into_iter())
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
