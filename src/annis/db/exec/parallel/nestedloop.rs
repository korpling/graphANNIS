use super::super::{Desc, ExecutionNode};
use crate::annis::db::query::conjunction::BinaryOperatorEntry;
use crate::annis::db::Match;
use crate::annis::operator::BinaryOperator;
use rayon::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 1024;

pub struct NestedLoop<'a> {
    outer: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
    inner: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
    op: Arc<BinaryOperator>,
    inner_idx: usize,
    outer_idx: usize,

    current_outer: Option<Arc<Vec<Match>>>,
    match_candidate_buffer: Vec<MatchCandidate>,
    match_receiver: Option<Receiver<Vec<Match>>>,
    inner_cache: Vec<Arc<Vec<Match>>>,
    pos_inner_cache: Option<usize>,

    left_is_outer: bool,
    desc: Desc,

    global_reflexivity: bool,
}

type MatchCandidate = (Arc<Vec<Match>>, Arc<Vec<Match>>, Sender<Vec<Match>>);

impl<'a> NestedLoop<'a> {
    pub fn new(
        op_entry: BinaryOperatorEntry,
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

                outer: lhs,
                inner: rhs,
                op: Arc::from(op_entry.op),
                outer_idx: lhs_idx,
                inner_idx: rhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.global_reflexivity,
                match_candidate_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
                current_outer: None,
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

                outer: rhs,
                inner: lhs,
                op: Arc::from(op_entry.op),
                outer_idx: rhs_idx,
                inner_idx: lhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.global_reflexivity,
                match_candidate_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
                current_outer: None,
            }
        }
    }

    fn peek_outer(&mut self) -> Option<Arc<Vec<Match>>> {
        if self.current_outer.is_none() {
            if let Some(result) = self.outer.next() {
                self.current_outer = Some(Arc::from(result));
            } else {
                self.current_outer = None;
            }
        }

        if let Some(result) = &self.current_outer {
            return Some(result.clone());
        } else {
            return None;
        }
    }

    fn next_match_buffer<'b>(&'b mut self, tx: &Sender<Vec<Match>>) {
        self.match_candidate_buffer.clear();

        while self.match_candidate_buffer.len() < MAX_BUFFER_SIZE {
            if let Some(m_outer) = self.peek_outer() {
                if self.pos_inner_cache.is_some() {
                    let mut cache_pos = self.pos_inner_cache.unwrap();

                    while cache_pos < self.inner_cache.len() {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);

                        self.match_candidate_buffer.push((
                            m_outer.clone(),
                            m_inner.clone(),
                            tx.clone(),
                        ));

                        if self.match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                            return;
                        }
                    }
                } else {
                    while let Some(m_inner) = self.inner.next() {
                        let m_inner: Arc<Vec<Match>> = Arc::from(m_inner);

                        self.inner_cache.push(m_inner.clone());

                        self.match_candidate_buffer
                            .push((m_outer.clone(), m_inner, tx.clone()));

                        if self.match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                            return;
                        }
                    }
                }
                // inner was completed once, use cache from now, or reset to first item once completed
                self.pos_inner_cache = Some(0)
            }

            // consume next outer
            self.current_outer = None;
            if self.peek_outer().is_none() {
                return;
            }
        }
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Vec<Match>>> {
        let (tx, rx) = channel();

        self.next_match_buffer(&tx);

        if self.match_candidate_buffer.is_empty() {
            return None;
        }

        let left_is_outer = self.left_is_outer;
        let outer_idx = self.outer_idx;
        let inner_idx = self.inner_idx;
        let op = self.op.clone();

        let op: &BinaryOperator = op.as_ref();
        let global_reflexivity = self.global_reflexivity;

        self.match_candidate_buffer
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
                    let mut result = Vec::with_capacity(m_outer.len() + m_inner.len());
                    result.extend(m_outer.iter().cloned());
                    result.extend(m_inner.iter().cloned());

                    if tx.send(result).is_err() {
                        return;
                    }
                }
            });
        self.match_candidate_buffer.clear();

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
