use super::super::{Desc, ExecutionNode};
use crate::annis::db::query::conjunction::OperatorEntry;
use crate::annis::db::Match;
use crate::annis::operator::Operator;
use rayon::prelude::*;
use std::iter::Peekable;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 512;

pub struct NestedLoop<'a> {
    outer: Peekable<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    inner: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
    op: Arc<Operator>,
    inner_idx: usize,
    outer_idx: usize,

    match_receiver: Option<Receiver<Vec<Match>>>,
    inner_cache: Vec<Vec<Match>>,
    pos_inner_cache: Option<usize>,

    left_is_outer: bool,
    desc: Desc,

    global_reflexivity: bool,
}

type MatchCandidate = (Vec<Match>, Vec<Match>, Sender<Vec<Match>>);

impl<'a> NestedLoop<'a> {
    pub fn new(
        op_entry: OperatorEntry,
        lhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        rhs: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
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
                out_lhs + (out_lhs * out_rhs)
            } else {
                // we use RHS as outer
                out_rhs + (out_rhs * out_lhs)
            }
        };

        if left_is_outer {
            NestedLoop {
                desc: Desc::join(
                    op_entry.op.as_ref(),
                    lhs.get_desc(),
                    rhs.get_desc(),
                    "nestedloop (parallel) L-R",
                    &format!(
                        "#{} {} #{}",
                        op_entry.node_nr_left, op_entry.op, op_entry.node_nr_right
                    ),
                    &processed_func,
                ),

                outer: lhs.peekable(),
                inner: rhs,
                op: Arc::from(op_entry.op),
                outer_idx: lhs_idx,
                inner_idx: rhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.global_reflexivity,
            }
        } else {
            NestedLoop {
                desc: Desc::join(
                    op_entry.op.as_ref(),
                    rhs.get_desc(),
                    lhs.get_desc(),
                    "nestedloop (parallel) R-L",
                    &format!(
                        "#{} {} #{}",
                        op_entry.node_nr_left, op_entry.op, op_entry.node_nr_right
                    ),
                    &processed_func,
                ),

                outer: rhs.peekable(),
                inner: lhs,
                op: Arc::from(op_entry.op),
                outer_idx: rhs_idx,
                inner_idx: lhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.global_reflexivity,
            }
        }
    }

    fn next_match_buffer(&mut self, tx: &Sender<Vec<Match>>) -> Vec<MatchCandidate> {
        let mut match_candidate_buffer: Vec<MatchCandidate> = Vec::with_capacity(MAX_BUFFER_SIZE);
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
                            return match_candidate_buffer;
                        }
                    }
                } else {
                    while let Some(m_inner) = self.inner.next() {
                        self.inner_cache.push(m_inner.clone());

                        match_candidate_buffer.push((m_outer.clone(), m_inner.clone(), tx.clone()));

                        if match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                            return match_candidate_buffer;
                        }
                    }
                }
                // inner was completed once, use cache from now, or reset to first item once completed
                self.pos_inner_cache = Some(0)
            }

            // consume next outer
            if self.outer.next().is_none() {
                return match_candidate_buffer;
            }
        }
        match_candidate_buffer
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        let (tx, rx) = channel();
        let mut match_candidate_buffer = self.next_match_buffer(&tx);

        if match_candidate_buffer.is_empty() {
            return None;
        }

        let left_is_outer = self.left_is_outer;
        let outer_idx = self.outer_idx;
        let inner_idx = self.inner_idx;
        let op = self.op.clone();

        let op: &Operator = op.as_ref();
        let global_reflexivity = self.global_reflexivity;

        match_candidate_buffer
            .par_iter_mut()
            .for_each(|(m_outer, m_inner, tx)| {
                let filter_true = if left_is_outer {
                    op.filter_match(&m_outer[outer_idx], &m_inner[inner_idx])
                } else {
                    op.filter_match(&m_inner[inner_idx], &m_outer[outer_idx])
                };
                // filter by reflexivity if necessary

                if filter_true
                    && (op.is_reflexive()
                        || (global_reflexivity
                            && m_outer[outer_idx].different_to_all(&m_inner)
                            && m_inner[inner_idx].different_to_all(&m_outer))
                        || (!global_reflexivity
                            && m_outer[outer_idx].different_to(&m_inner[inner_idx])))
                {
                    let mut result = m_outer.clone();
                    result.append(&mut m_inner.clone());

                    if tx.send(result).is_err() {
                        return;
                    }
                }
            });
        Some(rx)
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
                return None;
            };
        }

        loop {
            {
                let match_receiver = self.match_receiver.as_mut()?;
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
