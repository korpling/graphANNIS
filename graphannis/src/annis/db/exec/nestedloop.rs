use graphannis_core::annostorage::MatchGroup;

use super::{ExecutionNode, ExecutionNodeDesc};
use crate::annis::db::aql::conjunction::BinaryOperatorEntry;
use crate::annis::operator::{BinaryOperator, BinaryOperatorBase};
use crate::errors::Result;
use crate::try_as_option;
use std::iter::Peekable;

pub struct NestedLoop<'a> {
    outer: Peekable<Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>>,
    inner: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
    op: BinaryOperator<'a>,
    inner_idx: usize,
    outer_idx: usize,
    inner_cache: Vec<MatchGroup>,
    pos_inner_cache: Option<usize>,

    left_is_outer: bool,
    desc: ExecutionNodeDesc,

    global_reflexivity: bool,
}

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
            if let (Some(cost_lhs), Some(cost_rhs)) = (&desc_lhs.cost, &desc_rhs.cost) {
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
                    "nestedloop L-R",
                    &format!(
                        "#{} {} #{}",
                        op_entry.args.left, op_entry.op, op_entry.args.right
                    ),
                    &processed_func,
                )?,

                outer: lhs.peekable(),
                inner: rhs,
                op: op_entry.op,
                outer_idx: lhs_idx,
                inner_idx: rhs_idx,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.args.global_reflexivity,
            };
            Ok(join)
        } else {
            let join = NestedLoop {
                desc: ExecutionNodeDesc::join(
                    &op_entry.op,
                    rhs.get_desc(),
                    lhs.get_desc(),
                    "nestedloop R-L",
                    &format!(
                        "#{} {} #{}",
                        op_entry.args.left, op_entry.op, op_entry.args.right
                    ),
                    &processed_func,
                )?,

                outer: rhs.peekable(),
                inner: lhs,
                op: op_entry.op,
                outer_idx: rhs_idx,
                inner_idx: lhs_idx,
                inner_cache: Vec::new(),
                pos_inner_cache: None,
                left_is_outer,
                global_reflexivity: op_entry.args.global_reflexivity,
            };
            Ok(join)
        }
    }
}

impl<'a> ExecutionNode for NestedLoop<'a> {
    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        Some(&self.desc)
    }
}

impl<'a> Iterator for NestedLoop<'a> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(Ok(m_outer)) = self.outer.peek() {
                if let Some(mut cache_pos) = self.pos_inner_cache {
                    while cache_pos < self.inner_cache.len() {
                        let m_inner = &self.inner_cache[cache_pos];
                        cache_pos += 1;
                        self.pos_inner_cache = Some(cache_pos);
                        let filter_true = if self.left_is_outer {
                            self.op
                                .filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx])
                        } else {
                            self.op
                                .filter_match(&m_inner[self.inner_idx], &m_outer[self.outer_idx])
                        };
                        let filter_true = try_as_option!(filter_true);
                        // filter by reflexivity if necessary
                        if filter_true
                            && (self.op.is_reflexive()
                                || (self.global_reflexivity
                                    && m_outer[self.outer_idx].different_to_all(m_inner)
                                    && m_inner[self.inner_idx].different_to_all(m_outer))
                                || (!self.global_reflexivity
                                    && m_outer[self.outer_idx]
                                        .different_to(&m_inner[self.inner_idx])))
                        {
                            let mut result = m_outer.clone();
                            result.append(&mut m_inner.clone());
                            return Some(Ok(result));
                        }
                    }
                } else {
                    for m_inner in &mut self.inner {
                        let mut m_inner = try_as_option!(m_inner);
                        self.inner_cache.push(m_inner.clone());

                        let filter_true = if self.left_is_outer {
                            self.op
                                .filter_match(&m_outer[self.outer_idx], &m_inner[self.inner_idx])
                        } else {
                            self.op
                                .filter_match(&m_inner[self.inner_idx], &m_outer[self.outer_idx])
                        };
                        // filter by reflexivity if necessary
                        let filter_true = try_as_option!(filter_true);
                        if filter_true
                            && (self.op.is_reflexive()
                                || m_outer[self.outer_idx].node != m_inner[self.inner_idx].node
                                || m_outer[self.outer_idx].anno_key
                                    != m_inner[self.inner_idx].anno_key)
                        {
                            let mut result = m_outer.clone();
                            result.append(&mut m_inner);
                            return Some(Ok(result));
                        }
                    }
                }
                // inner was completed once, use cache from now, or reset to first item once completed
                self.pos_inner_cache = Some(0)
            }

            // consume next outer and fail on error
            match self.outer.next()? {
                Ok(_) => {}
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
    }
}
