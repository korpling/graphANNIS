use crate::annis::db::exec::ExecutionNode;
use crate::annis::db::exec::ExecutionNodeDesc;
use crate::annis::db::sort_matches;
use crate::annis::db::sort_matches::CollationType;
use crate::annis::db::token_helper;
use crate::{
    annis::db::aql::model::AnnotationComponentType, annis::db::token_helper::TokenHelper,
    graph::Match, AnnotationGraph,
};
use graphannis_core::{
    annostorage::MatchGroup,
    errors::Result,
    graph::{storage::GraphStorage, ANNIS_NS, NODE_TYPE_KEY},
    types::{AnnoKey, Component, NodeID},
};

use smallvec::smallvec;
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps the search for *all* token in a corpus.
pub struct AnyTokenSearch<'a> {
    desc: Option<ExecutionNodeDesc>,
    node_type_key: Arc<AnnoKey>,
    db: &'a AnnotationGraph,
    token_helper: Option<TokenHelper<'a>>,
    order_gs: Option<&'a dyn GraphStorage>,
    root_iterators: Option<Vec<Box<dyn Iterator<Item = NodeID> + 'a>>>,
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

impl<'a> AnyTokenSearch<'a> {
    pub fn new(db: &'a AnnotationGraph) -> Result<AnyTokenSearch<'a>> {
        let order_gs = db.get_graphstorage_as_ref(&COMPONENT_ORDER);

        Ok(AnyTokenSearch {
            order_gs,
            db,
            token_helper: TokenHelper::new(db),
            desc: None,
            node_type_key: NODE_TYPE_KEY.clone(),
            root_iterators: None,
        })
    }

    pub fn necessary_components(
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut components = token_helper::necessary_components(db);
        components.insert(COMPONENT_ORDER.clone());
        components
    }

    fn get_root_iterators(&mut self) -> &mut Vec<Box<dyn Iterator<Item = NodeID> + 'a>> {
        if let Some(ref mut root_iterators) = self.root_iterators {
            root_iterators
        } else {
            // iterate over all nodes that are token and check if they are root node nodes in the ORDERING component
            let mut root_nodes = MatchGroup::new();
            for tok_candidate in
                self.db
                    .get_node_annos()
                    .exact_anno_search(Some("annis"), "tok", None.into())
            {
                let n = tok_candidate.node;
                let mut is_root_tok = true;
                if let Some(order_gs) = self.order_gs {
                    is_root_tok = is_root_tok && order_gs.get_ingoing_edges(n).next() == None;
                }
                if let Some(ref token_helper) = self.token_helper {
                    is_root_tok = is_root_tok && !token_helper.has_outgoing_coverage_edges(n);
                }
                if is_root_tok {
                    root_nodes.push(Match {
                        node: n,
                        anno_key: self.node_type_key.clone(),
                    });
                }
            }
            root_nodes.sort_unstable_by(|a, b| {
                sort_matches::compare_match_by_text_pos(
                    b,
                    a,
                    self.db.get_node_annos(),
                    self.token_helper.as_ref(),
                    self.order_gs,
                    CollationType::Default,
                    false,
                )
            });

            // for root nodes add an iterator for all reachable nodes in the order component
            let mut root_iterators = Vec::new();
            for root in root_nodes {
                let it = if let Some(order_gs) = self.order_gs {
                    order_gs.find_connected(root.node, 0, std::ops::Bound::Unbounded)
                } else {
                    // there is only the the root token and no ordering component
                    Box::from(vec![root.node].into_iter())
                };
                root_iterators.push(it);
            }
            self.root_iterators = Some(root_iterators);
            self.root_iterators.as_mut().unwrap()
        }
    }
}

impl<'a> fmt::Display for AnyTokenSearch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tok")
    }
}

impl<'a> ExecutionNode for AnyTokenSearch<'a> {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = MatchGroup> {
        self
    }

    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for AnyTokenSearch<'a> {
    type Item = MatchGroup;

    fn next(&mut self) -> Option<MatchGroup> {
        // lazily initialize the sorted vector of iterators
        let root_iterators = self.get_root_iterators();
        // use the last iterator in the list to get the next match
        while !root_iterators.is_empty() {
            {
                let root_iterators_len = root_iterators.len();
                let it = &mut root_iterators[root_iterators_len - 1];
                if let Some(n) = it.next() {
                    return Some(smallvec![Match {
                        node: n,
                        anno_key: self.node_type_key.clone(),
                    }]);
                }
            }
            root_iterators.pop();
        }

        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::update::{GraphUpdate, UpdateEvent};

    #[test]
    fn find_with_only_one_token() {
        let mut g = AnnotationGraph::with_default_graphstorages(false).unwrap();

        let mut update = GraphUpdate::new();
        update
            .add_event(UpdateEvent::AddNode {
                node_name: "doc1/tok1".to_owned(),
                node_type: "node".to_owned(),
            })
            .unwrap();
        update
            .add_event(UpdateEvent::AddNodeLabel {
                node_name: "doc1/tok1".to_owned(),
                anno_ns: "annis".to_owned(),
                anno_name: "tok".to_owned(),
                anno_value: "The".to_owned(),
            })
            .unwrap();

        g.apply_update(&mut update, |_| {}).unwrap();

        let search_result: Vec<MatchGroup> = AnyTokenSearch::new(&g).unwrap().collect();
        assert_eq!(1, search_result.len());
    }
}
