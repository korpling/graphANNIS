use graphannis_core::annostorage::MatchGroup;
use itertools::Itertools;

use super::{CostEstimate, ExecutionNode, ExecutionNodeDesc};
use crate::annis::db::aql::conjunction::{BinaryOperatorEntry, UnaryOperatorEntry};
use crate::annis::operator::{BinaryOperatorBase, EstimationType, UnaryOperator};
use crate::errors::Result;

pub struct Filter<'a> {
    it: Box<dyn Iterator<Item = Result<MatchGroup>> + 'a>,
    desc: Option<ExecutionNodeDesc>,
}

fn calculate_binary_outputsize(op: &dyn BinaryOperatorBase, num_tuples: usize) -> Result<usize> {
    let output = match op.estimation_type()? {
        EstimationType::Selectivity(selectivity) => {
            let num_tuples = num_tuples as f64;
            if let Some(edge_sel) = op.edge_anno_selectivity()? {
                (num_tuples * selectivity * edge_sel).round() as usize
            } else {
                (num_tuples * selectivity).round() as usize
            }
        }
        EstimationType::Min => num_tuples,
    };
    // always assume at least one output item otherwise very small selectivity can fool the planner
    Ok(std::cmp::max(output, 1))
}

fn calculate_unary_outputsize(op: &dyn UnaryOperator, num_tuples: usize) -> usize {
    let output = match op.estimation_type() {
        EstimationType::Selectivity(selectivity) => {
            let num_tuples = num_tuples as f64;
            (num_tuples * selectivity).round() as usize
        }
        EstimationType::Min => num_tuples,
    };
    // always assume at least one output item otherwise very small selectivity can fool the planner
    std::cmp::max(output, 1)
}

impl<'a> Filter<'a> {
    pub fn new_binary(
        exec: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
        op_entry: BinaryOperatorEntry<'a>,
    ) -> Result<Filter<'a>> {
        let desc = if let Some(orig_desc) = exec.get_desc() {
            let cost_est = if let Some(ref orig_cost) = orig_desc.cost {
                Some(CostEstimate {
                    output: calculate_binary_outputsize(&op_entry.op, orig_cost.output)?,
                    processed_in_step: orig_cost.processed_in_step,
                    intermediate_sum: orig_cost.intermediate_sum + orig_cost.processed_in_step,
                })
            } else {
                None
            };

            Some(ExecutionNodeDesc {
                component_nr: orig_desc.component_nr,
                node_pos: orig_desc.node_pos.clone(),
                impl_description: String::from("filter"),
                query_fragment: format!(
                    "#{} {} #{}",
                    op_entry.args.left, op_entry.op, op_entry.args.right
                ),
                cost: cost_est,
                lhs: Some(Box::new(orig_desc.clone())),
                rhs: None,
            })
        } else {
            None
        };
        let it = exec
            .map(move |tuple| {
                let tuple = tuple?;
                let include = op_entry.op.filter_match(&tuple[lhs_idx], &tuple[rhs_idx])?;
                if include {
                    Ok(Some(tuple))
                } else {
                    Ok(None)
                }
            })
            .filter_map_ok(|t| t);
        Ok(Filter {
            desc,
            it: Box::new(it),
        })
    }

    pub fn new_unary(
        exec: Box<dyn ExecutionNode<Item = Result<MatchGroup>> + 'a>,
        idx: usize,
        op_entry: UnaryOperatorEntry<'a>,
    ) -> Filter<'a> {
        let desc = if let Some(orig_desc) = exec.get_desc() {
            let cost_est = if let Some(ref orig_cost) = orig_desc.cost {
                Some(CostEstimate {
                    output: calculate_unary_outputsize(op_entry.op.as_ref(), orig_cost.output),
                    processed_in_step: orig_cost.processed_in_step,
                    intermediate_sum: orig_cost.intermediate_sum + orig_cost.processed_in_step,
                })
            } else {
                None
            };

            Some(ExecutionNodeDesc {
                component_nr: orig_desc.component_nr,
                node_pos: orig_desc.node_pos.clone(),
                impl_description: String::from("filter"),
                query_fragment: format!("#{}{}", op_entry.node_nr, op_entry.op,),
                cost: cost_est,
                lhs: Some(Box::new(orig_desc.clone())),
                rhs: None,
            })
        } else {
            None
        };
        let it = exec
            .map(move |tuple| {
                let tuple = tuple?;
                if op_entry.op.filter_match(&tuple[idx])? {
                    Ok(Some(tuple))
                } else {
                    Ok(None)
                }
            })
            .filter_map_ok(|t| t);
        Filter {
            desc,
            it: Box::new(it),
        }
    }
}

impl ExecutionNode for Filter<'_> {
    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        self.desc.as_ref()
    }
}

impl Iterator for Filter<'_> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}
