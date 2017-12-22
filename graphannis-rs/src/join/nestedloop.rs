use Match;
use plan::{ExecutionNode,Desc};
use operator::Operator;
use std::iter::Peekable;

pub struct NestedLoop {
    outer: Peekable<Box<ExecutionNode<Item = Vec<Match>>>>,
    inner: Box<ExecutionNode<Item = Vec<Match>>>,
    op: Box<Operator>,
    inner_idx: usize,
    outer_idx: usize,
    inner_cache: Vec<Vec<Match>>,
    pos_inner_cache: Option<usize>,

    lhs_desc: Option<Desc>,
    rhs_desc: Option<Desc>,
}

impl NestedLoop {
    pub fn new(
        lhs: Box<ExecutionNode<Item = Vec<Match>>>,
        rhs: Box<ExecutionNode<Item = Vec<Match>>>,
        lhs_idx: usize,
        rhs_idx: usize,
        op: Box<Operator>,
    ) -> NestedLoop {
        // TODO: allow switching inner and outer
        let it = NestedLoop {
            lhs_desc: lhs.get_desc().cloned(),
            rhs_desc: rhs.get_desc().cloned(),
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


impl ExecutionNode for NestedLoop {

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



impl Iterator for NestedLoop {
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
                    // inner was completed once, use cache from now
                    self.pos_inner_cache = Some(0);
                }
            }

            // consume next outer
            if self.outer.next().is_none() {
                return None;
            }
        }
    }
}
