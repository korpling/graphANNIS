use crate::annis::db::aql::{model::AnnotationComponentType, operators::RangeSpec};
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{BinaryOperator, BinaryOperatorIndex, EstimationType};
use crate::{
    annis::operator::{BinaryOperatorBase, BinaryOperatorSpec},
    errors::Result,
    graph::{GraphStorage, Match},
};
use crate::{try_as_boxed_iter, AnnotationGraph};
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY},
    types::Component,
};

use itertools::Itertools;
use rustc_hash::FxHashSet;
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NearSpec {
    pub segmentation: Option<String>,
    pub dist: RangeSpec,
}

#[derive(Clone)]
struct Near<'a> {
    gs_order: Arc<dyn GraphStorage>,
    tok_helper: TokenHelper<'a>,
    spec: NearSpec,
}

impl BinaryOperatorSpec for NearSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let component_order = Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            self.segmentation
                .as_ref()
                .map_or_else(smartstring::alias::String::default, |s| s.into()),
        );

        let mut v = HashSet::default();
        v.insert(component_order);
        v.extend(token_helper::necessary_components(db));
        v
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Result<BinaryOperator<'a>> {
        let optional_op = Near::new(db, self.clone());
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Display for NearSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref seg) = self.segmentation {
            write!(f, "{} {}", seg, self.dist)
        } else {
            write!(f, "{}", self.dist)
        }
    }
}

impl<'a> Near<'a> {
    pub fn new(graph: &'a AnnotationGraph, spec: NearSpec) -> Result<Near<'a>> {
        let component_order = Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            spec.segmentation.clone().unwrap_or_default().into(),
        );

        let gs_order = graph.get_graphstorage(&component_order).ok_or_else(|| {
            GraphAnnisError::ImpossibleSearch(
                "Ordering component missing (needed for ^ operator)".to_string(),
            )
        })?;

        let tok_helper = TokenHelper::new(graph)?;

        Ok(Near {
            gs_order,
            tok_helper,
            spec,
        })
    }
}

impl<'a> std::fmt::Display for Near<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "^{}", self.spec)
    }
}

impl<'a> BinaryOperatorBase for Near<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        let start_end_forward = if self.spec.segmentation.is_some() {
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
        let start_end_backward = if self.spec.segmentation.is_some() {
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
            start_end_forward.0,
            start_end_forward.1,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )? || self.gs_order.is_connected(
            start_end_backward.1,
            start_end_backward.0,
            self.spec.dist.min_dist(),
            self.spec.dist.max_dist(),
        )?;
        Ok(result)
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if let Some(stats_order) = self.gs_order.get_statistics() {
            let max_dist = match self.spec.dist.max_dist() {
                std::ops::Bound::Unbounded => usize::max_value(),
                std::ops::Bound::Included(max_dist) => max_dist,
                std::ops::Bound::Excluded(max_dist) => max_dist - 1,
            };
            let max_possible_dist = std::cmp::min(max_dist, stats_order.max_depth);
            let num_of_descendants = 2 * (max_possible_dist - self.spec.dist.min_dist() + 1);

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
        let inverse = BinaryOperator::Index(Box::new(Near {
            gs_order: self.gs_order.clone(),
            tok_helper: TokenHelper::new(graph)?,
            spec: self.spec.clone(),
        }));
        Ok(Some(inverse))
    }
}

impl<'a> BinaryOperatorIndex for Near<'a> {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>>> {
        let start_forward = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            try_as_boxed_iter!(self.tok_helper.right_token_for(lhs.node))
        };

        let start_backward = if self.spec.segmentation.is_some() {
            Some(lhs.node)
        } else {
            try_as_boxed_iter!(self.tok_helper.left_token_for(lhs.node))
        };

        let it_forward: Box<dyn Iterator<Item = Result<u64>>> = if let Some(start) = start_forward {
            let connected = self
                .gs_order
                // get all token in the range
                .find_connected(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
                .fuse();
            let it = connected
                .map_ok(|t| {
                    let it_aligned = self.tok_helper.get_gs_left_token().get_ingoing_edges(t);
                    std::iter::once(Ok(t)).chain(it_aligned)
                })
                .flatten_ok()
                // Unwrap the Result<Result<_>>
                .map(|c| match c {
                    Ok(c) => match c {
                        Ok(c) => Ok(c),
                        Err(e) => Err(GraphAnnisError::from(e)),
                    },
                    Err(e) => Err(GraphAnnisError::from(e)),
                });

            Box::new(it)
        } else {
            Box::new(std::iter::empty::<Result<u64>>())
        };

        let it_backward: Box<dyn Iterator<Item = Result<u64>>> = if let Some(start) = start_backward
        {
            let connected = self
                .gs_order
                // get all token in the range
                .find_connected_inverse(start, self.spec.dist.min_dist(), self.spec.dist.max_dist())
                .fuse();
            let it = connected
                // find all right aligned nodes for this token and add it together with the token itself
                .map_ok(move |t| {
                    let it_aligned = self.tok_helper.get_gs_right_token_().get_ingoing_edges(t);
                    std::iter::once(Ok(t)).chain(it_aligned)
                })
                .flatten_ok()
                // Unwrap the Result<Result<_>>
                .map(|c| match c {
                    Ok(c) => match c {
                        Ok(c) => Ok(c),
                        Err(e) => Err(GraphAnnisError::from(e)),
                    },
                    Err(e) => Err(GraphAnnisError::from(e)),
                });
            Box::new(it)
        } else {
            Box::new(std::iter::empty::<Result<u64>>())
        };

        // materialize a set of all matches
        let result: Result<FxHashSet<Match>> = it_forward
            .chain(it_backward)
            // map the result as match
            .map_ok(|n| Match {
                node: n,
                anno_key: DEFAULT_ANNO_KEY.clone(),
            })
            .collect();
        let result = try_as_boxed_iter!(result);
        Box::new(result.into_iter().map(Ok))
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
