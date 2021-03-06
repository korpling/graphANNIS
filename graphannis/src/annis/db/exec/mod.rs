use self::nodesearch::NodeSearch;
use crate::annis::db::AnnotationStorage;
use crate::{
    annis::operator::{BinaryOperator, EstimationType},
    graph::Match,
};
use graphannis_core::{
    annostorage::MatchGroup,
    types::{AnnoKey, NodeID},
};

use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub output: usize,
    pub intermediate_sum: usize,
    pub processed_in_step: usize,
}

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr: usize,
    pub lhs: Option<Box<Desc>>,
    pub rhs: Option<Box<Desc>>,
    /// Maps the index of the node in the actual result to the index in the internal execution plan intermediate result.
    pub node_pos: BTreeMap<usize, usize>,
    pub impl_description: String,
    pub query_fragment: String,
    pub cost: Option<CostEstimate>,
}

fn calculate_outputsize(
    op: &dyn BinaryOperator,
    cost_lhs: &CostEstimate,
    cost_rhs: &CostEstimate,
) -> usize {
    let output = match op.estimation_type() {
        EstimationType::SELECTIVITY(selectivity) => {
            let num_tuples = (cost_lhs.output * cost_rhs.output) as f64;
            if let Some(edge_sel) = op.edge_anno_selectivity() {
                (num_tuples * selectivity * edge_sel).round() as usize
            } else {
                (num_tuples * selectivity).round() as usize
            }
        }
        EstimationType::MIN => std::cmp::min(cost_lhs.output, cost_rhs.output),
    };
    // always assume at least one output item otherwise very small selectivity can fool the planner
    std::cmp::max(output, 1)
}

pub struct NodeDescArg {
    query_fragment: String,
    node_nr: usize,
}

impl Desc {
    pub fn empty_with_fragment(node_desc_arg: NodeDescArg, est_size: Option<usize>) -> Desc {
        let mut node_pos = BTreeMap::new();
        node_pos.insert(node_desc_arg.node_nr, 0);

        let cost = if let Some(output) = est_size {
            Some(CostEstimate {
                output,
                intermediate_sum: 0,
                processed_in_step: 0,
            })
        } else {
            None
        };

        Desc {
            component_nr: 0,
            lhs: None,
            rhs: None,
            node_pos,
            impl_description: String::from(""),
            query_fragment: node_desc_arg.query_fragment,
            cost,
        }
    }

    pub fn join(
        op: &dyn BinaryOperator,
        lhs: Option<&Desc>,
        rhs: Option<&Desc>,
        impl_description: &str,
        query_fragment: &str,
        processed_func: &dyn Fn(EstimationType, usize, usize) -> usize,
    ) -> Desc {
        let component_nr = if let Some(d) = lhs {
            d.component_nr
        } else if let Some(d) = rhs {
            d.component_nr
        } else {
            0
        };

        // merge both node positions
        let mut node_pos = BTreeMap::new();
        let offset = if let Some(ref lhs) = lhs {
            node_pos = lhs.node_pos.clone();
            node_pos.len()
        } else {
            0
        };
        if let Some(ref rhs) = rhs {
            for e in &rhs.node_pos {
                // the RHS has an offset after the join
                node_pos.insert(*e.0, e.1 + offset);
            }
        }

        // merge costs
        let cost = if let (Some(ref lhs), Some(ref rhs)) = (lhs, rhs) {
            if let (&Some(ref cost_lhs), &Some(ref cost_rhs)) = (&lhs.cost, &rhs.cost) {
                // estimate output size using the operator
                let output = calculate_outputsize(op, cost_lhs, cost_rhs);

                let processed_in_step =
                    processed_func(op.estimation_type(), cost_lhs.output, cost_rhs.output);
                Some(CostEstimate {
                    output,
                    intermediate_sum: processed_in_step
                        + cost_lhs.intermediate_sum
                        + cost_rhs.intermediate_sum,
                    processed_in_step,
                })
            } else {
                None
            }
        } else {
            None
        };

        Desc {
            component_nr,
            lhs: lhs.map(|x| Box::new(x.clone())),
            rhs: rhs.map(|x| Box::new(x.clone())),
            node_pos,
            impl_description: String::from(impl_description),
            query_fragment: String::from(query_fragment),
            cost,
        }
    }

    pub fn debug_string(&self, indention: &str) -> String {
        let mut result = String::from(indention);

        let cost_str = if let Some(ref cost) = self.cost {
            format!(
                "out: {}, sum: {}, instep: {}",
                cost.output, cost.intermediate_sum, cost.processed_in_step
            )
        } else {
            String::from("no cost estimated")
        };

        // output the node number and query fragment for base nodes
        if self.lhs.is_none() && self.rhs.is_none() {
            let node_nr = self.node_pos.keys().next().cloned().unwrap_or(0) + 1;

            result.push_str(&format!(
                "#{} ({}) [{}] {}\n",
                &node_nr.to_string(),
                &self.query_fragment,
                &cost_str,
                &self.impl_description,
            ));
        } else {
            result.push_str(&format!(
                "+|{} ({}) [{}]\n",
                &self.impl_description, &self.query_fragment, &cost_str
            ));

            let new_indention = format!("{}    ", indention);
            if let Some(ref lhs) = self.lhs {
                result.push_str(&lhs.debug_string(&new_indention));
            }
            if let Some(ref rhs) = self.rhs {
                result.push_str(&rhs.debug_string(&new_indention));
            }
        }
        result
    }
}

pub type MatchFilterFunc =
    Box<dyn Fn(&Match, &dyn AnnotationStorage<NodeID>) -> bool + Send + Sync>;

pub struct NodeSearchDesc {
    pub qname: (Option<String>, Option<String>),
    pub cond: Vec<MatchFilterFunc>,
    pub const_output: Option<Arc<AnnoKey>>,
}

pub trait ExecutionNode: Iterator {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = MatchGroup>;
    fn as_nodesearch<'a>(&'a self) -> Option<&'a NodeSearch> {
        None
    }

    fn get_desc(&self) -> Option<&Desc> {
        None
    }

    fn is_sorted_by_text(&self) -> bool {
        false
    }
}

pub struct EmptyResultSet;

impl Iterator for EmptyResultSet {
    type Item = MatchGroup;

    fn next(&mut self) -> Option<MatchGroup> {
        None
    }
}

impl ExecutionNode for EmptyResultSet {
    fn as_iter(&mut self) -> &mut dyn Iterator<Item = MatchGroup> {
        self
    }
    fn as_nodesearch<'a>(&'a self) -> Option<&'a NodeSearch> {
        None
    }

    fn get_desc(&self) -> Option<&Desc> {
        None
    }
}

pub mod filter;
pub mod indexjoin;
pub mod nestedloop;
pub mod nodesearch;
pub mod parallel;
pub mod tokensearch;
