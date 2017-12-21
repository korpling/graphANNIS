use {Annotation, Match};
use operator::Operator;
use plan::ExecutionNode;
use std;
use std::iter::Peekable;

pub struct IndexJoin {
    lhs: Peekable<Box<Iterator<Item = Vec<Match>>>>,
    rhs_candidate: std::vec::IntoIter<Match>,
    op: Box<Operator>,
    lhs_idx: usize,
    anno_cond: Box<Fn(&Annotation) -> bool>,
}

impl IndexJoin {
    pub fn new(
        lhs: Box<Iterator<Item = Vec<Match>>>,
        lhs_idx: usize,
        op: Box<Operator>,
        anno_cond: Box<Fn(&Annotation) -> bool>,
    ) -> IndexJoin {
        let mut lhs_peek = lhs.peekable();
        let initial_candidates: Vec<Match> = if let Some(m_lhs) = lhs_peek.peek() {
            op.retrieve_matches(&m_lhs[lhs_idx.clone()]).collect()
        } else {
            vec![]
        };
        return IndexJoin {
            lhs: lhs_peek,
            lhs_idx,
            op,
            anno_cond,
            rhs_candidate: initial_candidates.into_iter(),
        };
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

impl ExecutionNode for IndexJoin {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }
}
