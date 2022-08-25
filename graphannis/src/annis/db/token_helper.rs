use crate::{
    annis::{
        db::{
            aql::model::{AnnotationComponentType, TOKEN_KEY},
            AnnotationStorage,
        },
        errors::GraphAnnisError,
    },
    errors::Result,
    graph::GraphStorage,
    AnnotationGraph,
};
use graphannis_core::{
    errors::GraphAnnisCoreError,
    graph::ANNIS_NS,
    types::{Component, NodeID},
};
use itertools::Itertools;

use std::collections::HashSet;
use std::{cmp::Ordering, sync::Arc};

#[derive(Clone)]
pub struct TokenHelper<'a> {
    node_annos: &'a dyn AnnotationStorage<NodeID>,
    left_edges: Arc<dyn GraphStorage>,
    right_edges: Arc<dyn GraphStorage>,
    cov_edges: Vec<Arc<dyn GraphStorage>>,
    ordering_edges: Arc<dyn GraphStorage>,
}

lazy_static! {
    static ref COMPONENT_LEFT: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::LeftToken,
            ANNIS_NS.into(),
            "".into(),
        )
    };
    static ref COMPONENT_RIGHT: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::RightToken,
            ANNIS_NS.into(),
            "".into(),
        )
    };
    static ref COMPONENT_ORDERING: Component<AnnotationComponentType> = {
        Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        )
    };
}

pub fn necessary_components(db: &AnnotationGraph) -> HashSet<Component<AnnotationComponentType>> {
    let mut result = HashSet::default();
    result.insert(COMPONENT_LEFT.clone());
    result.insert(COMPONENT_RIGHT.clone());
    // we need all coverage components
    result.extend(
        db.get_all_components(Some(AnnotationComponentType::Coverage), None)
            .into_iter(),
    );

    result
}

impl<'a> TokenHelper<'a> {
    pub fn new(graph: &'a AnnotationGraph) -> Result<TokenHelper<'a>> {
        let cov_edges: Vec<Arc<dyn GraphStorage>> = graph
            .get_all_components(Some(AnnotationComponentType::Coverage), None)
            .into_iter()
            .filter_map(|c| graph.get_graphstorage(&c))
            .filter(|gs| {
                if let Some(stats) = gs.get_statistics() {
                    stats.nodes > 0
                } else {
                    true
                }
            })
            .collect();

        let left_edges = graph
            .get_graphstorage(&COMPONENT_LEFT)
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(COMPONENT_LEFT.to_string()))?;
        let right_edges = graph
            .get_graphstorage(&COMPONENT_RIGHT)
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(COMPONENT_RIGHT.to_string()))?;

        let ordering_edges = graph
            .get_graphstorage(&COMPONENT_ORDERING)
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(COMPONENT_ORDERING.to_string()))?;

        Ok(TokenHelper {
            node_annos: graph.get_node_annos(),
            left_edges,
            right_edges,
            cov_edges,
            ordering_edges,
        })
    }
    pub fn get_gs_coverage(&self) -> &Vec<Arc<dyn GraphStorage>> {
        &self.cov_edges
    }

    pub fn get_gs_left_token(&self) -> &dyn GraphStorage {
        self.left_edges.as_ref()
    }

    pub fn get_gs_right_token(&self) -> &dyn GraphStorage {
        self.right_edges.as_ref()
    }

    pub fn is_token(&self, id: NodeID) -> Result<bool> {
        if self.node_annos.has_value_for_item(&id, &TOKEN_KEY)? {
            // check if there is no outgoing edge in any of the coverage components
            let has_outgoing = self.has_outgoing_coverage_edges(id)?;
            Ok(!has_outgoing)
        } else {
            Ok(false)
        }
    }

    pub fn has_outgoing_coverage_edges(&self, id: NodeID) -> Result<bool> {
        for c in self.cov_edges.iter() {
            if c.has_outgoing_edges(id)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn right_token_for(&self, n: NodeID) -> Result<Option<NodeID>> {
        if self.is_token(n)? {
            Ok(Some(n))
        } else {
            let mut out = self.right_edges.get_outgoing_edges(n);
            match out.next() {
                Some(out) => Ok(Some(out?)),
                None => Ok(None),
            }
        }
    }

    pub fn right_token_for_group(&self, nodes: &[NodeID]) -> Result<Option<NodeID>> {
        // Collect the right token for all the nodes
        let right_token: Result<Vec<NodeID>> = nodes
            .iter()
            .map(|n| self.right_token_for(*n))
            .filter_map_ok(|t| t)
            .collect();
        let right_token = right_token?;
        if right_token.is_empty() {
            Ok(None)
        } else {
            // Get the right-most token of the list
            let mut result = right_token[0];
            for i in 1..right_token.len() {
                if self.ordering_edges.is_connected(
                    result,
                    right_token[i],
                    1,
                    std::ops::Bound::Unbounded,
                )? {
                    result = right_token[i];
                }
            }
            Ok(Some(result))
        }
    }

    pub fn left_token_for(&self, n: NodeID) -> Result<Option<NodeID>> {
        if self.is_token(n)? {
            Ok(Some(n))
        } else {
            let mut out = self.left_edges.get_outgoing_edges(n);
            match out.next() {
                Some(out) => Ok(Some(out?)),
                None => Ok(None),
            }
        }
    }

    pub fn left_token_for_group(&self, nodes: &[NodeID]) -> Result<Option<NodeID>> {
        // Collect the left token for all the nodes
        let left_token: Result<Vec<NodeID>> = nodes
            .iter()
            .map(|n| self.left_token_for(*n))
            .filter_map_ok(|t| t)
            .collect();
        let left_token = left_token?;
        if left_token.is_empty() {
            Ok(None)
        } else {
            // Get the left-most token of the list
            let mut result = left_token[0];
            for i in 1..left_token.len() {
                if self.ordering_edges.is_connected(
                    left_token[i],
                    result,
                    1,
                    std::ops::Bound::Unbounded,
                )? {
                    result = left_token[i];
                }
            }
            Ok(Some(result))
        }
    }

    pub fn left_right_token_for(&self, n: NodeID) -> Result<(Option<NodeID>, Option<NodeID>)> {
        if self.is_token(n)? {
            Ok((Some(n), Some(n)))
        } else {
            let out_left = match self.left_edges.get_outgoing_edges(n).next() {
                Some(out) => Some(out?),
                None => None,
            };
            let out_right = match self.right_edges.get_outgoing_edges(n).next() {
                Some(out) => Some(out?),
                None => None,
            };

            Ok((out_left, out_right))
        }
    }
}
