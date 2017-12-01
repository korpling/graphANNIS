use Match;
use operator::Operator;
use std::iter::Peekable;
use std::rc::Rc;

pub struct NestedLoop {
    outer: Peekable<Box<Iterator<Item = Match>>>,
    inner: Box<Iterator<Item = Match>>,
    op: Rc<Operator>,
    inner_cache: Vec<Match>,
    pos_inner_cache: Option<usize>,
}

impl NestedLoop {
    pub fn new(
        lhs: Box<Iterator<Item = Match>>,
        rhs: Box<Iterator<Item = Match>>,
        op: Rc<Operator>,
    ) -> NestedLoop {
        // TODO: allow switching inner and outer
        NestedLoop {
            outer: lhs.peekable(),
            inner: rhs,
            op: op,
            inner_cache: Vec::new(),
            pos_inner_cache: None,
        }
    }
}

impl Iterator for NestedLoop {
    type Item = (Match, Match);


    fn next(&mut self) -> Option<(Match, Match)> {
        loop {
            if let Some(m_outer) = self.outer.peek() {
                if  self.pos_inner_cache.is_some() {
                    let mut cache_pos = self.pos_inner_cache.unwrap();

                    while cache_pos < self.inner_cache.len()  {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);
                        if self.op.filter_match(m_outer, &m_inner) {
                            return Some((m_outer.clone(), m_inner.clone()));
                        }
                    }

                } else {
                    while let Some(m_inner) = self.inner.next() {
                        self.inner_cache.push(m_inner.clone());

                        if self.op.filter_match(m_outer, &m_inner) {
                            return Some((m_outer.clone(), m_inner));
                        }
                    }
                }
            }

            // consume next outer
            if self.outer.next().is_none() {
                return None;
            }
        }
    }
}
