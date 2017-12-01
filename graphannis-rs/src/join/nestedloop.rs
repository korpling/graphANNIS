use {Match};
use operator::Operator;
use std::collections::LinkedList;
use std::iter::Peekable;
use std::rc::Rc;

pub struct NestedLoop<'a> {
    outer : Peekable<Box<Iterator<Item = Match>>>,
    inner : Box<Iterator<Item = Match>>,
    op : Rc<Operator>,
    inner_cache : LinkedList<Match>,
    it_inner_cache : Option<Box<Iterator<Item = &'a Match> + 'a>>,
}

impl<'a> NestedLoop<'a> {
    pub fn new(lhs : Box<Iterator<Item = Match>>, rhs : Box<Iterator<Item = Match>>, op : Rc<Operator>) -> NestedLoop<'a> {
        // TODO: allow switching inner and outer
        NestedLoop {
            outer : lhs.peekable(),
            inner : rhs,
            op : op,
            inner_cache : LinkedList::new(),
            it_inner_cache : None,
        }
    }
}

impl<'a> Iterator for NestedLoop<'a> {
    type Item = (Match,Match);


    fn next(&mut self) -> Option<(Match,Match)> {

        loop {
            if let Some(m_outer) = self.outer.peek() {
                while let Some(m_inner) = self.inner.next() {
                    if self.it_inner_cache.is_none() {
                        self.inner_cache.push_back(m_inner.clone());
                    }

                    if self.op.filter_match(m_outer, &m_inner) {
                        return Some((m_outer.clone(), m_inner));
                    }
                }

                self.it_inner_cache = Some(Box::new(self.inner_cache.iter()));
            }

            // consume next outer
            if self.outer.next().is_none() {
                return None;
            }
        }
    }
}