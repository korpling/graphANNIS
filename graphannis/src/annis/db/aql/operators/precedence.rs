use crate::annis::db::aql::operators::RangeSpec;
use crate::annis::db::exec::CostEstimate;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    errors::Result,
    graph::{GraphStorage, Match},
    model::{AnnotationComponent, AnnotationComponentType},
};
use crate::{try_as_boxed_iter, AnnotationGraph};
use graphannis_core::graph::{ANNIS_NS, DEFAULT_ANNO_KEY, DEFAULT_NS};
use itertools::Itertools;

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrecedenceSpec {
    pub segmentation: Option<String>,
    pub dist: RangeSpec,
}

pub struct Precedence<'a> {
    gs_order: Arc<dyn GraphStorage>,
    gs_left: Arc<dyn GraphStorage>,
    gs_right: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
    spec: PrecedenceSpec,
}

lazy_static! {
    static ref COMPONENT_LEFT: AnnotationComponent = {
        AnnotationComponent::new(
            AnnotationComponentType::LeftToken,
            ANNIS_NS.into(),
            "".into(),
        )
    };
    static ref COMPONENT_RIGHT: AnnotationComponent = {
        AnnotationComponent::new(
            AnnotationComponentType::RightToken,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

impl BinaryOperatorSpec for PrecedenceSpec {
    fn necessary_components(&self, db: &AnnotationGraph) -> HashSet<AnnotationComponent> {
        let ordering_layer = if self.segmentation.is_none() {
            ANNIS_NS.to_owned()
        } else {
            DEFAULT_NS.to_owned()
        };
        let component_order = AnnotationComponent::new(
            AnnotationComponentType::Ordering,
            ordering_layer.into(),
            self.segmentation.clone().unwrap_or_default().into(),
        );

        let mut v = HashSet::default();
        v.insert(component_order);
        v.insert(COMPONENT_LEFT.clone());
        v.insert(COMPONENT_RIGHT.clone());
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        _cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>> {
        let op = Precedence::new(db, self.clone())?;
        Ok(BinaryOperator::Index(Box::new(op)))
    }

    #[cfg(test)]
    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any> {
        self
    }

    #[cfg(test)]
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Display for PrecedenceSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref seg) = self.segmentation {
            write!(f, "{} {}", seg, self.dist)
        } else {
            write!(f, "{}", self.dist)
        }
    }
}

impl<'a> Precedence<'a> {
    pub fn new(graph: &'a AnnotationGraph, spec: PrecedenceSpec) -> Result<Precedence<'a>> {
        let ordering_layer = if spec.segmentation.is_none() {
            ANNIS_NS.to_owned()
        } else {
            DEFAULT_NS.to_owned()
        };
        let component_order = AnnotationComponent::new(
            AnnotationComponentType::Ordering,
            ordering_layer.into(),
            spec.segmentation.clone().unwrap_or_default().into(),
        );

        let gs_order = graph.get_graphstorage(&component_order).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "Ordering component missing (needed for . operator)".to_string(),
            )
        })?;
        let gs_left = graph.get_graphstorage(&COMPONENT_LEFT).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "LeftToken component missing (needed for . operator)".to_string(),
            )
        })?;
        let gs_right = graph.get_graphstorage(&COMPONENT_RIGHT).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "RightToken component missing (needed for . operator)".to_string(),
            )
        })?;

        let tok_helper = TokenHelper::new(graph)?;

        Ok(Precedence {
            gs_order,
            gs_left,
            gs_right,
            tok_helper,
            spec,
        })
    }
}

impl std::fmt::Display for Precedence<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".{}", self.spec)
    }
}

impl BinaryOperatorBase for Precedence<'_> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.right_token_for(lhs.node)?;
            let end = self.tok_helper.left_token_for(rhs.node)?;

            if let (Some(start), Some(end)) = (start, end) {
                (start, end)
            } else {
                return Ok(false);
            }
        };

        let result = self.gs_order.is_connected(
            start_end.0,
            start_end.1,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )?;
        Ok(result)
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_dist = match self.spec.dist.max_dist() {
                std::ops::Bound::Unbounded => usize::MAX,
                std::ops::Bound::Included(max_dist) => max_dist,
                std::ops::Bound::Excluded(max_dist) => max_dist - 1,
            };
            let max_possible_dist = std::cmp::min(max_dist, stats_order.max_depth);
            let num_of_descendants =
                max_possible_dist.saturating_sub(self.spec.dist.min_dist()) + 1;

            return Ok(EstimationType::Selectivity(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            ));
        }

        Ok(EstimationType::Selectivity(0.1))
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        // Check if order graph storages has the same inverse cost.
        // If not, we don't provide an inverse operator, because the plans would not account for the different costs
        if !self.gs_order.inverse_has_same_cost() {
            return Ok(None);
        }

        let inv_precedence = InversePrecedence {
            gs_order: self.gs_order.clone(),
            gs_left: self.gs_left.clone(),
            gs_right: self.gs_right.clone(),
            tok_helper: TokenHelper::new(graph)?,
            spec: self.spec.clone(),
        };
        Ok(Some(BinaryOperator::Index(Box::new(inv_precedence))))
    }
}

impl BinaryOperatorIndex for Precedence<'_> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>> + 'b> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            try_as_boxed_iter!(self.tok_helper.right_token_for(lhs.node))
        };

        if start.is_none() {
            return Box::new(std::iter::empty());
        }

        let start = start.unwrap();

        let connected = self
            .gs_order
            // get all token in the range
            .find_connected(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
            .map(|t| t.map_err(GraphAnnisError::from))
            .fuse();
        let result = connected
            // find all left aligned nodes for this token and add it together with the token itself
            .map_ok(move |t| {
                let it_aligned = self.gs_left.get_ingoing_edges(t);
                std::iter::once(Ok(t)).chain(it_aligned)
            })
            .flatten_ok()
            // Unwrap the Result<Result<_>>
            .map(|c| match c {
                Ok(c) => match c {
                    Ok(n) => Ok(Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    }),
                    Err(e) => Err(GraphAnnisError::from(e)),
                },
                Err(e) => Err(e),
            });

        Box::new(result)
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}

pub struct InversePrecedence<'a> {
    gs_order: Arc<dyn GraphStorage>,
    gs_left: Arc<dyn GraphStorage>,
    gs_right: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
    spec: PrecedenceSpec,
}

impl std::fmt::Display for InversePrecedence<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".\u{20D6}{}", self.spec)
    }
}

impl BinaryOperatorBase for InversePrecedence<'_> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let start_end = if self.spec.segmentation.is_some() {
            (lhs.node, rhs.node)
        } else {
            let start = self.tok_helper.left_token_for(lhs.node)?;
            let end = self.tok_helper.right_token_for(rhs.node)?;

            if let (Some(start), Some(end)) = (start, end) {
                (start, end)
            } else {
                return Ok(false);
            }
        };

        let result = self.gs_order.is_connected(
            start_end.1,
            start_end.0,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )?;
        Ok(result)
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        let prec = Precedence {
            gs_order: self.gs_order.clone(),
            gs_left: self.gs_left.clone(),
            gs_right: self.gs_right.clone(),
            tok_helper: TokenHelper::new(graph)?,
            spec: self.spec.clone(),
        };
        Ok(Some(BinaryOperator::Index(Box::new(prec))))
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_dist = match self.spec.dist.max_dist() {
                std::ops::Bound::Unbounded => usize::MAX,
                std::ops::Bound::Included(max_dist) => max_dist,
                std::ops::Bound::Excluded(max_dist) => max_dist - 1,
            };
            let max_possible_dist = std::cmp::min(max_dist, stats_order.max_depth);
            let num_of_descendants = max_possible_dist - self.spec.dist.min_dist() + 1;

            return Ok(EstimationType::Selectivity(
                (num_of_descendants as f64) / (stats_order.nodes as f64 / 2.0),
            ));
        }

        Ok(EstimationType::Selectivity(0.1))
    }
}

impl BinaryOperatorIndex for InversePrecedence<'_> {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>> + 'b> {
        let start = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            try_as_boxed_iter!(self.tok_helper.left_token_for(lhs.node))
        };

        if start.is_none() {
            return Box::new(std::iter::empty());
        }

        let start = start.unwrap();

        let token_in_range = self
            .gs_order
            // get all token in the range
            .find_connected_inverse(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
            .fuse();

        let result = token_in_range
            .map_ok(move |t| {
                // find all right aligned nodes for this token and add it together with the token itself
                let it_aligned = self.gs_right.get_ingoing_edges(t);
                std::iter::once(Ok(t)).chain(it_aligned)
            })
            .flatten_ok()
            // Unwrap the Result<Result<_>>
            .map(|c| match c {
                Ok(c) => match c {
                    Ok(n) => Ok(Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    }),
                    Err(e) => Err(GraphAnnisError::from(e)),
                },
                Err(e) => Err(GraphAnnisError::from(e)),
            });
        Box::new(result)
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
