use super::super::{ExecutionNode, ExecutionNodeDesc, NodeSearchDesc};
use crate::annis::db::aql::conjunction::BinaryOperatorArguments;
use crate::annis::db::AnnotationStorage;
use crate::annis::operator::BinaryOperatorIndex;
use crate::{annis::operator::EstimationType, errors::Result, graph::Match};
use graphannis_core::{annostorage::MatchGroup, types::NodeID};
use itertools::Itertools;
use rayon::prelude::*;
use std::error::Error;
use std::iter::Peekable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 512;

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>>,
    match_receiver: Option<Receiver<Result<MatchGroup>>>,
    op: Arc<dyn BinaryOperatorIndex + 'a>,
    lhs_idx: usize,
    node_search_desc: Arc<NodeSearchDesc>,
    node_annos: &'a dyn AnnotationStorage<NodeID>,
    desc: ExecutionNodeDesc,
    global_reflexivity: bool,
}

impl<'a> IndexJoin<'a> {
    /// Create a new `IndexJoin`
    /// # Arguments
    ///
    /// * `lhs` - An iterator for a left-hand-side
    /// * `lhs_idx` - The index of the element in the LHS that should be used as a source
    /// * `op_entry` - The operator that connects the LHS and RHS (with description)
    /// * `anno_qname` A pair of the annotation namespace and name (both optional) to define which annotations to fetch
    /// * `anno_cond` - A filter function to determine if a RHS candidate is included
    pub fn new(
        lhs: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
        lhs_idx: usize,
        op: Box<dyn BinaryOperatorIndex + 'a>,
        op_args: &BinaryOperatorArguments,
        node_search_desc: Arc<NodeSearchDesc>,
        node_annos: &'a dyn AnnotationStorage<NodeID>,
        rhs_desc: Option<&ExecutionNodeDesc>,
    ) -> IndexJoin<'a> {
        let lhs_desc = lhs.get_desc().cloned();
        let lhs_peek = lhs.peekable();

        let processed_func = |est_type: EstimationType, out_lhs: usize, out_rhs: usize| {
            match est_type {
                EstimationType::Selectivity(op_sel) => {
                    // A index join processes each LHS and for each LHS the number of reachable nodes given by the operator.
                    // The selectivity of the operator itself an estimation how many nodes are filtered out by the cross product.
                    // We can use this number (without the edge annotation selectivity) to re-construct the number of reachable nodes.

                    // avgReachable = (sel * cross) / lhs
                    //              = (sel * lhs * rhs) / lhs
                    //              = sel * rhs
                    // processedInStep = lhs + (avgReachable * lhs)
                    //                 = lhs + (sel * rhs * lhs)

                    let result = (out_lhs as f64) + (op_sel * (out_rhs as f64) * (out_lhs as f64));

                    result.round() as usize
                }
                EstimationType::Min => out_lhs,
            }
        };

        IndexJoin {
            desc: ExecutionNodeDesc::join(
                op.as_binary_operator(),
                lhs_desc.as_ref(),
                rhs_desc,
                "indexjoin (parallel)",
                &format!("#{} {} #{}", op_args.left, &op, op_args.right),
                &processed_func,
            ),
            lhs: lhs_peek,
            lhs_idx,
            op: Arc::from(op),
            node_search_desc,
            node_annos,
            match_receiver: None,
            global_reflexivity: op_args.global_reflexivity,
        }
    }

    fn next_lhs_buffer(
        &mut self,
        tx: &Sender<Result<MatchGroup>>,
    ) -> Vec<(Result<MatchGroup>, Sender<Result<MatchGroup>>)> {
        let mut lhs_buffer = Vec::with_capacity(MAX_BUFFER_SIZE);
        while lhs_buffer.len() < MAX_BUFFER_SIZE {
            if let Some(lhs) = self.lhs.next() {
                lhs_buffer.push((lhs, tx.clone()));
            } else {
                break;
            }
        }
        lhs_buffer
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Result<MatchGroup>>> {
        let (tx, rx) = channel();
        let lhs_buffer = self.next_lhs_buffer(&tx);

        if lhs_buffer.is_empty() {
            return None;
        }

        let node_search_desc: Arc<NodeSearchDesc> = self.node_search_desc.clone();
        let op: Arc<dyn BinaryOperatorIndex> = self.op.clone();
        let lhs_idx = self.lhs_idx;
        let node_annos = self.node_annos;

        let op: &dyn BinaryOperatorIndex = op.as_ref();
        let global_reflexivity = self.global_reflexivity;

        // find all RHS in parallel
        lhs_buffer.into_par_iter().for_each(|(m_lhs, tx)| {
            match m_lhs {
                Ok(m_lhs) => {
                    match next_candidates(&m_lhs, op, lhs_idx, node_annos, &node_search_desc) {
                        Ok(rhs_candidate) => {
                            let mut rhs_candidate = rhs_candidate.into_iter().peekable();
                            while let Some(mut m_rhs) = rhs_candidate.next() {
                                // check if all filters are true
                                let mut include_match = true;
                                for f in &node_search_desc.cond {
                                    if !(f)(&m_rhs, node_annos) {
                                        include_match = false;
                                        break;
                                    }
                                }

                                if include_match {
                                    // replace the annotation with a constant value if needed
                                    if let Some(ref const_anno) = node_search_desc.const_output {
                                        m_rhs = (m_rhs.node, const_anno.clone()).into();
                                    }

                                    // check if lhs and rhs are equal and if this is allowed in this query
                                    if op.is_reflexive()
                                        || (global_reflexivity && m_rhs.different_to_all(&m_lhs)
                                            || (!global_reflexivity
                                                && m_rhs.different_to(&m_lhs[lhs_idx])))
                                    {
                                        // filters have been checked, return the result
                                        let mut result: MatchGroup = m_lhs.clone();
                                        let matched_node = m_rhs.node;
                                        result.push(m_rhs);
                                        if node_search_desc.const_output.is_some() {
                                            // only return the one unique constAnno for this node and no duplicates
                                            // skip all RHS candidates that have the same node ID
                                            #[allow(clippy::while_let_loop)]
                                            loop {
                                                if let Some(next_match) = rhs_candidate.peek() {
                                                    if next_match.node != matched_node {
                                                        break;
                                                    }
                                                } else {
                                                    break;
                                                }
                                                rhs_candidate.next();
                                            }
                                        }
                                        if tx.send(Ok(result)).is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if let Err(e) = tx.send(Err(e)) {
                                trace!("Could not send error in parallel index join: {}", e);
                            };
                        }
                    }
                }
                Err(e) => {
                    if let Err(e) = tx.send(Err(e)) {
                        trace!("Could not send error in parallel index join: {}", e);
                    }
                }
            };
        });
        Some(rx)
    }
}

fn next_candidates(
    m_lhs: &[Match],
    op: &dyn BinaryOperatorIndex,
    lhs_idx: usize,
    node_annos: &dyn AnnotationStorage<NodeID>,
    node_search_desc: &Arc<NodeSearchDesc>,
) -> Result<Vec<Match>> {
    let it_nodes = op
        .retrieve_matches(&m_lhs[lhs_idx])
        .map_ok(|m| m.node)
        .map(|n| {
            n.map_err(|e| {
                let e: Box<dyn Error + Send + Sync> = Box::new(e);
                e
            })
        })
        .fuse();
    let it_nodes: Box<
        dyn Iterator<Item = std::result::Result<NodeID, Box<dyn Error + Send + Sync>>>,
    > = Box::from(it_nodes);

    let result = node_annos.get_keys_for_iterator(
        node_search_desc.qname.0.as_deref(),
        node_search_desc.qname.1.as_deref(),
        it_nodes,
    )?;
    Ok(result)
}

impl<'a> ExecutionNode for IndexJoin<'a> {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = Result<MatchGroup>> {
        self
    }

    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        Some(&self.desc)
    }
}

impl<'a> Iterator for IndexJoin<'a> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        // lazily initialize
        if self.match_receiver.is_none() {
            self.match_receiver = if let Some(rhs) = self.next_match_receiver() {
                Some(rhs)
            } else {
                return None;
            };
        }

        loop {
            {
                let match_receiver = self.match_receiver.as_mut()?;
                if let Ok(result) = match_receiver.recv() {
                    return Some(result);
                }
            }

            // inner was completed once, get new candidates
            if let Some(rhs) = self.next_match_receiver() {
                self.match_receiver = Some(rhs);
            } else {
                // no more results to fetch
                return None;
            }
        }
    }
}
