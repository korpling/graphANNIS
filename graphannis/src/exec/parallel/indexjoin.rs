use super::super::{Desc, ExecutionNode, NodeSearchDesc};
use annostorage::AnnoStorage;
use operator::{EstimationType, Operator};
use rayon::prelude::*;
use std;
use std::iter::Peekable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use stringstorage::StringStorage;
use util;
use {AnnoKey, Annotation, Match, NodeID};

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    match_receiver: Option<Receiver<Vec<Match>>>,
    op: Arc<Operator>,
    lhs_idx: usize,
    node_search_desc: Arc<NodeSearchDesc>,
    node_annos: Arc<AnnoStorage<NodeID>>,
    strings: Arc<StringStorage>,
    desc: Desc,
}

impl<'a> IndexJoin<'a> {
    /// Create a new `IndexJoin`
    /// # Arguments
    ///
    /// * `lhs` - An iterator for a left-hand-side
    /// * `lhs_idx` - The index of the element in the LHS that should be used as a source
    /// * `op` - The operator that connects the LHS and RHS
    /// * `anno_qname` A pair of the annotation namespace and name (both optional) to define which annotations to fetch
    /// * `anno_cond` - A filter function to determine if a RHS candidate is included
    pub fn new(
        lhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        node_nr_lhs: usize,
        node_nr_rhs: usize,
        op: Box<Operator>,
        node_search_desc: Arc<NodeSearchDesc>,
        node_annos: Arc<AnnoStorage<NodeID>>,
        strings: Arc<StringStorage>,
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

                    return result.round() as usize;
                }
                EstimationType::MIN | EstimationType::MAX => {
                    return out_lhs;
                }
            }
        };

        return IndexJoin {
            desc: Desc::join(
                &op,
                lhs_desc.as_ref(),
                rhs_desc,
                "indexjoin",
                &format!("#{} {} #{}", node_nr_lhs, op, node_nr_rhs),
                &processed_func,
            ),
            lhs: lhs_peek,
            lhs_idx,
            op: Arc::from(op),
            node_search_desc,
            node_annos,
            strings,
            match_receiver: None,
        };
    }

    fn next_candidates(&mut self) -> Option<Vec<Match>> {
        if let Some(m_lhs) = self.lhs.peek().cloned() {
            let it_nodes = self.op.retrieve_matches(&m_lhs[self.lhs_idx]).fuse();

            let node_annos = self.node_annos.clone();
            if let Some(name) = self.node_search_desc.qname.1 {
                if let Some(ns) = self.node_search_desc.qname.0 {
                    // return the only possible annotation for each node
                    let mut matches: Vec<Match> = Vec::new();
                    for match_node in it_nodes {
                        let key = AnnoKey { ns: ns, name: name };
                        if let Some(val) = node_annos.get(&match_node.node, &key) {
                            matches.push(Match {
                                node: match_node.node,
                                anno: Annotation {
                                    key,
                                    val: val.clone(),
                                },
                            });
                        }
                    }
                    return Some(matches);
                } else {
                    let keys = self.node_annos.get_qnames(name);
                    // return all annotations with the correct name for each node
                    let mut matches: Vec<Match> = Vec::new();
                    for match_node in it_nodes {
                        for k in keys.clone() {
                            if let Some(val) = node_annos.get(&match_node.node, &k) {
                                matches.push(Match {
                                    node: match_node.node,
                                    anno: Annotation {
                                        key: k,
                                        val: val.clone(),
                                    },
                                })
                            }
                        }
                    }
                    return Some(matches);
                }
            } else {
                // return all annotations for each node
                let mut matches: Vec<Match> = Vec::new();
                for match_node in it_nodes {
                    let annos = node_annos.get_all(&match_node.node);
                    for a in annos {
                        matches.push(Match {
                            node: match_node.node,
                            anno: a,
                        });
                    }
                }
                return Some(matches);
            }
        }

        return None;
    }

    fn next_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        if let Some(m_lhs) = self.lhs.peek().cloned() {
            if let Some(rhs_candidate) = self.next_candidates() {
                let (tx, rx) = channel();

                // extend the vector with a seperate sender for each element
                let mut rhs_candidate_with_tx: Vec<(Match, Sender<Vec<Match>>)> =
                    Vec::with_capacity(rhs_candidate.len());
                for m_rhs in rhs_candidate.into_iter() {
                    rhs_candidate_with_tx.push((m_rhs, tx.clone()));
                }

                let node_search_desc: Arc<NodeSearchDesc> = self.node_search_desc.clone();
                let strings: Arc<StringStorage> = self.strings.clone();
                let op: Arc<Operator> = self.op.clone();
                let lhs_idx = self.lhs_idx;

                let op: &Operator = op.as_ref();
                // check all RHS candidates in parallel
                rhs_candidate_with_tx
                    .par_iter_mut()
                    .for_each(|(m_rhs, tx)| {
                        // check if all filters are true
                        let mut filter_result = true;
                        for f in node_search_desc.cond.iter() {
                            if !(f)(&m_rhs, strings.as_ref()) {
                                filter_result = false;
                                break;
                            }
                        }

                        if filter_result {
                            // replace the annotation with a constant value if needed
                            if let Some(ref const_anno) = node_search_desc.const_output {
                                m_rhs.anno = const_anno.clone();
                            }

                            // check if lhs and rhs are equal and if this is allowed in this query
                            if op.is_reflexive() || m_lhs[lhs_idx].node != m_rhs.node
                                || !util::check_annotation_key_equal(
                                    &m_lhs[lhs_idx].anno,
                                    &m_rhs.anno,
                                ) {
                                // filters have been checked, return the result
                                let mut result = m_lhs.clone();
                                let _matched_node = m_rhs.node;
                                result.push(m_rhs.clone());
                                if node_search_desc.const_output.is_some() {
                                    // only return the one unique constAnno for this node and no duplicates
                                    // TODO: skip all RHS candidates that have the same node ID
                                    unimplemented!()
                                    // loop {
                                    //     if let Some(next_match) = rhs_candidate.last() {
                                    //         if next_match.node != matched_node {
                                    //             break;
                                    //         }
                                    //     } else {
                                    //         break;
                                    //     }
                                    //     rhs_candidate.pop();
                                    // }
                                }
                                tx.send(result);
                            }
                        }
                    });
            
                return Some(rx);
            }
        }
        return None;
    }
}

impl<'a> ExecutionNode for IndexJoin<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
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
        if self.match_receiver.is_none() {
            self.match_receiver = if let Some(rhs) = self.next_receiver() {
                Some(rhs)
            } else {
                None
            };
        }

        if self.match_receiver.is_none() {
            return None;
        }

        loop {
            {
                let match_receiver: &mut Receiver<Vec<Match>> =
                    self.match_receiver.as_mut().unwrap();
                if let Ok(result) = match_receiver.recv() {
                    return Some(result);
                }

                // consume next outer
                if self.lhs.next().is_none() {
                    return None;
                }
            }

            // inner was completed once, get new candidates
            self.match_receiver = if let Some(rhs) = self.next_receiver() {
                Some(rhs)
            } else {
                None
            };
        }
    }
}
