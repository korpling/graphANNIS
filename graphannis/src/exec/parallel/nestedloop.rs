use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use Match;
use util;
use super::super::{Desc, ExecutionNode};
use operator::Operator;
use std::iter::Peekable;
use rayon::prelude::*;

const MAX_BUFFER_SIZE : usize = 512;

pub struct NestedLoop<'a> {
    outer: Peekable<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    inner: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
    op: Arc<Operator>,
    inner_idx: usize,
    outer_idx: usize,

    match_receiver: Option<Receiver<Vec<Match>>>,
    inner_cache: Vec<Vec<Match>>,
    pos_inner_cache: Option<usize>,

    left_is_outer : bool,
    desc: Desc,
}

impl<'a> NestedLoop<'a> {
    pub fn new(
        lhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        rhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
        node_nr_lhs: usize,
        node_nr_rhs: usize,
        op: Box<Operator>,
    ) -> NestedLoop<'a> {

        let mut left_is_outer = true;
        if let (Some(ref desc_lhs), Some(ref desc_rhs)) = (lhs.get_desc(), rhs.get_desc()) {
            if let (&Some(ref cost_lhs), &Some(ref cost_rhs)) = (&desc_lhs.cost, &desc_rhs.cost) {

                if cost_lhs.output > cost_rhs.output {
                    left_is_outer = false;
                }
            }
        }
        
        let processed_func = |_, out_lhs: usize, out_rhs: usize| {
            if out_lhs <= out_rhs {
                // we use LHS as outer
                return out_lhs + (out_lhs * out_rhs);
            } else {
                // we use RHS as outer
                return out_rhs + (out_rhs * out_lhs);
            }
        };

        let it = if left_is_outer {
            NestedLoop {
                desc: Desc::join(
                    &op,
                    lhs.get_desc(),
                    rhs.get_desc(),
                    "nestedloop L-R",
                    &format!("#{} {} #{}", node_nr_lhs, op, node_nr_rhs),
                    &processed_func,
                ),

                outer: lhs.peekable(),
                inner: rhs,
                op: Arc::from(op),
                outer_idx: lhs_idx,
                inner_idx: rhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
            }
        } else {
            NestedLoop {
                desc: Desc::join(
                    &op,
                    rhs.get_desc(),
                    lhs.get_desc(),
                    "nestedloop R-L",
                    &format!("#{} {} #{}", node_nr_lhs, op, node_nr_rhs),
                    &processed_func,
                ),

                outer: rhs.peekable(),
                inner: lhs,
                op: Arc::from(op),
                outer_idx: rhs_idx,
                inner_idx: lhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
            }
        };

        return it;
    }

    fn next_match_buffer(&mut self, tx : Sender<Vec<Match>>) -> Vec<(Vec<Match>, Vec<Match>, Sender<Vec<Match>>)> {
        let mut match_candidate_buffer : Vec<(Vec<Match>, Vec<Match>, Sender<Vec<Match>>)> = Vec::with_capacity(MAX_BUFFER_SIZE);
        while match_candidate_buffer.len() < MAX_BUFFER_SIZE {
           if let Some(m_outer) = self.outer.peek() {
                if self.pos_inner_cache.is_some() {
                    let mut cache_pos = self.pos_inner_cache.unwrap();

                    while cache_pos < self.inner_cache.len() {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);

                        match_candidate_buffer.push((m_outer.clone(), m_inner.clone(), tx.clone()));

                        if match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                            break;
                        }
                    }
                } else {
                    while let Some(m_inner) = self.inner.next() {
                        self.inner_cache.push(m_inner.clone());

                        match_candidate_buffer.push((m_outer.clone(), m_inner.clone(), tx.clone()));

                        if match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                            break;
                        }
                    }
                }
                // inner was completed once, use cache from now, or reset to first item once completed
                self.pos_inner_cache = Some(0)
            }

            // consume next outer
            if self.outer.next().is_none() {
                break;
            }
        }
        return match_candidate_buffer; 
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        
        let (tx, rx) = channel();
        let mut match_candidate_buffer = self.next_match_buffer(tx);

        if match_candidate_buffer.is_empty() {
            return None;
        }

        let left_is_outer = self.left_is_outer;
        let outer_idx = self.outer_idx;
        let inner_idx = self.inner_idx;
        let op = self.op.clone();
        
        let op: &Operator = op.as_ref();

        match_candidate_buffer.par_iter_mut().for_each(|(m_outer, m_inner, tx)| {
            let filter_true = if left_is_outer {
                op.filter_match(&m_outer[outer_idx], &m_inner[inner_idx])
            } else {
                op.filter_match(&m_inner[inner_idx], &m_outer[outer_idx])
            };
            if filter_true
            {
                // filter by reflexivity if necessary
                if op.is_reflexive()
                    || m_outer[outer_idx].node != m_inner[inner_idx].node
                    || !util::check_annotation_key_equal(
                        &m_outer[outer_idx].anno,
                        &m_inner[inner_idx].anno,
                    ) {
                    let mut result = m_outer.clone();
                    result.append(&mut m_inner.clone());

                    tx.send(result);
                }
            }
        });
        return Some(rx);
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

            // get new candidates
            if let Some(rhs) = self.next_match_receiver() {
                self.match_receiver = Some(rhs);
            } else {
                // no more results to fetch
                return None;
            }
        }
    }
}
