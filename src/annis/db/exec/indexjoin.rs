use super::{Desc, ExecutionNode, NodeSearchDesc};
use crate::annis::db::query::conjunction::BinaryOperatorEntry;
use crate::annis::db::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::operator::{BinaryOperator, EstimationType};
use crate::annis::types::{AnnoKey, NodeID};
use std;
use std::iter::Peekable;
use std::sync::Arc;

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<dyn ExecutionNode<Item = Vec<Match>> + 'a>>,
    rhs_candidate: Option<std::iter::Peekable<Box<dyn Iterator<Item = Match>>>>,
    op: Box<dyn BinaryOperator>,
    lhs_idx: usize,
    node_search_desc: Arc<NodeSearchDesc>,
    node_annos: Arc<AnnotationStorage<NodeID>>,
    desc: Desc,
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
        lhs: Box<dyn ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        op_entry: BinaryOperatorEntry,
        node_search_desc: Arc<NodeSearchDesc>,
        node_annos: Arc<AnnotationStorage<NodeID>>,
        rhs_desc: Option<&Desc>,
    ) -> IndexJoin<'a> {
        let lhs_desc = lhs.get_desc().cloned();
        // TODO, we
        let lhs_peek = lhs.peekable();

        let processed_func = |est_type: EstimationType, out_lhs: usize, out_rhs: usize| {
            match est_type {
                EstimationType::SELECTIVITY(op_sel) => {
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
                EstimationType::MIN => out_lhs,
            }
        };

        IndexJoin {
            desc: Desc::join(
                op_entry.op.as_ref(),
                lhs_desc.as_ref(),
                rhs_desc,
                "indexjoin",
                &format!(
                    "#{} {} #{}",
                    op_entry.node_nr_left, op_entry.op, op_entry.node_nr_right
                ),
                &processed_func,
            ),
            lhs: lhs_peek,
            lhs_idx,
            op: op_entry.op,
            node_search_desc,
            node_annos,
            rhs_candidate: None,
            global_reflexivity: op_entry.global_reflexivity,
        }
    }

    fn next_candidates(&mut self) -> Option<Box<dyn Iterator<Item = Match>>> {
        if let Some(m_lhs) = self.lhs.peek().cloned() {
            let it_nodes = self.op.retrieve_matches(&m_lhs[self.lhs_idx]).fuse();

            let node_annos = self.node_annos.clone();
            if let Some(name) = self.node_search_desc.qname.1.clone() {
                if let Some(ns) = self.node_search_desc.qname.0.clone() {
                    // return the only possible annotation for each node
                    let key = AnnoKey {
                        ns: ns.clone(),
                        name: name.clone(),
                    };
                    return Some(Box::new(it_nodes.filter_map(move |match_node| {
                        if node_annos
                            .get_value_for_item(&match_node.node, &key)
                            .is_some()
                        {
                            Some(Match {
                                node: match_node.node,
                                anno_key: key.clone(),
                            })
                        } else {
                            // this annotation was not found for this node, remove it from iterator
                            None
                        }
                
                    })));
                } else {
                    let keys: Vec<AnnoKey> = self
                        .node_annos
                        .get_qnames(&name);
                    // return all annotations with the correct name for each node
                    return Some(Box::new(it_nodes.flat_map(move |match_node| {
                        let mut matches: Vec<Match> = Vec::new();
                        matches.reserve(keys.len());
                        for k in keys.iter() {
                            if node_annos
                                .get_value_for_item(&match_node.node, k)
                                .is_some()
                            {
                                matches.push(Match {
                                    node: match_node.node,
                                    anno_key: k.clone(),
                                })
                            }
                        }
                        matches.into_iter()
                    })));
                }
            } else {
                // return all annotations for each node
                return Some(Box::new(it_nodes.flat_map(move |match_node| {
                    let anno_keys = node_annos.get_all_keys_for_item(&match_node.node);
                    let mut matches: Vec<Match> = Vec::new();
                    matches.reserve(anno_keys.len());
                    for anno_key in anno_keys {
                        matches.push(Match {
                            node: match_node.node,
                            anno_key,
                        });
                    
                    }
                    matches.into_iter()
                })));
            }
        }

        None
    }
}

impl<'a> ExecutionNode for IndexJoin<'a> {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        Some(&self.desc)
    }
}

impl<'a> Iterator for IndexJoin<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        // lazily initialize the RHS candidates for the first LHS
        if self.rhs_candidate.is_none() {
            self.rhs_candidate = if let Some(rhs) = self.next_candidates() {
                Some(rhs.peekable())
            } else {
                return None;
            };
        }

        loop {
            if let Some(m_lhs) = self.lhs.peek() {
                let rhs_candidate = self.rhs_candidate.as_mut()?;
                while let Some(mut m_rhs) = rhs_candidate.next() {
                    // check if all filters are true
                    let mut filter_result = true;
                    for f in &self.node_search_desc.cond {
                        if !(f)(&m_rhs) {
                            filter_result = false;
                            break;
                        }
                    }

                    if filter_result {
                        // replace the annotation with a constant value if needed
                        if let Some(ref const_anno) = self.node_search_desc.const_output {
                            m_rhs.anno_key = const_anno.clone();
                        }

                        // check if lhs and rhs are equal and if this is allowed in this query
                        if self.op.is_reflexive()
                            || (self.global_reflexivity && m_rhs.different_to_all(&m_lhs)
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
                            return Some(result);
                        }
                    }
                }
            }

            // consume next outer
            self.lhs.next()?;

            // inner was completed once, get new candidates
            self.rhs_candidate = if let Some(rhs) = self.next_candidates() {
                Some(rhs.peekable())
            } else {
                None
            };
        }
    }
}
