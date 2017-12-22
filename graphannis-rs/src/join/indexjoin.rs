use {Annotation, Match};
use operator::Operator;
use plan::{ExecutionNode,Desc};
use std;
use std::iter::Peekable;

/// A join that takes any iterator as left-hand-side (LHS) and an annotation condition as right-hand-side (RHS).
/// It then retrieves all matches as defined by the operator for each LHS element and checks
/// if the annotation condition is true.
pub struct IndexJoin {
    lhs: Peekable<Box<ExecutionNode<Item = Vec<Match>>>>,
    rhs_candidate: std::vec::IntoIter<Match>,
    op: Box<Operator>,
    lhs_idx: usize,
    anno_cond: Box<Fn(&Annotation) -> bool>,

    lhs_desc: Option<Desc>,
    rhs_desc: Option<Desc>,
}

impl IndexJoin {

    /// Create a new `IndexJoin`
    /// # Arguments
    /// 
    /// * `lhs` - An iterator for a left-hand-side
    /// * `lhs_idx` - The index of the element in the LHS that should be used as a source
    /// * `op` - The operator that connects the LHS and RHS
    /// * `anno_cond` - A filter function to determine if a RHS candidate is included
    pub fn new(
        lhs: Box<ExecutionNode<Item = Vec<Match>>>,
        lhs_idx: usize,
        op: Box<Operator>,
        anno_cond: Box<Fn(&Annotation) -> bool>,
    ) -> IndexJoin {
        let lhs_desc = lhs.get_desc().cloned();
        let mut lhs_peek = lhs.peekable();
        let initial_candidates: Vec<Match> = if let Some(m_lhs) = lhs_peek.peek() {
            op.retrieve_matches(&m_lhs[lhs_idx.clone()]).collect()
        } else {
            vec![]
        };
        return IndexJoin {
            lhs_desc,
            rhs_desc: None,
            lhs: lhs_peek,
            lhs_idx,
            op,
            anno_cond,
            rhs_candidate: initial_candidates.into_iter(),
        };
    }
}

impl ExecutionNode for IndexJoin {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_lhs_desc(&self) -> Option<&Desc> {
        self.lhs_desc.as_ref()
    }

    fn get_rhs_desc(&self) -> Option<&Desc> {
        self.rhs_desc.as_ref()
    }
}


impl Iterator for IndexJoin {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        loop {
            if let Some(m_lhs) = self.lhs.peek() {
                while let Some(m_rhs) = self.rhs_candidate.next() {
                    // filter by annotation
                    if (self.anno_cond)(&m_rhs.anno) {
                        let mut result = m_lhs.clone();
                        result.push(m_rhs.clone());
                        return Some(result);
                    }
                }
                // inner was completed once, get new candidates
                let candidates: Vec<Match> =
                    self.op.retrieve_matches(&m_lhs[self.lhs_idx]).collect();
                self.rhs_candidate = candidates.into_iter();
            }

            // consume next outer
            if self.lhs.next().is_none() {
                return None;
            }
        }
    }
}
