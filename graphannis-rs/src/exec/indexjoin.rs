use {Annotation, Match, NodeID, StringID, AnnoKey};
use operator::Operator;
use annostorage::AnnoStorage;
use super::{ExecutionNode,Desc, NodeSearchDesc};
use std;
use std::iter::Peekable;

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin<'a> {
    lhs: Peekable<Box<ExecutionNode<Item = Vec<Match>>+'a>>,
    rhs_candidate: std::vec::IntoIter<Match>,
    op: Box<Operator + 'a>,
    lhs_idx: usize,
    node_search_desc: NodeSearchDesc<'a>,
    node_annos : &'a AnnoStorage<NodeID>,
    desc: Desc,
}

fn next_candidates(op : &Operator, m_lhs : &Vec<Match>, lhs_idx : usize, 
    anno_qname: &(Option<StringID>, Option<StringID>),
    node_annos : &AnnoStorage<NodeID>) -> Vec<Match> {

    let it_nodes = op.retrieve_matches(&m_lhs[lhs_idx]);
    
    if let Some(name) = anno_qname.1 {
        if let Some(ns) = anno_qname.0 {
            // return the only possible annotation for each node
            return it_nodes
                .filter_map(|match_node| {
                    let key = AnnoKey {ns: ns, name: name};
                    if let Some(val) = node_annos.get(&match_node.node, &key) {
                        Some(Match {node: match_node.node, anno: Annotation {
                            key,
                            val: val.clone(),
                        }})
                    } else {
                        // this annotation was not found for this node, remove it from iterator
                        None
                    }
                })
                .collect();
            
        } else {
            let keys = node_annos.get_qnames(name);
            // return all annotations with the correct name for each node
            return it_nodes
                .flat_map(|match_node| {
                    let mut matches : Vec<Match> = Vec::new();
                    matches.reserve(keys.len());
                    for k in keys.clone() {
                        if let Some(val) = node_annos.get(&match_node.node, &k) {
                            matches.push(Match{node: match_node.node, anno: Annotation {
                                key: k,
                                val: val.clone()
                            }})
                        }
                    }
                    matches.into_iter()
                })
                .collect();
        }
    } else {
        // return all annotations for each node
        return it_nodes
            .flat_map(|match_node| {
                let annos = node_annos.get_all(&match_node.node);
                let mut matches : Vec<Match> = Vec::new();
                matches.reserve(annos.len());
                for a in annos {
                    matches.push(Match{node: match_node.node, anno: a});
                }
                matches.into_iter()
            })
            .collect();
    }
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
        op: Box<Operator + 'a>,
        node_search_desc: NodeSearchDesc<'a>,
        node_annos : &'a AnnoStorage<NodeID>,
        rhs_desc: Option<&Desc>,
    ) -> IndexJoin<'a> {
        let lhs_desc = lhs.get_desc().cloned();
        // TODO, we 
        let mut lhs_peek = lhs.peekable();
        let initial_candidates: Vec<Match> = if let Some(m_lhs) = lhs_peek.peek() {
            next_candidates(op.as_ref(), &m_lhs, lhs_idx.clone(), &node_search_desc.qname, node_annos)
        } else {
            vec![]
        };
        return IndexJoin {
            desc: Desc::join(lhs_desc.as_ref(), rhs_desc),
            lhs: lhs_peek,
            lhs_idx,
            op,
            node_search_desc,
            node_annos,
            rhs_candidate: initial_candidates.into_iter(),
        };
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
        loop {
            if let Some(m_lhs) = self.lhs.peek() {
                while let Some(m_rhs) = self.rhs_candidate.next() {
                    // check if the filter is true
                    if (self.node_search_desc.cond)(m_rhs.anno.clone()) {
                        let mut result = m_lhs.clone();
                        result.push(m_rhs);
                        return Some(result);
                    }                    
                }
            }

            // consume next outer
            if self.lhs.next().is_none() {
                return None;
            }

            // inner was completed once, get new candidates
            if let Some(m_lhs) = self.lhs.peek() {
                let candidates: Vec<Match> = next_candidates(self.op.as_ref(), 
                    &m_lhs, self.lhs_idx.clone(), &self.node_search_desc.qname, self.node_annos);
                self.rhs_candidate = candidates.into_iter();
            }
        }
    }
}
