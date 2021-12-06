use crate::{
    annis::db::{
        aql::model::{AnnotationComponentType, TOKEN_KEY},
        AnnotationStorage,
    },
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

    pub fn is_token(&self, id: NodeID) -> bool {
        if self.node_annos.has_value_for_item(&id, &TOKEN_KEY) {
            // check if there is no outgoing edge in any of the coverage components
            !self.has_outgoing_coverage_edges(id)
        } else {
            false
        }
    }

    pub fn has_outgoing_coverage_edges(&self, id: NodeID) -> bool {
        self.cov_edges.iter().any(|c| c.has_outgoing_edges(id))
    }

    pub fn right_token_for(&self, n: NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            Some(n)
        } else {
            let mut out = self.right_edges.get_outgoing_edges(n);
            out.next()
        }
    }

    pub fn left_token_for(&self, n: NodeID) -> Option<NodeID> {
        if self.is_token(n) {
            Some(n)
        } else {
            let mut out = self.left_edges.get_outgoing_edges(n);
            out.next()
        }
    }

    pub fn left_right_token_for(&self, n: NodeID) -> (Option<NodeID>, Option<NodeID>) {
        if self.is_token(n) {
            (Some(n), Some(n))
        } else {
            let mut out_left = self.left_edges.get_outgoing_edges(n);
            let mut out_right = self.right_edges.get_outgoing_edges(n);

            (out_left.next(), out_right.next())
        }
    }
}
