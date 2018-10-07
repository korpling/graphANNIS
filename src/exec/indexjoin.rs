use annostorage::AnnoStorage;
use std::sync::Arc;
use {AnnoKey, Annotation, Match, NodeID};
use operator::{EstimationType, Operator};
use super::{Desc, ExecutionNode, NodeSearchDesc};
use std;
use std::iter::Peekable;


/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    rhs_candidate: Option<std::iter::Peekable<Box<Iterator<Item=Match>>>>,
    op: Box<Operator>,
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
            op,
            node_search_desc,
            node_annos,
            rhs_candidate: None,
        };
    }

    fn next_candidates(&mut self) -> Option<Box<Iterator<Item=Match>>> {
        if let Some(m_lhs) = self.lhs.peek().cloned() {
            let it_nodes = self.op.retrieve_matches(&m_lhs[self.lhs_idx]).fuse();

            let node_annos = self.node_annos.clone();
            if let Some(name) = self.node_search_desc.qname.1.clone() {
                if let Some(ns) = self.node_search_desc.qname.0.clone() {
                    // return the only possible annotation for each node
                    let key = Arc::from(AnnoKey { ns: ns.clone(), name: name.clone() });
                    let key_id = self.node_annos.get_key_id(key.as_ref());
                    return Some(Box::new(it_nodes
                        .filter_map(move |match_node| {
                            if let Some(key_id) = key_id {
                                if let Some(val) = node_annos.get_by_id(&match_node.node, key_id) {
                                    Some(Match {
                                        node: match_node.node,
                                        anno: Annotation {
                                            key: key.clone(),
                                            val: val.clone(),
                                        },
                                    })
                                } else {
                                    // this annotation was not found for this node, remove it from iterator
                                    None
                                }
                            } else {
                                None
                            }
                        }))
                    );
                } else {
                    let keys: Vec<(AnnoKey, usize)> = self.node_annos
                    .get_qnames(&name)
                    .into_iter()
                    .filter_map(|k| self.node_annos.get_key_id(&k).and_then(|id| Some((k,id))))
                    .collect();
                    // return all annotations with the correct name for each node
                    return Some(Box::new(it_nodes
                        .flat_map(move |match_node| {
                            let mut matches: Vec<Match> = Vec::new();
                            matches.reserve(keys.len());
                            for (key, key_id) in keys.clone().into_iter() {
                                if let Some(val) = node_annos.get_by_id(&match_node.node, key_id) {
                                    matches.push(Match {
                                        node: match_node.node,
                                        anno: Annotation {
                                            key: Arc::from(key),
                                            val: val.clone(),
                                        },
                                    })
                                }
                            }
                            matches.into_iter()
                        }))
                    );
                }
            } else {
                // return all annotations for each node
                return Some(Box::new(it_nodes
                    .flat_map(move |match_node| {
                        let annos = node_annos.get_all(&match_node.node);
                        let mut matches: Vec<Match> = Vec::new();
                        matches.reserve(annos.len());
                        for a in annos {
                            matches.push(Match {
                                node: match_node.node,
                                anno: a,
                            });
                        }
                        matches.into_iter()
                    }))
                );
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
        if self.rhs_candidate.is_none() {
            self.rhs_candidate = if let Some(rhs) = self.next_candidates() {Some(rhs.into_iter().peekable())} else {None};
        }

        if self.rhs_candidate.is_none() {
            return None;
        }

        loop {
            if let Some(m_lhs) = self.lhs.peek() {
                let rhs_candidate = self.rhs_candidate.as_mut().unwrap();
                while let Some(mut m_rhs) = rhs_candidate.next() {
                    // check if all filters are true
                    let mut filter_result = true;
                    for f in self.node_search_desc.cond.iter() {
                        if !(f)(&m_rhs) {
                            filter_result = false;
                            break;
                        }
                    }

                    if filter_result {

                        // replace the annotation with a constant value if needed
                        if let Some(ref const_anno) = self.node_search_desc.const_output {
                            m_rhs.anno = const_anno.clone();
                        }

                        // check if lhs and rhs are equal and if this is allowed in this query
                        if self.op.is_reflexive() || m_lhs[self.lhs_idx].node != m_rhs.node
                            || m_lhs[self.lhs_idx].anno.key != m_rhs.anno.key
                        {
                            // filters have been checked, return the result
                            let mut result = m_lhs.clone();
                            let matched_node = m_rhs.node;
                            result.push(m_rhs);
                            if self.node_search_desc.const_output.is_some() {
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
                            return Some(result);
                        
                        }
                    }
                }
            }

            // consume next outer
            if self.lhs.next().is_none() {
                return None;
            }

            // inner was completed once, get new candidates
            self.rhs_candidate = if let Some(rhs) = self.next_candidates() {Some(rhs.into_iter().peekable())} else {None};
        }
    }
}
