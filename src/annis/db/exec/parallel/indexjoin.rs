use super::super::{Desc, ExecutionNode, NodeSearchDesc};
use annis::db::annostorage::AnnoStorage;
use annis::db::Match;
use annis::operator::{EstimationType, Operator};
use annis::types::{AnnoKey, NodeID};
use rayon::prelude::*;
use std::iter::Peekable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 512;

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
                EstimationType::MIN => {
                    out_lhs
                }
            }
        };

        IndexJoin {
            desc: Desc::join(
                op.as_ref(),
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
            match_receiver: None,
        }
    }

    fn next_lhs_buffer(&mut self, tx: &Sender<Vec<Match>>) -> Vec<(Vec<Match>, Sender<Vec<Match>>)> {
        let mut lhs_buffer: Vec<(Vec<Match>, Sender<Vec<Match>>)> =
            Vec::with_capacity(MAX_BUFFER_SIZE);
        while lhs_buffer.len() < MAX_BUFFER_SIZE {
            if let Some(lhs) = self.lhs.next() {
                lhs_buffer.push((lhs, tx.clone()));
            } else {
                break;
            }
        }
        lhs_buffer
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        let (tx, rx) = channel();
        let mut lhs_buffer = self.next_lhs_buffer(&tx);

        if lhs_buffer.is_empty() {
            return None;
        }

        let node_search_desc: Arc<NodeSearchDesc> = self.node_search_desc.clone();
        let op: Arc<Operator> = self.op.clone();
        let lhs_idx = self.lhs_idx;
        let node_annos = self.node_annos.clone();

        let op: &Operator = op.as_ref();

        // find all RHS in parallel
        lhs_buffer.par_iter_mut().for_each(|(m_lhs, tx)| {
            if let Some(rhs_candidate) = next_candidates(m_lhs, op, lhs_idx, &node_annos, &node_search_desc) {
                let mut rhs_candidate = rhs_candidate.into_iter().peekable();
                while let Some(mut m_rhs) = rhs_candidate.next() {
                    // check if all filters are true
                    let mut filter_result = true;
                    for f in &node_search_desc.cond {
                        if !(f)(&m_rhs) {
                            filter_result = false;
                            break;
                        }
                    }

                    if filter_result {

                        // replace the annotation with a constant value if needed
                        if let Some(ref const_anno) = node_search_desc.const_output {
                            m_rhs.anno_key = *const_anno;
                        }

                        // check if lhs and rhs are equal and if this is allowed in this query
                        if op.is_reflexive() || m_lhs[lhs_idx].node != m_rhs.node
                            || m_lhs[lhs_idx].anno_key !=  m_rhs.anno_key
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
                            if let Err(_) = tx.send(result) {
                                return;
                            }
                        }
                    }
                }
            }
        });
        Some(rx)
    }
}

fn next_candidates(
    m_lhs: &[Match],
    op: &Operator,
    lhs_idx: usize,
    node_annos: &Arc<AnnoStorage<NodeID>>,
    node_search_desc: &Arc<NodeSearchDesc>,
) -> Option<Vec<Match>> {
    let it_nodes = op.retrieve_matches(&m_lhs[lhs_idx]).fuse();

    if let Some(ref name) = node_search_desc.qname.1 {
        if let Some(ref ns) = node_search_desc.qname.0 {
            // return the only possible annotation for each node
            let mut matches: Vec<Match> = Vec::new();
            let key = Arc::from(AnnoKey {
                ns: ns.clone(),
                name: name.clone(),
            });
            let key_id = node_annos.get_key_id(&key);

            for match_node in it_nodes {
                if let Some(key_id) = key_id {
                    if node_annos.get_value_for_item_by_id(&match_node.node, key_id).is_some() {
                        matches.push(Match {
                            node: match_node.node,
                            anno_key: key_id,
                        });
                    }
                }
            }
            return Some(matches);
        } else {
            let keys: Vec<usize> = node_annos
                .get_qnames(&name)
                .into_iter()
                .filter_map(|k| node_annos.get_key_id(&k))
                .collect();
            // return all annotations with the correct name for each node
            let mut matches: Vec<Match> = Vec::new();
            for match_node in it_nodes {
                for key_id in keys.clone() {
                    if node_annos.get_value_for_item_by_id(&match_node.node, key_id).is_some() {
                        matches.push(Match {
                            node: match_node.node,
                            anno_key: key_id,
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
            let all_keys = node_annos.get_all_keys_for_item(&match_node.node);
            for anno_key in all_keys {
                if let Some(key_id) = node_annos.get_key_id(&anno_key) {
                    matches.push(Match {
                        node: match_node.node,
                        anno_key: key_id,
                    });
                }
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
        // lazily initialize
        if self.match_receiver.is_none() {
            self.match_receiver = if let Some(rhs) = self.next_match_receiver() {
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
