use crate::annis::db::exec::Desc;
use crate::annis::db::exec::ExecutionNode;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::sort_matches;
use crate::annis::db::sort_matches::CollationType;
use crate::annis::db::token_helper;
use crate::annis::db::{AnnotationStorage, Graph, Match, NODE_TYPE_KEY};
use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, Component, ComponentType, NodeID};

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps the search for *all* token in a corpus.
pub struct AnyTokenSearch<'a> {
    desc: Option<Desc>,
    node_type_key: Arc<AnnoKey>,
    db: &'a Graph,
    order_gs: Option<&'a dyn GraphStorage>,
    root_iterators: Option<Vec<Box<dyn Iterator<Item = NodeID> + 'a>>>,
}

lazy_static! {
    static ref COMPONENT_ORDER: Component = {
        Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl<'a> AnyTokenSearch<'a> {
    pub fn new(db: &'a Graph) -> Result<AnyTokenSearch<'a>> {
        let order_gs = db.get_graphstorage_as_ref(&COMPONENT_ORDER);

        Ok(AnyTokenSearch {
            order_gs,
            db,
            desc: None,
            node_type_key: NODE_TYPE_KEY.clone(),
            root_iterators: None,
        })
    }

    pub fn necessary_components(db: &Graph) -> HashSet<Component> {
        let mut components = token_helper::necessary_components(db);
        components.insert(COMPONENT_ORDER.clone());
        components
    }

    fn get_root_iterators(&mut self) -> &mut Vec<Box<dyn Iterator<Item = NodeID> + 'a>> {
        if let Some(ref mut root_iterators) = self.root_iterators {
            return root_iterators;
        } else {
            // iterate over all nodes that are token and check if they are root node nodes in the ORDERING component
            let mut root_nodes: Vec<Match> = Vec::new();
            for tok_candidate in
                self.db
                    .node_annos
                    .exact_anno_search(Some("annis"), "tok", None.into())
            {
                let n = tok_candidate.node;
                let mut is_root_tok = true;
                if let Some(order_gs) = self.order_gs {
                    is_root_tok = is_root_tok && order_gs.get_ingoing_edges(n).next() == None;
                }
                if let Some(ref token_helper) = self.db.token_helper {
                    is_root_tok = is_root_tok && token_helper.is_token(n);
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
                    self.db.node_annos.as_ref(),
                    self.db.token_helper.as_ref(),
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
            return self.root_iterators.as_mut().unwrap();
        }
    }
}

impl<'a> fmt::Display for AnyTokenSearch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tok")
    }
}

impl<'a> ExecutionNode for AnyTokenSearch<'a> {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for AnyTokenSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        // lazily initialize the sorted vector of iterators
        let root_iterators = self.get_root_iterators();
        // use the last iterator in the list to get the next match
        while !root_iterators.is_empty() {
            {
                let root_iterators_len = root_iterators.len();
                let it = &mut root_iterators[root_iterators_len - 1];
                if let Some(n) = it.next() {
                    return Some(vec![Match {
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
        let mut g = Graph::new();

        let mut update = GraphUpdate::new();
        update.add_event(UpdateEvent::AddNode {
            node_name: "doc1/tok1".to_owned(),
            node_type: "node".to_owned(),
        });
        update.add_event(UpdateEvent::AddNodeLabel {
            node_name: "doc1/tok1".to_owned(),
            anno_ns: "annis".to_owned(),
            anno_name: "tok".to_owned(),
            anno_value: "The".to_owned(),
        });
        update.finish();

        g.apply_update(&mut update).unwrap();

        let search_result: Vec<Vec<Match>> = AnyTokenSearch::new(&g).unwrap().collect();
        assert_eq!(1, search_result.len());
    }
}
