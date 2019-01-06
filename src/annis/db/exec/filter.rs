use super::{CostEstimate, Desc, ExecutionNode};
use crate::annis::db::query::conjunction::BinaryOperatorEntry;
use crate::annis::db::Match;
use crate::annis::operator::{EstimationType, BinaryOperator};
use std;

pub struct Filter<'a> {
    it: Box<Iterator<Item = Vec<Match>> + 'a>,
    desc: Option<Desc>,
}

fn calculate_outputsize(op: &BinaryOperator, num_tuples: usize) -> usize {
    let output = match op.estimation_type() {
        EstimationType::SELECTIVITY(selectivity) => {
            let num_tuples = num_tuples as f64;
            if let Some(edge_sel) = op.edge_anno_selectivity() {
                (num_tuples * selectivity * edge_sel).round() as usize
            } else {
                (num_tuples * selectivity).round() as usize
            }
        }
        EstimationType::MIN => num_tuples,
    };
    // always assume at least one output item otherwise very small selectivity can fool the planner
    std::cmp::max(output, 1)
}

impl<'a> Filter<'a> {
    pub fn new_binary(
        exec: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
        op_entry: BinaryOperatorEntry,
    ) -> Filter<'a> {
        let desc = if let Some(orig_desc) = exec.get_desc() {
            let cost_est = if let Some(ref orig_cost) = orig_desc.cost {
                Some(CostEstimate {
                    output: calculate_outputsize(op_entry.op.as_ref(), orig_cost.output),
                    processed_in_step: orig_cost.processed_in_step,
                    intermediate_sum: orig_cost.intermediate_sum + orig_cost.processed_in_step,
                })
            } else {
                None
            };

            Some(Desc {
                component_nr: orig_desc.component_nr,
                node_pos: orig_desc.node_pos.clone(),
                impl_description: String::from("filter"),
                query_fragment: format!(
                    "#{} {} #{}",
                    op_entry.node_nr_left, op_entry.op, op_entry.node_nr_right
                ),
                cost: cost_est,
                lhs: Some(Box::new(orig_desc.clone())),
                rhs: None,
            })
        } else {
            None
        };
        let it =
            exec.filter(move |tuple| op_entry.op.filter_match(&tuple[lhs_idx], &tuple[rhs_idx]));
        Filter {
            desc,
            it: Box::new(it),
        }
    }

}

impl<'a> ExecutionNode for Filter<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for Filter<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}
