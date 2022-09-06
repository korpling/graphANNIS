use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use graphannis_core::errors::GraphAnnisCoreError;
use graphannis_core::graph::storage::GraphStorage;
use graphannis_core::graph::{ANNIS_NS, DEFAULT_NS, NODE_NAME_KEY};
use graphannis_core::{
    annostorage::{Match, MatchGroup},
    errors::Result as CoreResult,
    graph::Graph,
    types::{Component, Edge, NodeID},
};
use smallvec::smallvec;

use crate::annis::db::token_helper::TokenHelper;
use crate::annis::errors::GraphAnnisError;
use crate::annis::util::quicksort;
use crate::try_as_option;
use crate::{annis::errors::Result, model::AnnotationComponentType, AnnotationGraph};

struct TokenIterator<'a> {
    end_token: NodeID,
    current_token: Option<NodeID>,
    covering_nodes: Box<dyn Iterator<Item = NodeID>>,
    token_helper: TokenHelper<'a>,
    ordering_edges: Arc<dyn GraphStorage>,
}

impl<'a> TokenIterator<'a> {
    fn calculate_covering_nodes(&mut self) -> Result<()> {
        let mut covering_nodes = HashSet::new();

        if let Some(current_token) = self.current_token {
            // get all nodes that are covering the token (in all coverage components)
            for gs_cov in self.token_helper.get_gs_coverage().iter() {
                for n in gs_cov.get_ingoing_edges(current_token) {
                    let n = n?;
                    covering_nodes.insert(n);
                }
            }
        }

        self.covering_nodes = Box::new(covering_nodes.into_iter());
        Ok(())
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Result<NodeID>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we still need to output some covering nodes for the current node
        if let Some(next_covering_node) = self.covering_nodes.next() {
            return Some(Ok(next_covering_node));
        }

        if let Some(old_current_token) = self.current_token {
            if old_current_token == self.end_token {
                self.current_token = None;
                return Some(Ok(old_current_token));
            }

            // Get the next token in the chain
            let out: CoreResult<Vec<NodeID>> = self
                .ordering_edges
                .get_outgoing_edges(old_current_token)
                .collect();
            match out {
                Ok(out) => {
                    let next_token = out.into_iter().next();
                    self.current_token = next_token;
                    try_as_option!(self.calculate_covering_nodes());

                    // Return the previous current token
                    return Some(Ok(old_current_token));
                }
                Err(e) => {
                    return Some(Err(e.into()));
                }
            }
        }

        None
    }
}

fn get_left_right_token_with_offset_with_token(
    ordering_edges: &dyn GraphStorage,
    ordered_covered_token: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
) -> Result<(NodeID, NodeID)> {
    // Get the token with the correct offset from the left-most and right-most covered token
    let left_most_covered_token = *ordered_covered_token
        .first()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;
    let right_most_covered_token = *ordered_covered_token
        .last()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;

    // The context might be larger than the actual document, try to get the
    // largest possible context
    let mut left_with_context = left_most_covered_token;
    for ctx in (0..=ctx_left).rev() {
        if let Some(result) = ordering_edges
            .find_connected_inverse(left_most_covered_token, ctx, std::ops::Bound::Included(ctx))
            .next()
        {
            left_with_context = result?;
            break;
        }
    }
    let mut right_with_context = right_most_covered_token;
    for ctx in (0..=ctx_right).rev() {
        if let Some(result) = ordering_edges
            .find_connected(
                right_most_covered_token,
                ctx,
                std::ops::Bound::Included(ctx),
            )
            .next()
        {
            right_with_context = result?;
            break;
        }
    }
    Ok((left_with_context, right_with_context))
}

fn get_left_right_token_with_offset_with_segmentation(
    graph: &Graph<AnnotationComponentType>,
    token_helper: &TokenHelper,
    ordered_covered_token: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
    segmentation: &str,
) -> Result<(NodeID, NodeID)> {
    let left_most_covered_token = *ordered_covered_token
        .first()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;
    let right_most_covered_token = *ordered_covered_token
        .last()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;

    // Get the ordering component for this segmentation
    let component_ordering = Component::new(
        AnnotationComponentType::Ordering,
        DEFAULT_NS.into(),
        segmentation.into(),
    );
    let gs_ordering = graph.get_graphstorage_as_ref(&component_ordering).ok_or(
        GraphAnnisCoreError::MissingComponent(component_ordering.to_string()),
    )?;
    // Get all nodes covering the token and that are part of
    // the matching ordering component
    let mut covering_segmentation_nodes = HashSet::new();
    for gs_cov in token_helper.get_gs_coverage() {
        for token in ordered_covered_token {
            for n in gs_cov.get_ingoing_edges(*token) {
                let n = n?;
                if let Some(first_outgoing_edge) = gs_ordering.get_outgoing_edges(n).next() {
                    first_outgoing_edge?;
                    covering_segmentation_nodes.insert(n);
                } else if let Some(first_incoming_edge) = gs_ordering.get_ingoing_edges(n).next() {
                    first_incoming_edge?;
                    covering_segmentation_nodes.insert(n);
                }
            }
        }
    }
    // Sort the covering segmentation nodes by their ordering edges
    let mut covering_segmentation_nodes: Vec<_> = covering_segmentation_nodes.into_iter().collect();
    quicksort::sort(&mut covering_segmentation_nodes, |a, b| {
        if a == b {
            Ok(Ordering::Equal)
        } else if gs_ordering.is_connected(*a, *b, 1, std::ops::Bound::Unbounded)? {
            Ok(Ordering::Less)
        } else if gs_ordering.is_connected(*b, *a, 1, std::ops::Bound::Unbounded)? {
            Ok(Ordering::Greater)
        } else {
            Ok(a.cmp(b))
        }
    })?;

    let left_seg = *covering_segmentation_nodes
        .first()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;
    let right_seg = *covering_segmentation_nodes
        .last()
        .ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;

    // The context might be larger than the actual document, try to get the
    // largest possible context
    let mut left_with_context = left_seg;
    for ctx in (0..=ctx_left).rev() {
        if let Some(result) = gs_ordering
            .find_connected_inverse(left_seg, ctx, std::ops::Bound::Included(ctx))
            .next()
        {
            left_with_context = result?;
            break;
        }
    }
    let mut right_with_context = right_seg;
    for ctx in (0..=ctx_right).rev() {
        if let Some(result) = gs_ordering
            .find_connected(right_seg, ctx, std::ops::Bound::Included(ctx))
            .next()
        {
            right_with_context = result?;
            break;
        }
    }
    let left_token = token_helper
        .left_token_for(left_with_context)?
        .unwrap_or(left_most_covered_token);
    let right_token = token_helper
        .right_token_for(right_with_context)?
        .unwrap_or(right_most_covered_token);
    Ok((left_token, right_token))
}

#[derive(Clone)]
struct TokenRegion<'a> {
    start_token: NodeID,
    end_token: NodeID,
    token_helper: TokenHelper<'a>,
    ordering_edges: Arc<dyn GraphStorage>,
}

impl<'a> TokenRegion<'a> {
    fn from_node_with_context(
        graph: &'a Graph<AnnotationComponentType>,
        node_id: NodeID,
        ctx_left: usize,
        ctx_right: usize,
        segmentation: Option<String>,
    ) -> Result<TokenRegion<'a>> {
        let token_helper = TokenHelper::new(graph)?;
        let ordering_edges = graph
            .get_graphstorage(&Component::new(
                AnnotationComponentType::Ordering,
                ANNIS_NS.into(),
                "".into(),
            ))
            .ok_or(GraphAnnisCoreError::MissingComponent(
                "Ordering/annis/".into(),
            ))?;
        // Get all covered token for this node
        let mut covered_token = HashSet::new();
        if token_helper.is_token(node_id)? {
            covered_token.insert(node_id);
        } else {
            for gs_cov in token_helper.get_gs_coverage() {
                for candidate in gs_cov.get_outgoing_edges(node_id) {
                    let candidate = candidate?;
                    if token_helper.is_token(candidate)? {
                        covered_token.insert(candidate);
                    }
                }
            }
        }
        // Sort the covered token by their position in the text
        let mut covered_token: Vec<_> = covered_token.into_iter().collect();
        quicksort::sort(&mut covered_token, |a, b| {
            if a == b {
                Ok(Ordering::Equal)
            } else if ordering_edges.is_connected(*a, *b, 1, std::ops::Bound::Unbounded)? {
                Ok(Ordering::Less)
            } else if ordering_edges.is_connected(*b, *a, 1, std::ops::Bound::Unbounded)? {
                Ok(Ordering::Greater)
            } else {
                Ok(a.cmp(b))
            }
        })?;

        // Get the token at the borders of the context
        let (start_token, end_token) = if let Some(segmentation) = &segmentation {
            get_left_right_token_with_offset_with_segmentation(
                graph,
                &token_helper,
                &covered_token,
                ctx_left,
                ctx_right,
                segmentation,
            )?
        } else {
            get_left_right_token_with_offset_with_token(
                ordering_edges.as_ref(),
                &covered_token,
                ctx_left,
                ctx_right,
            )?
        };

        Ok(TokenRegion {
            start_token,
            end_token,
            token_helper,
            ordering_edges,
        })
    }

    fn into_token_iterator_with_coverage(self) -> Result<TokenIterator<'a>> {
        let mut result = TokenIterator {
            current_token: Some(self.start_token),
            end_token: self.end_token,
            token_helper: self.token_helper,
            covering_nodes: Box::new(std::iter::empty()),
            ordering_edges: self.ordering_edges,
        };
        result.calculate_covering_nodes()?;
        Ok(result)
    }
}

/// Creates an iterator over all overlapped nodes of the match with gaps.
fn new_overlapped_nodes_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: &[NodeID],
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<u64>> + 'a>> {
    let mut regions: Vec<TokenRegion<'a>> = Vec::default();
    for n in node_ids {
        let token_region = TokenRegion::from_node_with_context(
            graph,
            *n,
            ctx_left,
            ctx_right,
            segmentation.clone(),
        );
        match token_region {
            Ok(token_region) => regions.push(token_region),
            Err(e) => match e {
                // Ignore match nodes without covered token in this itertor
                GraphAnnisError::NoCoveredTokenForSubgraph => {}
                _ => return Err(e),
            },
        };
    }
    let component_ordering = Component::new(
        AnnotationComponentType::Ordering,
        ANNIS_NS.into(),
        "".into(),
    );
    let gs_ordering = graph.get_graphstorage(&component_ordering).ok_or(
        GraphAnnisCoreError::MissingComponent(component_ordering.to_string()),
    )?;
    // Sort regions by position in the text, so the iterator of nodes
    // keeps the order of the nodes as given by the text.
    quicksort::sort(&mut regions, move |a, b| {
        if a.start_token == b.start_token && a.end_token == b.end_token {
            Ok(Ordering::Equal)
        } else if gs_ordering.is_connected(
            a.end_token,
            b.start_token,
            1,
            std::ops::Bound::Unbounded,
        )? {
            Ok(Ordering::Less)
        } else if gs_ordering.is_connected(
            b.end_token,
            a.start_token,
            1,
            std::ops::Bound::Unbounded,
        )? {
            Ok(Ordering::Greater)
        } else {
            Ok((a.start_token, a.end_token).cmp(&(b.start_token, b.end_token)))
        }
    })?;

    // Create and chain all iterators of the vector
    let mut iterators = Vec::new();
    for token_region in regions.into_iter() {
        iterators.push(token_region.into_token_iterator_with_coverage()?);
    }
    let result = iterators.into_iter().flat_map(|it| it);
    Ok(Box::new(result))
}

/// Creates an iterator over all parent nodes of the matched nodes in the
/// corpus graph, including data sources.
fn new_parent_nodes_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: &[NodeID],
) -> Result<Box<dyn Iterator<Item = Result<u64>> + 'a>> {
    let component_part_of =
        Component::new(AnnotationComponentType::PartOf, ANNIS_NS.into(), "".into());
    let gs_part_of =
        graph
            .get_graphstorage(&component_part_of)
            .ok_or(GraphAnnisCoreError::MissingComponent(
                component_part_of.to_string(),
            ))?;
    let mut parents = HashSet::new();
    for n in node_ids {
        for p in gs_part_of.find_connected(*n, 1, std::ops::Bound::Unbounded) {
            let p = p?;
            parents.insert(p);
        }
    }
    Ok(Box::new(parents.into_iter().map(|p| Ok(p))))
}

pub fn new_subgraph_iterator<'a>(
    graph: &'a Graph<AnnotationComponentType>,
    node_ids: Vec<String>,
    ctx_left: usize,
    ctx_right: usize,
    segmentation: Option<String>,
) -> Result<Box<dyn Iterator<Item = Result<MatchGroup>> + 'a>> {
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

    let overlapped_nodes =
        new_overlapped_nodes_iterator(graph, &node_ids, ctx_left, ctx_right, segmentation.clone())?;
    let parent_nodes = new_parent_nodes_iterator(graph, &node_ids)?;

    // Chain iterators into a single iterator, also include the matched node IDs
    // in the result
    let result = node_ids
        .into_iter()
        .map(|n| Ok(n))
        .chain(overlapped_nodes)
        .chain(parent_nodes)
        .map(|n| {
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

    let gs_orig_ordering = orig_graph.get_graphstorage(&Component::new(
        AnnotationComponentType::Ordering,
        ANNIS_NS.into(),
        "".into(),
    ));
    let ds_ordering_component = Component::new(
        AnnotationComponentType::Ordering,
        ANNIS_NS.into(),
        "datasource-gap".into(),
    );
    let token_helper = TokenHelper::new(&orig_graph)?;

    let mut previous_token = None;

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

                    if token_helper.is_token(m.node)? {
                        if let (Some(gs_ordering), Some(previous_node)) =
                            (&gs_orig_ordering, previous_token)
                        {
                            if let Some(distance) = gs_ordering.distance(previous_node, m.node)? {
                                if distance > 1 {
                                    let gs_result_ds_ordering_ =
                                        result.get_or_create_writable(&ds_ordering_component)?;

                                    gs_result_ds_ordering_.add_edge(Edge {
                                        source: previous_node,
                                        target: m.node,
                                    })?;
                                }
                            }
                        }
                        previous_token = Some(m.node);
                    }

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
