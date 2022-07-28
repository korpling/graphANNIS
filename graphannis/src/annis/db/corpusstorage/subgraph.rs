use std::collections::BTreeSet;
use std::sync::Arc;

use graphannis_core::errors::{GraphAnnisCoreError, Result as CoreResult};
use graphannis_core::graph::storage::GraphStorage;
use graphannis_core::{
    annostorage::{Match, MatchGroup},
    graph::{Graph, ANNIS_NS, DEFAULT_NS},
    types::{Component, Edge, NodeID},
};

use crate::{
    annis::{
        db::token_helper,
        errors::{GraphAnnisError, Result},
    },
    model::AnnotationComponentType,
    AnnotationGraph,
};

pub struct SubgraphIterator<'a> {
    graph: &'a Graph<AnnotationComponentType>,
    gs_segmentation_order: Arc<dyn GraphStorage>,
    context_start_node: u64,
}

impl<'a> SubgraphIterator<'a> {
    pub fn new(
        graph: &'a Graph<AnnotationComponentType>,
        node_ids: Vec<String>,
        ctx_left: usize,
        ctx_right: usize,
        segmentation: Option<String>,
    ) -> Result<SubgraphIterator<'a>> {
        let token_order_component = Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        );

        let gs_token_order = graph.get_graphstorage(&token_order_component).ok_or(
            GraphAnnisCoreError::MissingComponent(token_order_component.to_string()),
        )?;

        // Find left-most and right-most covered token of the match
        let token_helper = token_helper::TokenHelper::new(graph)?;

        let mut left_most_token = None;
        let mut right_most_token = None;

        for node_name in node_ids {
            if let Some(n) = graph.get_node_id_from_name(&node_name)? {
                let (left, right) = token_helper.left_right_token_for(n)?;
                let left = left
                    .ok_or_else(|| GraphAnnisError::NoCoveredTokenForNode(node_name.to_string()))?;
                let right = right
                    .ok_or_else(|| GraphAnnisError::NoCoveredTokenForNode(node_name.to_string()))?;
                if let Some(current_left_most_token) = left_most_token {
                    // Compare the current left-most token with the new candidate
                    if gs_token_order.is_connected(
                        left,
                        current_left_most_token,
                        1,
                        std::ops::Bound::Unbounded,
                    )? {
                        left_most_token = Some(left);
                    }
                } else {
                    left_most_token = Some(left)
                }

                if let Some(current_right_most_token) = right_most_token {
                    // Compare the current right-most token with the new candidate
                    if gs_token_order.is_connected(
                        current_right_most_token,
                        right,
                        1,
                        std::ops::Bound::Unbounded,
                    )? {
                        right_most_token = Some(right);
                    }
                } else {
                    right_most_token = Some(right)
                }
            }
        }

        let left_most_token = left_most_token.ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;
        let right_most_token =
            right_most_token.ok_or(GraphAnnisError::NoCoveredTokenForSubgraph)?;

        // Find the first node of the left context to start the iterator from
        let gs_segmentation_order = if let Some(segmentation) = &segmentation {
            let segmentation_order_component = Component::new(
                AnnotationComponentType::Ordering,
                DEFAULT_NS.into(),
                segmentation.into(),
            );
            let gs = graph
                .get_graphstorage(&segmentation_order_component)
                .ok_or(GraphAnnisCoreError::MissingComponent(
                    segmentation_order_component.to_string(),
                ))?;
            gs
        } else {
            gs_token_order
        };

        let left_most_seg_node = if let Some(segmentation) = segmentation {
            // Get the segmentation node that covers this token
            let result: CoreResult<Vec<_>> = token_helper
                .get_gs_left_token()
                .get_ingoing_edges(left_most_token)
                .collect();
            let result = result?;
            result[0]
        } else {
            left_most_token
        };

        let context_start_node = gs_segmentation_order
            .find_connected_inverse(
                left_most_seg_node,
                ctx_left,
                std::ops::Bound::Included(ctx_left),
            )
            .next()
            .unwrap_or_else(|| Ok(left_most_seg_node))?;

        let it = SubgraphIterator {
            graph,
            gs_segmentation_order,
            context_start_node,
        };
        Ok(it)
    }
}

impl<'a> Iterator for SubgraphIterator<'a> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        // // find all nodes covering the same token
        // for source_node_id in node_ids {
        //     // remove the obsolete "salt:/" prefix
        //     let source_node_id: &str = source_node_id
        //         .strip_prefix("salt:/")
        //         .unwrap_or(&source_node_id);

        //     let m = NodeSearchSpec::ExactValue {
        //         ns: Some(ANNIS_NS.to_string()),
        //         name: NODE_NAME.to_string(),
        //         val: Some(source_node_id.to_string()),
        //         is_meta: false,
        //     };

        //     // nodes overlapping the match: m _o_ node
        //     {
        //         let mut q = Conjunction::new();
        //         let node_idx = q.add_node(NodeSearchSpec::AnyNode, None);
        //         let m_idx = q.add_node(m.clone(), None);
        //         q.add_operator(
        //             Arc::new(operators::OverlapSpec { reflexive: true }),
        //             &m_idx,
        //             &node_idx,
        //             false,
        //         )?;
        //         query.alternatives.push(q);
        //     }

        //     // token left/right and their overlapped nodes
        //     if let Some(ref segmentation) = segmentation {
        //         add_subgraph_precedence_with_segmentation(
        //             &mut query,
        //             ctx_left,
        //             segmentation,
        //             &m,
        //             true,
        //         )?;
        //         add_subgraph_precedence_with_segmentation(
        //             &mut query,
        //             ctx_right,
        //             segmentation,
        //             &m,
        //             false,
        //         )?;
        //     } else {
        //         add_subgraph_precedence(&mut query, ctx_left, &m, true)?;
        //         add_subgraph_precedence(&mut query, ctx_right, &m, false)?;
        //     }

        //     // add the textual data sources (which are not part of the corpus graph)
        //     {
        //         let mut q = Conjunction::new();
        //         let datasource_idx = q.add_node(
        //             NodeSearchSpec::ExactValue {
        //                 ns: Some(ANNIS_NS.to_string()),
        //                 name: NODE_TYPE.to_string(),
        //                 val: Some("datasource".to_string()),
        //                 is_meta: false,
        //             },
        //             None,
        //         );
        //         let m_idx = q.add_node(m.clone(), None);
        //         q.add_operator(
        //             Arc::new(operators::PartOfSubCorpusSpec {
        //                 dist: RangeSpec::Bound {
        //                     min_dist: 1,
        //                     max_dist: 1,
        //                 },
        //             }),
        //             &m_idx,
        //             &datasource_idx,
        //             false,
        //         )?;
        //         query.alternatives.push(q);
        //     }
        // }
        todo!()
    }
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
