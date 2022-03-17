use crate::{
    annis::db::{
        aql::model::{AnnotationComponentType, TOKEN_KEY},
        AnnotationStorage,
    },
    errors::Result,
    graph::GraphStorage,
    AnnotationGraph,
};
use graphannis_core::{
    graph::ANNIS_NS,
    types::{Component, NodeID},
};

use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct TokenHelper<'a> {
    node_annos: &'a dyn AnnotationStorage<NodeID>,
    left_edges: Arc<dyn GraphStorage>,
    right_edges: Arc<dyn GraphStorage>,
    cov_edges: Vec<Arc<dyn GraphStorage>>,
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
    pub fn new(graph: &'a AnnotationGraph) -> Option<TokenHelper<'a>> {
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

        Some(TokenHelper {
            node_annos: graph.get_node_annos(),
            left_edges: graph.get_graphstorage(&COMPONENT_LEFT)?,
            right_edges: graph.get_graphstorage(&COMPONENT_RIGHT)?,
            cov_edges,
        })
    }
    pub fn get_gs_coverage(&self) -> &Vec<Arc<dyn GraphStorage>> {
        &self.cov_edges
    }

    pub fn get_gs_left_token(&self) -> &dyn GraphStorage {
        self.left_edges.as_ref()
    }

    pub fn get_gs_right_token_(&self) -> &dyn GraphStorage {
        self.right_edges.as_ref()
    }

    pub fn is_token(&self, id: NodeID) -> Result<bool> {
        if self.node_annos.has_value_for_item(&id, &TOKEN_KEY) {
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
