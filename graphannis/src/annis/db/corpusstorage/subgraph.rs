use std::collections::BTreeSet;

use graphannis_core::graph::NODE_NAME_KEY;
use graphannis_core::{
    annostorage::{Match, MatchGroup},
    graph::Graph,
    types::{Component, Edge, NodeID},
};
use smallvec::smallvec;

use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::{annis::errors::Result, model::AnnotationComponentType, AnnotationGraph};

/// Creates a new iterator over all token of the match with the context without gaps.
fn new_token_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    token_helper: TokenHelper<'a>,
    node_ids: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<u64>> + 'a>> {
    let left_without_context = token_helper.left_token_for_group(node_ids)?;
    let right_without_context = token_helper.right_token_for_group(node_ids)?;

    todo!()
}

/// Creates an iterator over all overlapped non-token nodes of the match with gaps.
fn new_overlapped_nodes_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<u64>> + 'a>> {
    todo!()
}

/// Creates an iterator over all parent nodes of the matched nodes in the corpus graph
fn new_parent_nodes_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<u64>> + 'a>> {
    todo!()
}

pub fn new_subgraph_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: Vec<String>,
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<MatchGroup>> + 'a>> {
    let token_helper = TokenHelper::new(graph)?;

    // Get the node IDs for the whole match
    let node_ids: Result<Vec<NodeID>> = node_ids
        .into_iter()
        .map(|node_name| {
            let id = graph.get_node_id_from_name(&node_name)?;
            let id = id.ok_or(GraphAnnisError::NoSuchNodeID(node_name))?;
            Ok(id)
        })
        .collect();
    let node_ids = node_ids?;

    let tokens = new_token_iterator(
        graph,
        token_helper,
        &node_ids,
        ctx_left,
        ctx_right,
        segmentation.clone(),
    )?;
    let overlapped_nodes =
        new_overlapped_nodes_iterator(graph, &node_ids, ctx_left, ctx_right, segmentation.clone())?;
    let parent_nodes =
        new_parent_nodes_iterator(graph, &node_ids, ctx_left, ctx_right, segmentation)?;

    // Chain iterators into a single iterator
    let result = tokens.chain(overlapped_nodes).chain(parent_nodes).map(|n| {
        let n = n?;
        let m: MatchGroup = smallvec![Match {
            node: n,
            anno_key: NODE_NAME_KEY.clone(),
        }];
        Ok(m)
    });
    Ok(Box::new(result))
}

pub fn create_subgraph_for_iterator<I>(
    it: I,
    match_idx: &[usize],
    orig_graph: &Graph<AnnotationComponentType>,
    component_type_filter: Option<AnnotationComponentType>,
) -> Result<AnnotationGraph>
where
    I: Iterator<Item = Result<MatchGroup>>,
{
    // We have to keep our own unique set because the query will return "duplicates" whenever the other parts of the
    // match vector differ.
    let mut match_result: BTreeSet<Match> = BTreeSet::new();

    let mut result = AnnotationGraph::new(false)?;

    // create the subgraph description
    for r in it {
        let r = r?;
        trace!("subgraph query found match {:?}", r);
        for i in match_idx.iter().cloned() {
            if i < r.len() {
                let m: &Match = &r[i];
                if !match_result.contains(m) {
                    match_result.insert(m.clone());
                    trace!("subgraph query extracted node {:?}", m.node);
                    create_subgraph_node(m.node, &mut result, orig_graph)?;
                }
            }
        }
    }

    let components = orig_graph.get_all_components(component_type_filter, None);

    for m in &match_result {
        create_subgraph_edge(m.node, &mut result, orig_graph, &components)?;
    }

    Ok(result)
}

fn create_subgraph_node(
    id: NodeID,
    db: &mut AnnotationGraph,
    orig_db: &AnnotationGraph,
) -> Result<()> {
    // add all node labels with the same node ID
    for a in orig_db.get_node_annos().get_annotations_for_item(&id)? {
        db.get_node_annos_mut().insert(id, a)?;
    }
    Ok(())
}
fn create_subgraph_edge(
    source_id: NodeID,
    db: &mut AnnotationGraph,
    orig_db: &AnnotationGraph,
    components: &[Component<AnnotationComponentType>],
) -> Result<()> {
    // find outgoing edges
    for c in components {
        // don't include index components
        let ctype = c.get_type();
        if !((ctype == AnnotationComponentType::Coverage
            && c.layer == "annis"
            && !c.name.is_empty())
            || ctype == AnnotationComponentType::RightToken
            || ctype == AnnotationComponentType::LeftToken)
        {
            if let Some(orig_gs) = orig_db.get_graphstorage(c) {
                for target in orig_gs.get_outgoing_edges(source_id) {
                    let target = target?;
                    if !db
                        .get_node_annos()
                        .get_all_keys_for_item(&target, None, None)?
                        .is_empty()
                    {
                        let e = Edge {
                            source: source_id,
                            target,
                        };
                        if let Ok(new_gs) = db.get_or_create_writable(c) {
                            new_gs.add_edge(e.clone())?;
                        }

                        for a in orig_gs.get_anno_storage().get_annotations_for_item(&Edge {
                            source: source_id,
                            target,
                        })? {
                            if let Ok(new_gs) = db.get_or_create_writable(c) {
                                new_gs.add_edge_annotation(e.clone(), a)?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
