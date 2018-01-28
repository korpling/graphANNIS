use Match;
use super::{ExecutionNode,Desc};
use operator::Operator;
use std::iter::Peekable;

pub struct NestedLoop<'a> {
    outer: Peekable<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    inner: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
    op: Box<Operator + 'a>,
    inner_idx: usize,
    outer_idx: usize,
    inner_cache: Vec<Vec<Match>>,
    pos_inner_cache: Option<usize>,

    desc: Desc,
}

impl<'a> NestedLoop<'a> {
    pub fn new(
        lhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        rhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
        op: Box<Operator + 'a>,
    ) -> NestedLoop<'a> {
        // TODO: allow switching inner and outer
        let it = NestedLoop {

            desc : Desc::join(lhs.get_desc(), rhs.get_desc()),

            outer: lhs.peekable(),
            inner: rhs,
            op: op,
            outer_idx: lhs_idx,
            inner_idx: rhs_idx,
            inner_cache: Vec::new(),
            pos_inner_cache: None,

        };
        return it;
    }
}


impl<'a> ExecutionNode for NestedLoop<'a> {

    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        Some(&self.desc)
    }
}



impl<'a> Iterator for NestedLoop<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        loop {
            if let Some(m_outer) = self.outer.peek() {
                if self.pos_inner_cache.is_some() {
                    let mut cache_pos = self.pos_inner_cache.unwrap();

                    while cache_pos < self.inner_cache.len() {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);
                        if self.op
                            .filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx])
                        {
                            let mut result = m_outer.clone();
                            result.append(&mut m_inner.clone());
                            return Some(result);
                        }
                    }
                } else {
                    while let Some(m_inner) = self.inner.next() {
                        self.inner_cache.push(m_inner.clone());

                        if self.op
                            .filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx])
                        {
                            let mut result = m_outer.clone();
                            result.append(&mut m_inner.clone());
                            return Some(result);
                        }
                    }
                }
                // inner was completed once, use cache from now, or reset to first item once completed
                 self.pos_inner_cache = Some(0)
            }

            // consume next outer
            if self.outer.next().is_none() {
                return None;
            }
        }
    }
}
