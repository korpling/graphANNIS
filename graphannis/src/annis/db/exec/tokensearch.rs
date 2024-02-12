use crate::annis::db::exec::ExecutionNode;
use crate::annis::db::exec::ExecutionNodeDesc;
use crate::annis::db::sort_matches::CollationType;
use crate::annis::db::sort_matches::SortCache;
use crate::annis::db::token_helper;
use crate::annis::util::quicksort;
use crate::{
    annis::db::aql::model::AnnotationComponentType, annis::db::token_helper::TokenHelper,
    errors::Result, graph::Match, AnnotationGraph,
};
use graphannis_core::{
    annostorage::MatchGroup,
    graph::{storage::GraphStorage, ANNIS_NS, NODE_TYPE_KEY},
    types::{AnnoKey, Component, NodeID},
};
use itertools::Itertools;
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
    root_iterators: Option<Vec<Box<dyn Iterator<Item = Result<NodeID>> + 'a>>>,
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
        let token_helper = TokenHelper::new(db).ok();

        Ok(AnyTokenSearch {
            order_gs,
            db,
            token_helper,
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

    fn is_root_tok(&self, n: NodeID) -> Result<bool> {
        let no_ingoing_edges = if let Some(order_gs) = self.order_gs {
            !order_gs.has_ingoing_edges(n)?
        } else {
            true
        };

        if no_ingoing_edges {
            if let Some(ref token_helper) = self.token_helper {
                let no_outgoing_coverage = !token_helper.has_outgoing_coverage_edges(n)?;
                Ok(no_outgoing_coverage)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    fn create_new_root_iterator(
        &self,
    ) -> Result<Vec<Box<dyn Iterator<Item = Result<NodeID>> + 'a>>> {
        // iterate over all nodes that are token and check if they are root node nodes in the ORDERING component
        let root_nodes: Result<Vec<_>> = self
            .db
            .get_node_annos()
            .exact_anno_search(Some("annis"), "tok", None.into())
            .map(|tok_candidate| -> Result<Option<Match>> {
                let tok_candidate = tok_candidate?;
                if self.is_root_tok(tok_candidate.node)? {
                    Ok(Some(tok_candidate))
                } else {
                    Ok(None)
                }
            })
            .filter_map_ok(|m| m)
            .collect();
        let mut root_nodes = root_nodes?;

        let mut cache = SortCache::new(self.db.get_graphstorage(&COMPONENT_ORDER));

        quicksort::sort(&mut root_nodes, |a, b| {
            cache.compare_match_by_text_pos(
                b,
                a,
                self.db.get_node_annos(),
                self.token_helper.as_ref(),
                CollationType::Default,
                false,
            )
        })?;

        // for root nodes add an iterator for all reachable nodes in the order component
        let mut root_iterators: Vec<Box<dyn Iterator<Item = Result<u64>>>> = Vec::new();
        for root in root_nodes {
            let it = if let Some(order_gs) = self.order_gs {
                order_gs.find_connected(root.node, 0, std::ops::Bound::Unbounded)
            } else {
                // there is only the the root token and no ordering component
                Box::from(vec![Ok(root.node)].into_iter())
            };
            root_iterators.push(Box::new(it.map(|it| it.map_err(|e| e.into()))));
        }

        Ok(root_iterators)
    }

    fn get_root_iterators(&mut self) -> &mut Vec<Box<dyn Iterator<Item = Result<NodeID>> + 'a>> {
        if let Some(ref mut root_iterators) = self.root_iterators {
            root_iterators
        } else {
            match self.create_new_root_iterator() {
                Ok(root_iterators) => {
                    self.root_iterators = Some(root_iterators);
                }
                Err(e) => {
                    // Set the internal cache to a failure state
                    let err_iterator = std::iter::once(Err(e));
                    self.root_iterators = Some(vec![Box::new(err_iterator)]);
                }
            };
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
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = Result<MatchGroup>> {
        self
    }

    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for AnyTokenSearch<'a> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Result<MatchGroup>> {
        // lazily initialize the sorted vector of iterators
        let root_iterators = self.get_root_iterators();
        // use the last iterator in the list to get the next match
        while !root_iterators.is_empty() {
            {
                let root_iterators_len = root_iterators.len();
                let it = &mut root_iterators[root_iterators_len - 1];
                if let Some(n) = it.next() {
                    let result: Option<Result<MatchGroup>> = match n {
                        Ok(n) => Some(Ok(smallvec![Match {
                            node: n,
                            anno_key: self.node_type_key.clone(),
                        }])),
                        Err(e) => Some(Err(e)),
                    };
                    return result;
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

        let search_result: Result<Vec<MatchGroup>> = AnyTokenSearch::new(&g).unwrap().collect();
        assert_eq!(1, search_result.unwrap().len());
    }
}
