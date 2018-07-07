use super::super::{Desc, ExecutionNode, NodeSearchDesc};
use annostorage::AnnoStorage;
use operator::{EstimationType, Operator};
use rayon::prelude::*;
use std::iter::Peekable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use stringstorage::StringStorage;
use util;
use {AnnoKey, Annotation, Match, NodeID};

const MAX_BUFFER_SIZE : usize = 512;

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

    fn next_lhs_buffer(&mut self, tx : Sender<Vec<Match>>) -> Vec<(Vec<Match>, Sender<Vec<Match>>)> {
        let mut lhs_buffer : Vec<(Vec<Match>, Sender<Vec<Match>>)> = Vec::with_capacity(MAX_BUFFER_SIZE);
        while lhs_buffer.len() < MAX_BUFFER_SIZE {
            if let Some(lhs) = self.lhs.next() {
                lhs_buffer.push((lhs, tx.clone()));
            } else {
                break;
            }
        }
        return lhs_buffer;
    }

    

    fn next_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        
        let (tx, rx) = channel();
        let mut lhs_buffer = self.next_lhs_buffer(tx);

        let node_search_desc: Arc<NodeSearchDesc> = self.node_search_desc.clone();
        let strings: Arc<StringStorage> = self.strings.clone();
        let op: Arc<Operator> = self.op.clone();
        let lhs_idx = self.lhs_idx;
        let node_annos = self.node_annos.clone();

        let op: &Operator = op.as_ref();

        // find all RHS in parallel
        lhs_buffer.par_iter_mut().for_each(|(m_lhs, tx)| {
            if let Some(rhs_candidate) = next_candidates(m_lhs, op, lhs_idx, node_annos.clone(), node_search_desc.clone()) {
                let mut rhs_candidate = rhs_candidate.into_iter().peekable();
                while let Some(mut m_rhs) = rhs_candidate.next() {
                    // check if all filters are true
                    let mut filter_result = true;
                    for f in node_search_desc.cond.iter() {
                        if !(f)(&m_rhs, &strings) {
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
                            || !util::check_annotation_key_equal(&m_lhs[lhs_idx].anno, &m_rhs.anno)
                        {
                            // filters have been checked, return the result
                            let mut result = m_lhs.clone();
                            let matched_node = m_rhs.node;
                            result.push(m_rhs);
                            if node_search_desc.const_output.is_some() {
                                // only return the one unique constAnno for this node and no duplicates
                                // skip all RHS candidates that have the same node ID
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
                            // TODO: handle error
                            tx.send(result);
                        
                        }
                    }
                }
            }
        });
        return Some(rx);
    }
}

fn next_candidates(m_lhs : &Vec<Match>, op : &Operator, lhs_idx : usize, node_annos: Arc<AnnoStorage<NodeID>>, node_search_desc: Arc<NodeSearchDesc>) -> Option<Vec<Match>> {
    let it_nodes = op.retrieve_matches(&m_lhs[lhs_idx]).fuse();

    if let Some(name) = node_search_desc.qname.1 {
        if let Some(ns) = node_search_desc.qname.0 {
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
            let keys = node_annos.get_qnames(name);
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
