use std::collections::HashMap;
use std::sync::Arc;
use types::Edge;
use annostorage::AnnoStorage;
use {Component, Match, NodeDesc};
use graphdb::GraphDB;
use graphstorage::GraphStatistic;
use operator::{Operator, OperatorSpec};
use exec::{CostEstimate, Desc, ExecutionNode, NodeSearchDesc};
use exec::indexjoin::IndexJoin;
use exec::parallel;
use exec::nestedloop::NestedLoop;
use exec::nodesearch::{NodeSearch, NodeSearchSpec};
use exec::binary_filter::BinaryFilter;

use super::disjunction::Disjunction;
use super::Config;

use std::collections::BTreeMap;
use std::iter::FromIterator;

use rand::XorShiftRng;
use rand::SeedableRng;
use rand::distributions::Range;
use rand::distributions::Distribution;

use errors::*;

#[derive(Debug)]
struct OperatorEntry<'a> {
    op: Box<OperatorSpec + 'a>,
    idx_left: usize,
    idx_right: usize,
    /*    original_order: usize, */
}

#[derive(Debug)]
pub struct Conjunction<'a> {
    nodes: Vec<(String, NodeSearchSpec)>,
    operators: Vec<OperatorEntry<'a>>,
    variables : HashMap<String, usize>,
}

fn update_components_for_nodes(
    node2component: &mut BTreeMap<usize, usize>,
    from: usize,
    to: usize,
) {
    if from == to {
        // nothing todo
        return;
    }

    let mut node_ids_to_update: Vec<usize> = Vec::new();
    for (k, v) in node2component.iter() {
        if *v == from {
            node_ids_to_update.push(*k);
        }
    }

    // set the component id for each node of the other component
    for nid in node_ids_to_update.iter() {
        node2component.insert(*nid, to);
    }
}

fn should_switch_operand_order<'a>(
    op_entry: &OperatorEntry,
    node2cost: &BTreeMap<usize, CostEstimate>,
) -> bool {

    if let (Some(cost_lhs), Some(cost_rhs)) = (
        node2cost.get(&op_entry.idx_left),
        node2cost.get(&op_entry.idx_right),
    ) {
        let cost_lhs: &CostEstimate = cost_lhs;
        let cost_rhs: &CostEstimate = cost_rhs;

        if cost_rhs.output < cost_lhs.output {
            // switch operands
            return true;
        }
    }

    return false;
}

impl<'a> Conjunction<'a> {
    pub fn new() -> Conjunction<'a> {
        Conjunction {
            nodes: vec![],
            operators: vec![],
            variables: HashMap::default(),
        }
    }

    pub fn into_disjunction(self) -> Disjunction<'a> {
        Disjunction::new(vec![self])
    }

    pub fn get_node_descriptions(&self) -> Vec<NodeDesc> {
        let mut result = Vec::default();
        for (var, spec) in self.nodes.iter() {
            let anno_name = match spec {
                NodeSearchSpec::ExactValue{name, ..} => Some(name.clone()),
                NodeSearchSpec::RegexValue{name, ..} => Some(name.clone()),
                _ => None,
            };
            let desc = NodeDesc {
                component_nr: 0,
                aql_fragment: format!("{}", spec),
                variable: var.clone(),
                anno_name,
            };
            result.push(desc);
        }
        return result; 
    }

    pub fn add_node(&mut self, node: NodeSearchSpec, variable : Option<&str>,) -> String {
        let idx = self.nodes.len();
        let variable = if let Some(variable) = variable {
            variable.to_string()
        } else {
            (idx+1).to_string()
        };
        self.nodes.push((variable.clone(), node));
        self.variables.insert(variable.clone(), idx);
        return variable;
    }

    pub fn add_operator(&mut self, op: Box<OperatorSpec>, var_left: &str, var_right: &str) -> Result<()> {
        //let original_order = self.operators.len();
        if let (Some(idx_left), Some(idx_right)) = (self.variables.get(var_left), self.variables.get(var_right)) {
            self.operators.push(OperatorEntry {
                op,
                idx_left: idx_left.clone(),
                idx_right: idx_right.clone(),
            });
            return Ok(());
        } else {
            return Err(ErrorKind::AQLSemanticError("Operand not found".into()).into());
        }

    }

    pub fn num_of_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_variable_pos(&self, variable : &str) -> Option<usize> {
        self.variables.get(variable).cloned()
    }

    pub fn necessary_components(&self, db : &GraphDB) -> Vec<Component> {
        let mut result = vec![];

        for op_entry in self.operators.iter() {
            let mut c = op_entry.op.necessary_components(db);
            result.append(&mut c);
        }

        return result;
    }

    fn optimize_join_order_heuristics(&self, db: &'a GraphDB, config : &Config) -> Result<Vec<usize>> {
        // check if there is something to optimize
        if self.operators.is_empty() {
            return Ok(vec![]);
        } else if self.operators.len() == 1 {
            return Ok(vec![0]);
        }

        // use a constant seed to make the result deterministic
        let mut rng = XorShiftRng::from_seed(*b"Graphs are great");
        let dist = Range::new(0, self.operators.len());

        let mut best_operator_order = Vec::from_iter(0..self.operators.len());

        // TODO: cache the base estimates
        let initial_plan = self.make_exec_plan_with_order(db, config, best_operator_order.clone())?;
        let mut best_cost = initial_plan
            .get_desc()
            .ok_or("Plan description missing")?
            .cost
            .clone()
            .ok_or("Plan cost missing")?
            .intermediate_sum
            .clone();
        trace!(
            "initial plan:\n{}",
            initial_plan
                .get_desc()
                .ok_or("Plan description missing")?
                .debug_string("  ")
        );

        let num_new_generations = 4;
        let max_unsuccessful_tries = 5 * self.operators.len();
        let mut unsucessful = 0;
        while unsucessful < max_unsuccessful_tries {
            let mut family_operators: Vec<Vec<usize>> = Vec::new();
            family_operators.reserve(num_new_generations + 1);

            family_operators.push(best_operator_order.clone());

            for i in 0..num_new_generations {
                // use the the previous generation as basis
                let mut tmp_operators = family_operators[i].clone();
                // randomly select two joins
                let mut a = 0;
                let mut b = 0;
                while a == b {
                    a = dist.sample(&mut rng);
                    b = dist.sample(&mut rng);
                }
                // switch the order of the selected joins
                tmp_operators.swap(a, b);
                family_operators.push(tmp_operators);
            }

            let mut found_better_plan = false;
            for i in 1..family_operators.len() {
                let alt_plan = self.make_exec_plan_with_order(db, config, family_operators[i].clone())?;
                let alt_cost = alt_plan
                    .get_desc()
                    .ok_or("Plan description missing")?
                    .cost
                    .clone()
                    .ok_or("Plan cost missing")?
                    .intermediate_sum;
                trace!(
                    "alternatives plan: \n{}",
                    initial_plan
                        .get_desc()
                        .ok_or("Plan description missing")?
                        .debug_string("  ")
                );

                if alt_cost < best_cost {
                    best_operator_order = family_operators[i].clone();
                    found_better_plan = true;
                    trace!("Found better plan");
                    best_cost = alt_cost;
                    unsucessful = 0;
                }
            }

            if !found_better_plan {
                unsucessful += 1;
            }
        }

        Ok(best_operator_order)
    }

    fn optimize_node_search_by_operator(
        &'a self,
        node_search_desc: Arc<NodeSearchDesc>,
        desc: Option<&Desc>,
        op_entries: Box<Iterator<Item = &'a OperatorEntry> + 'a>,
        db: &'a GraphDB,
    ) -> Option<Box<ExecutionNode<Item = Vec<Match>> + 'a>> {
        let desc = desc?;
        // check if we can replace this node search with a generic "all nodes from either of these components" search
        let node_search_cost: &CostEstimate = desc.cost.as_ref()?;

        for e in op_entries {
            let op_spec = &e.op;
            if e.idx_left == desc.component_nr { 
                // get the necessary components and count the number of nodes in these components
                let components = op_spec.necessary_components(db);
                if components.len() > 0 {
                    let mut estimated_component_search = 0;

                    let mut estimation_valid = false;
                    for c in components.iter() {
                        if let Some(gs) = db.get_graphstorage(c) {
                            // check if we can apply an even more restrictive edge annotation search
                            if let Some(edge_anno_spec) = op_spec.get_edge_anno_spec() {
                                let anno_storage: &AnnoStorage<Edge> = gs.get_anno_storage();
                                if let Some(edge_anno_est) =
                                    edge_anno_spec.guess_max_count(&anno_storage, &db.strings)
                                {
                                    estimated_component_search += edge_anno_est;
                                    estimation_valid = true;
                                }
                            } else if let Some(stats) = gs.get_statistics() {
                                let stats: &GraphStatistic = stats;
                                estimated_component_search += stats.nodes;
                                estimation_valid = true;
                            }
                        }
                    }

                    if estimation_valid && node_search_cost.output > estimated_component_search {
                        let poc_search = NodeSearch::new_partofcomponentsearch(
                            db,
                            node_search_desc,
                            Some(desc),
                            components,
                            op_spec.get_edge_anno_spec(),
                        );
                        if let Ok(poc_search) = poc_search{
                            // TODO: check if there is another operator with even better estimates
                            return Some(Box::new(poc_search));
                        } else {
                            return None;
                        }
                    }
                }
            }
        }

        return None;
    }

    fn create_join<'b>(&self, 
        db: &GraphDB,
        config: &Config,
        op: Box<Operator>,
        exec_left: Box<ExecutionNode<Item = Vec<Match>> + 'b> , 
        exec_right : Box<ExecutionNode<Item = Vec<Match>> + 'b>,
        spec_idx_left: usize, spec_idx_right: usize,
        idx_left: usize, idx_right: usize) -> Box<ExecutionNode<Item = Vec<Match>> + 'b> {
        
        if exec_right.as_nodesearch().is_some() {
            // use index join
            if config.use_parallel_joins {
                let join = parallel::indexjoin::IndexJoin::new(
                    exec_left,
                    idx_left,
                    spec_idx_left + 1,
                    spec_idx_right + 1,
                    op,
                    exec_right.as_nodesearch().unwrap().get_node_search_desc(),
                    db.node_annos.clone(),
                    db.strings.clone(),
                    exec_right.get_desc(),
                );
                return Box::new(join);
            } else {
                let join = IndexJoin::new(
                    exec_left,
                    idx_left,
                    spec_idx_left + 1,
                    spec_idx_right + 1,
                    op,
                    exec_right.as_nodesearch().unwrap().get_node_search_desc(),
                    db.node_annos.clone(),
                    db.strings.clone(),
                    exec_right.get_desc(),
                );
                return Box::new(join);
            }
        } else if exec_left.as_nodesearch().is_some() {

            // avoid a nested loop join by switching the operand and using and index join
            if let Some(inverse_op) = op.get_inverse_operator() {
                if config.use_parallel_joins {
                    let join = parallel::indexjoin::IndexJoin::new(
                        exec_right,
                        idx_right,
                        spec_idx_right + 1,
                        spec_idx_left + 1,
                        inverse_op,
                        exec_left.as_nodesearch().unwrap().get_node_search_desc(),
                        db.node_annos.clone(),
                        db.strings.clone(),
                        exec_left.get_desc(),
                    );
                    return Box::new(join);
                } else {
                    let join = IndexJoin::new(
                        exec_right,
                        idx_right,
                        spec_idx_right + 1,
                        spec_idx_left + 1,
                        inverse_op,
                        exec_left.as_nodesearch().unwrap().get_node_search_desc(),
                        db.node_annos.clone(),
                        db.strings.clone(),
                        exec_left.get_desc(),
                    );
                    return Box::new(join);
                }
            }
        }

         // use nested loop as "fallback"
        if config.use_parallel_joins {
            let join = parallel::nestedloop::NestedLoop::new(
                exec_left,
                exec_right,
                idx_left,
                idx_right,
                spec_idx_left + 1,
                spec_idx_right + 1,
                op,
            );
            return Box::new(join);
        } else {
            let join = NestedLoop::new(
                exec_left,
                exec_right,
                idx_left,
                idx_right,
                spec_idx_left + 1,
                spec_idx_right + 1,
                op,
            );
            return Box::new(join);
        }
    }

    fn make_exec_plan_with_order(
        &'a self,
        db: &'a GraphDB,
        config: &Config,
        operator_order: Vec<usize>,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>> {
        let mut node2component: BTreeMap<usize, usize> = BTreeMap::new();

        // 1. add all nodes

        // Create a map where the key is the component number
        // and move all nodes with their index as component number.
        let mut component2exec: BTreeMap<usize, Box<ExecutionNode<Item = Vec<Match>> + 'a>> =
            BTreeMap::new();
        let mut node2cost: BTreeMap<usize, CostEstimate> = BTreeMap::new();

        {
            for node_nr in 0..self.nodes.len() {
                let n_spec = &self.nodes[node_nr].1;
                let mut node_search = NodeSearch::from_spec(n_spec.clone(), node_nr, db)?;
                node2component.insert(node_nr, node_nr);

                let (orig_query_frag, orig_impl_desc, cost) =
                    if let Some(d) = node_search.get_desc() {
                        if let Some(ref c) = d.cost {
                            node2cost.insert(node_nr, c.clone());
                        }

                        (
                            d.query_fragment.clone(),
                            d.impl_description.clone(),
                            d.cost.clone(),
                        )
                    } else {
                        (String::from(""), String::from(""), None)
                    };
                // make sure the description is correct
                let mut node_pos = BTreeMap::new();
                node_pos.insert(node_nr.clone(), 0);
                let new_desc = Desc {
                    component_nr: node_nr,
                    lhs: None,
                    rhs: None,
                    node_pos,
                    impl_description: orig_impl_desc,
                    query_fragment: orig_query_frag,
                    cost: cost,
                };
                node_search.set_desc(Some(new_desc));

                let node_by_component_search =  self.optimize_node_search_by_operator(
                        node_search.get_node_search_desc(),
                        node_search.get_desc(),
                        Box::new(self.operators.iter()),
                        db,
                );

                // move to map
                if let Some(node_by_component_search) = node_by_component_search {
                    component2exec.insert(node_nr, node_by_component_search);
                } else {
                    component2exec.insert(node_nr, Box::new(node_search));
                }
            }
        }

        // 2. add the joins which produce the results in operand order
        for i in operator_order.into_iter() {
            let op_entry: &OperatorEntry<'a> = &self.operators[i];

            let mut op: Box<Operator> = op_entry.op.create_operator(db).ok_or(
                ErrorKind::ImpossibleSearch(format!("could not create operator {:?}", op_entry)),
            )?;

            let mut spec_idx_left = op_entry.idx_left;
            let mut spec_idx_right = op_entry.idx_right;

            let inverse_op = op.get_inverse_operator();
            if let Some(inverse_op) = inverse_op {
                if should_switch_operand_order(op_entry, &node2cost) {
                    spec_idx_left = op_entry.idx_right;
                    spec_idx_right = op_entry.idx_left;

                    op = inverse_op;
                }
            }


            let component_left = node2component
                .get(&spec_idx_left)
                .ok_or(ErrorKind::ImpossibleSearch(format!(
                    "no component for node #{}",
                    spec_idx_left + 1
                )))?
                .clone();
            let component_right = node2component
                .get(&spec_idx_right)
                .ok_or(ErrorKind::ImpossibleSearch(format!(
                    "no component for node #{}",
                    spec_idx_right + 1
                )))?
                .clone();

            // get the original execution node
            let exec_left: Box<ExecutionNode<Item = Vec<Match>> + 'a> = component2exec
                .remove(&component_left)
                .ok_or(ErrorKind::ImpossibleSearch(format!(
                    "no execution node for component {}",
                    component_left
                )))?;


            let idx_left = exec_left
                .get_desc()
                .ok_or("Plan description missing")?
                .node_pos
                .get(&spec_idx_left)
                .ok_or("LHS operand not found")?
                .clone();

            let new_exec: Box<ExecutionNode<Item = Vec<Match>>> = if component_left
                == component_right
            {
                // don't create new tuples, only filter the existing ones
                // TODO: check if LHS or RHS is better suited as filter input iterator
                let idx_right = exec_left
                    .get_desc()
                    .ok_or("Plan description missing")?
                    .node_pos
                    .get(&spec_idx_right)
                    .ok_or("RHS operand not found")?
                    .clone();

                let filter = BinaryFilter::new(
                    exec_left,
                    idx_left,
                    idx_right,
                    spec_idx_left + 1,
                    spec_idx_right + 1,
                    op,
                );
                Box::new(filter)
            } else {
                let exec_right = component2exec.remove(&component_right).ok_or(
                    ErrorKind::ImpossibleSearch(format!(
                        "no execution node for component {}",
                        component_right
                    )),
                )?;
                let idx_right = exec_right
                    .get_desc()
                    .ok_or("Plan description missing")?
                    .node_pos
                    .get(&spec_idx_right)
                    .ok_or("RHS operand not found")?
                    .clone();

                self.create_join(db, config, op, exec_left, exec_right, spec_idx_left, spec_idx_right, idx_left, idx_right)
            };

            let new_component_nr = new_exec
                .get_desc()
                .ok_or(ErrorKind::ImpossibleSearch(String::from(
                    "missing description for execution node",
                )))?
                .component_nr;
            update_components_for_nodes(&mut node2component, component_left, new_component_nr);
            update_components_for_nodes(&mut node2component, component_right, new_component_nr);
            component2exec.insert(new_component_nr, new_exec);
        }

        // 3. check if there is only one component left (all nodes are connected)
        let mut first_component_id: Option<usize> = None;
        for (_, cid) in node2component.iter() {
            if first_component_id.is_none() {
                first_component_id = Some(*cid);
            } else if let Some(first) = first_component_id {
                if first != *cid {
                    return Err(ErrorKind::AQLSemanticError("Components not connected".to_string()).into());
                }
            }
        }

        let first_component_id = first_component_id.ok_or(ErrorKind::ImpossibleSearch(String::from(
            "no component in query at all",
        )))?;
        return component2exec
            .remove(&first_component_id)
            .ok_or(ErrorKind::ImpossibleSearch(String::from(
                "could not find execution node for query component",
            )).into());
    }

    pub fn make_exec_node(
        &'a self,
        db: &'a GraphDB,
        config: &Config,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>> {
        let operator_order = self.optimize_join_order_heuristics(db, config)?;
        return self.make_exec_plan_with_order(db, config, operator_order);
    }
}
