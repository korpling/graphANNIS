use super::super::{ExecutionNode, ExecutionNodeDesc};
use crate::annis::db::aql::conjunction::BinaryOperatorEntry;
use crate::annis::operator::BinaryOperatorBase;
use crate::errors::Result;
use graphannis_core::annostorage::MatchGroup;
use rayon::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 1024;

pub struct NestedLoop<'a> {
    outer: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
    inner: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
    op: Arc<dyn BinaryOperatorBase + 'a>,
    inner_idx: usize,
    outer_idx: usize,

    current_outer: Option<Arc<MatchGroup>>,
    match_candidate_buffer: Vec<MatchCandidate>,
    match_receiver: Option<Receiver<Result<MatchGroup>>>,
    inner_cache: Vec<Arc<MatchGroup>>,
    pos_inner_cache: Option<usize>,

    left_is_outer: bool,
    desc: ExecutionNodeDesc,

    global_reflexivity: bool,
}

type MatchCandidate = (Arc<MatchGroup>, Arc<MatchGroup>, Sender<Result<MatchGroup>>);

impl<'a> NestedLoop<'a> {
    pub fn new(
        op_entry: BinaryOperatorEntry<'a>,
        lhs: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
        rhs: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
    ) -> Result<NestedLoop<'a>> {
        let mut left_is_outer = true;
        if let (Some(desc_lhs), Some(desc_rhs)) = (lhs.get_desc(), rhs.get_desc()) {
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
            let join = NestedLoop {
                desc: ExecutionNodeDesc::join(
                    &op_entry.op,
                    lhs.get_desc(),
                    rhs.get_desc(),
                    "nestedloop (parallel) L-R",
                    &format!(
                        "#{} {} #{}",
                        op_entry.args.left, op_entry.op, op_entry.args.right
                    ),
                    &processed_func,
                )?,

                outer: lhs,
                inner: rhs,
                op: Arc::from(op_entry.op),
                outer_idx: lhs_idx,
                inner_idx: rhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.args.global_reflexivity,
                match_candidate_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
                current_outer: None,
            };
            Ok(join)
        } else {
            let join = NestedLoop {
                desc: ExecutionNodeDesc::join(
                    &op_entry.op,
                    rhs.get_desc(),
                    lhs.get_desc(),
                    "nestedloop (parallel) R-L",
                    &format!(
                        "#{} {} #{}",
                        op_entry.args.left, op_entry.op, op_entry.args.right
                    ),
                    &processed_func,
                )?,

                outer: rhs,
                inner: lhs,
                op: Arc::from(op_entry.op),
                outer_idx: rhs_idx,
                inner_idx: lhs_idx,
                match_receiver: None,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.args.global_reflexivity,
                match_candidate_buffer: Vec::with_capacity(MAX_BUFFER_SIZE),
                current_outer: None,
            };
            Ok(join)
        }
    }

    fn peek_outer(&mut self) -> Result<Option<Arc<MatchGroup>>> {
        if self.current_outer.is_none() {
            if let Some(result) = self.outer.next() {
                match result {
                    Ok(result) => {
                        self.current_outer = Some(Arc::from(result));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                self.current_outer = None;
            }
        }

        Ok(self.current_outer.as_ref().cloned())
    }

    fn next_match_buffer(&mut self, tx: &Sender<Result<MatchGroup>>) {
        self.match_candidate_buffer.clear();

        while self.match_candidate_buffer.len() < MAX_BUFFER_SIZE {
            match self.peek_outer() {
                Ok(m_outer) => {
                    if let Some(m_outer) = m_outer {
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
                            for m_inner in &mut self.inner {
                                match m_inner {
                                    Ok(m_inner) => {
                                        let m_inner: Arc<MatchGroup> = Arc::from(m_inner);

                                        self.inner_cache.push(m_inner.clone());

                                        self.match_candidate_buffer.push((
                                            m_outer.clone(),
                                            m_inner,
                                            tx.clone(),
                                        ));

                                        if self.match_candidate_buffer.len() >= MAX_BUFFER_SIZE {
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        if tx.send(Err(e)).is_err() {
                                            return;
                                        }
                                    }
                                };
                            }
                        }
                        // inner was completed once, use cache from now, or reset to first item once completed
                        self.pos_inner_cache = Some(0)
                    }
                }
                Err(e) => {
                    if tx.send(Err(e)).is_err() {
                        return;
                    }
                }
            }

            // consume next outer
            self.current_outer = None;
            match self.peek_outer() {
                Ok(next_outer) => {
                    if next_outer.is_none() {
                        return;
                    }
                }
                Err(e) => {
                    if tx.send(Err(e)).is_err() {
                        return;
                    }
                }
            };
        }
    }

    fn next_match_receiver(&mut self) -> Option<Receiver<Result<MatchGroup>>> {
        let (tx, rx) = channel();

        self.next_match_buffer(&tx);

        if self.match_candidate_buffer.is_empty() {
            return None;
        }

        let left_is_outer = self.left_is_outer;
        let outer_idx = self.outer_idx;
        let inner_idx = self.inner_idx;
        let op = self.op.clone();

        let op: &dyn BinaryOperatorBase = op.as_ref();
        let global_reflexivity = self.global_reflexivity;

        self.match_candidate_buffer
            .par_iter_mut()
            .for_each(|(m_outer, m_inner, tx)| {
                let filter_true = if left_is_outer {
                    op.filter_match(&m_outer[outer_idx], &m_inner[inner_idx])
                } else {
                    op.filter_match(&m_inner[inner_idx], &m_outer[outer_idx])
                };

                match filter_true {
                    Ok(filter_true) => {
                        // filter by reflexivity if necessary
                        if filter_true
                            && (op.is_reflexive()
                                || (global_reflexivity
                                    && m_outer[outer_idx].different_to_all(m_inner)
                                    && m_inner[inner_idx].different_to_all(m_outer))
                                || (!global_reflexivity
                                    && m_outer[outer_idx].different_to(&m_inner[inner_idx])))
                        {
                            let mut result = MatchGroup::new();
                            result.extend(m_outer.iter().cloned());
                            result.extend(m_inner.iter().cloned());

                            if let Err(err) = tx.send(Ok(result)) {
                                trace!("Could not send match in nested loop: {}", err);
                            }
                        }
                    }
                    Err(e) => {
                        if let Err(e) = tx.send(Err(e)) {
                            trace!("Could not send error in parallel nested loop: {}", e);
                        }
                    }
                };
            });
        self.match_candidate_buffer.clear();

        Some(rx)
    }
}

impl<'a> ExecutionNode for NestedLoop<'a> {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = Result<MatchGroup>> {
        self
    }

    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        Some(&self.desc)
    }
}

impl<'a> Iterator for NestedLoop<'a> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
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
