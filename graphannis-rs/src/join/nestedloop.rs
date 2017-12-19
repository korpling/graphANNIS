use Match;
use operator::Operator;
use std::iter::Peekable;

pub fn new<'a>(
        lhs: Box<Iterator<Item = Vec<Match>>>,
        rhs: Box<Iterator<Item = Vec<Match>>>,
        lhs_idx : usize,
        rhs_idx : usize,
        op: &'a Operator,
    ) -> Box<Iterator<Item = Vec<Match>> + 'a> {
        // TODO: allow switching inner and outer
       let it =  NestedLoop {
            outer: lhs.peekable(),
            inner: rhs,
            op: op,
            outer_idx : lhs_idx,
            inner_idx : rhs_idx,
            inner_cache: Vec::new(),
            pos_inner_cache: None,
        };
        return Box::new(it);
    }

struct NestedLoop<'a> {
    outer: Peekable<Box<Iterator<Item = Vec<Match>>>>,
    inner: Box<Iterator<Item = Vec<Match>>>,
    op: &'a Operator,
    inner_idx : usize,
    outer_idx : usize,
    inner_cache: Vec<Vec<Match>>,
    pos_inner_cache: Option<usize>,
}


impl<'a> Iterator for NestedLoop<'a> {
    type Item = Vec<Match>;


    fn next(&mut self) -> Option<Vec<Match>> {
        loop {
            if let Some(m_outer) = self.outer.peek() {
                if  self.pos_inner_cache.is_some() {
                    let mut cache_pos = self.pos_inner_cache.unwrap();

                    while cache_pos < self.inner_cache.len()  {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);
                        if self.op.filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx]) {
                            let mut result = m_outer.clone();
                            result.append(&mut m_inner.clone());
                            return Some(result);
                        }
                    }

                } else {
                    while let Some(m_inner) = self.inner.next() {
                        self.inner_cache.push(m_inner.clone());

                        if self.op.filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx]) {
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
