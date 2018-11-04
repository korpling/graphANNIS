use annis::db::exec::Desc;
use annis::db::exec::ExecutionNode;
use annis::db::graphstorage::GraphStorage;
use annis::db::sort_matches;
use annis::db::token_helper;
use annis::db::token_helper::TokenHelper;
use annis::db::AnnotationStorage;
use annis::db::Graph;
use annis::db::Match;
use annis::errors::*;
use annis::types::AnnoKeyID;
use annis::types::{Component, ComponentType, NodeID};
use annis::util;

use std::fmt;

use rustc_hash::FxHashMap;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps the search for *all* token in a corpus.
pub struct AnyTokenSearch<'a> {
    desc: Option<Desc>,
    node_name_key: AnnoKeyID,
    db: &'a Graph,
    token_helper: Option<TokenHelper>,
    order_gs: Option<&'a GraphStorage>,
    root_iterators: Option<Vec<Box<Iterator<Item = NodeID> + 'a>>>,
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
        let token_helper = TokenHelper::new(db);

        Ok(AnyTokenSearch {
            order_gs,
            token_helper,
            db,
            desc: None,
            node_name_key: db
                .node_annos
                .get_key_id(&db.get_node_name_key())
                .unwrap_or_default(),
            root_iterators: None,
        })
    }

    pub fn necessary_components() -> Vec<Component> {
        let mut components = token_helper::necessary_components();
        components.push(COMPONENT_ORDER.clone());
        components
    }

    fn get_root_iterators(&mut self) -> &mut Vec<Box<Iterator<Item = NodeID> + 'a>> {
        if let Some(ref mut root_iterators) = self.root_iterators {
            return root_iterators;
        } else {
            // iterate over all nodes that are token and check if they are root node nodes in the ORDERING component
            let mut root_nodes: Vec<Match> = Vec::new();
            for tok_candidate in self.db.node_annos.exact_anno_search(
                Some("annis".to_owned()),
                "tok".to_owned(),
                None,
            ) {
                let n = tok_candidate.node;
                let mut is_root_tok = true;
                if let Some(order_gs) = self.order_gs {
                    is_root_tok = is_root_tok && order_gs.get_ingoing_edges(n).next() == None;
                }
                if let Some(ref token_helper) = self.token_helper {
                    is_root_tok = is_root_tok && token_helper.is_token(n);
                }
                if is_root_tok {
                    root_nodes.push(Match {
                        node: n,
                        anno_key: self.node_name_key,
                    });
                }
            
            }
            // Sort the root nodes by their reverse text position,
            // so that removing the last item will return the first root node.
            let mut node_to_path: FxHashMap<NodeID, (Vec<&str>, &str)> = FxHashMap::default();
            for m in &root_nodes {
                if let Some(path) = self
                    .db
                    .node_annos
                    .get_value_for_item_by_id(&m.node, self.node_name_key)
                {
                    node_to_path.insert(m.node, util::extract_node_path(path));
                }
            }

            root_nodes.sort_unstable_by(|a, b| {
                sort_matches::compare_match_by_text_pos(
                    b,
                    a,
                    &node_to_path,
                    self.token_helper.as_ref(),
                    self.order_gs,
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
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for AnyTokenSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let node_name_key: AnnoKeyID = self.node_name_key;
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
                        anno_key: node_name_key,
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
    use update::{GraphUpdate, UpdateEvent};

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
