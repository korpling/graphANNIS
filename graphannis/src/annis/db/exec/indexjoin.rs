use super::{ExecutionNode, ExecutionNodeDesc, NodeSearchDesc};
use crate::annis::db::aql::conjunction::BinaryOperatorArguments;
use crate::annis::operator::BinaryOperatorIndex;
use crate::errors::Result;
use crate::try_as_option;
use crate::{annis::operator::EstimationType, graph::Match};
use graphannis_core::annostorage::NodeAnnotationStorage;
use graphannis_core::{annostorage::MatchGroup, types::NodeID};
use std::boxed::Box;
use std::error::Error;
use std::iter::Peekable;
use std::sync::Arc;

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>>,
    rhs_candidate: Option<Peekable<std::vec::IntoIter<Match>>>,
    op: Box<dyn BinaryOperatorIndex + 'a>,
    lhs_idx: usize,
    node_search_desc: Arc<NodeSearchDesc>,
    node_annos: &'a dyn NodeAnnotationStorage,
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
        node_annos: &'a dyn NodeAnnotationStorage,
        rhs_desc: Option<&ExecutionNodeDesc>,
    ) -> Result<IndexJoin<'a>> {
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

        let join = IndexJoin {
            desc: ExecutionNodeDesc::join(
                op.as_ref(),
                lhs_desc.as_ref(),
                rhs_desc,
                "indexjoin",
                &format!("#{} {} #{}", op_args.left, op, op_args.right),
                &processed_func,
            )?,
            lhs: lhs_peek,
            lhs_idx,
            op,
            node_search_desc,
            node_annos,
            rhs_candidate: None,
            global_reflexivity: op_args.global_reflexivity,
        };
        Ok(join)
    }

    fn next_candidates(&mut self) -> Result<Option<Vec<Match>>> {
        if let Some(Ok(m_lhs)) = self.lhs.peek() {
            let it_nodes = self
                .op
                .retrieve_matches(&m_lhs[self.lhs_idx])
                .fuse()
                .map(|m| {
                    m.map_err(|e| {
                        let e: Box<dyn Error + Send + Sync> = Box::new(e);
                        e
                    })
                    .map(|m| m.node)
                });
            let it_nodes: Box<
                dyn Iterator<Item = std::result::Result<NodeID, Box<dyn Error + Send + Sync>>>,
            > = Box::from(it_nodes);

            let result = self.node_annos.get_keys_for_iterator(
                self.node_search_desc.qname.0.as_deref(),
                self.node_search_desc.qname.1.as_deref(),
                it_nodes,
            )?;
            return Ok(Some(result));
        }
        Ok(None)
    }
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
        // lazily initialize the RHS candidates for the first LHS
        if self.rhs_candidate.is_none() {
            let rhs_candidates = try_as_option!(self.next_candidates());
            self.rhs_candidate = rhs_candidates.map(|c| c.into_iter().peekable());
        }

        loop {
            if let Some(Ok(m_lhs)) = self.lhs.peek() {
                let rhs_candidate = self.rhs_candidate.as_mut()?;
                while let Some(mut m_rhs) = rhs_candidate.next() {
                    // check if all filters are true
                    let mut filter_result = true;
                    for f in &self.node_search_desc.cond {
                        let single_filter_result = try_as_option!((f)(&m_rhs, self.node_annos));
                        if !single_filter_result {
                            filter_result = false;
                            break;
                        }
                    }

                    if filter_result {
                        // replace the annotation with a constant value if needed
                        if let Some(ref const_anno) = self.node_search_desc.const_output {
                            m_rhs = (m_rhs.node, const_anno.clone()).into();
                        }

                        // check if lhs and rhs are equal and if this is allowed in this query
                        if self.op.is_reflexive()
                            || (self.global_reflexivity && m_rhs.different_to_all(m_lhs)
                                || (!self.global_reflexivity
                                    && m_rhs.different_to(&m_lhs[self.lhs_idx])))
                        {
                            // filters have been checked, return the result
                            let mut result = m_lhs.clone();
                            let matched_node = m_rhs.node;
                            result.push(m_rhs);
                            if self.node_search_desc.const_output.is_some() {
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
                            return Some(Ok(result));
                        }
                    }
                }
            }

            // consume next outer and return if there is an error
            if let Err(e) = self.lhs.next()? {
                return Some(Err(e));
            }

            // inner was completed once, get new candidates
            let rhs_candidates = try_as_option!(self.next_candidates());
            self.rhs_candidate = rhs_candidates.map(|c| c.into_iter().peekable());
        }
    }
}
