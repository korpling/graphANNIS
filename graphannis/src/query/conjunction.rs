use types::Edge;
use annostorage::AnnoStorage;
use {Component, Match};
use graphdb::GraphDB;
use graphstorage::{GraphStatistic};
use operator::{Operator, OperatorSpec};
use exec::{CostEstimate, Desc, ExecutionNode, NodeSearchDesc};
use exec::indexjoin::IndexJoin;
use exec::nestedloop::NestedLoop;
use exec::nodesearch::{NodeSearch, NodeSearchSpec};
use exec::binary_filter::BinaryFilter;

use super::disjunction::Disjunction;

use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::rc::Rc;

use rand::XorShiftRng;
use rand::SeedableRng;
use rand::distributions::Range;
use rand::distributions::Sample;

#[derive(Debug)]
pub enum Error {
    ImpossibleSearch(String),
    MissingDescription,
    MissingCost,
    ComponentsNotConnected,
    OperatorIdxNotFound,
}

#[derive(Debug)]
struct OperatorEntry<'a> {
    op: Box<OperatorSpec + 'a>,
    idx_left: usize,
    idx_right: usize,
    /*    original_order: usize, */
}

#[derive(Debug)]
pub struct Conjunction<'a> {
    nodes: Vec<NodeSearchSpec>,
    operators: Vec<OperatorEntry<'a>>,
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

fn optimized_operand_order<'a>(
    op_entry: &OperatorEntry,
    op: &'a Box<Operator + 'a>,
    node2cost: &BTreeMap<usize, CostEstimate>,
) -> (usize, usize) {
    if op.is_commutative() {
        if let (Some(cost_lhs), Some(cost_rhs)) = (
            node2cost.get(&op_entry.idx_left),
            node2cost.get(&op_entry.idx_right),
        ) {
            let cost_lhs: &CostEstimate = cost_lhs;
            let cost_rhs: &CostEstimate = cost_rhs;

            if cost_rhs.output < cost_lhs.output {
                // switch operands
                return (op_entry.idx_right, op_entry.idx_left);
            }
        }
    }
    return (op_entry.idx_left, op_entry.idx_right);
}

impl<'a> Conjunction<'a> {
    pub fn new() -> Conjunction<'a> {
        Conjunction {
            nodes: vec![],
            operators: vec![],
        }
    }

    pub fn into_disjunction(self) -> Disjunction<'a> {
        Disjunction::new(vec![self])
    }

    pub fn add_node(&mut self, node: NodeSearchSpec) -> usize {
        let idx = self.nodes.len();

        // TODO allow wrapping with an "any node anno" search
        self.nodes.push(node);

        idx
    }

    pub fn add_operator(&mut self, op: Box<OperatorSpec>, idx_left: usize, idx_right: usize) {
        //let original_order = self.operators.len();
        self.operators.push(OperatorEntry {
            op,
            idx_left,
            idx_right,
            /*            original_order, */
        });
    }

    pub fn necessary_components(&self) -> Vec<Component> {
        let mut result = vec![];

        for op_entry in self.operators.iter() {
            let mut c = op_entry.op.necessary_components();
            result.append(&mut c);
        }

        return result;
    }

    fn optimize_join_order_heuristics(&self, db: &'a GraphDB) -> Result<Vec<usize>, Error> {
        // check if there is something to optimize
        if self.operators.is_empty() {
            return Ok(vec![]);
        } else if self.operators.len() == 1 {
            return Ok(vec![0]);
        }

        // use a constant seed to make the result deterministic
        let mut rng = XorShiftRng::from_seed([4711, 1, 2, 3]);
        let mut dist = Range::new(0, self.operators.len());

        let mut best_operator_order = Vec::from_iter(0..self.operators.len());

        // TODO: cache the base estimates
        let initial_plan = self.make_exec_plan_with_order(db, best_operator_order.clone())?;
        let mut best_cost = initial_plan
            .get_desc()
            .ok_or(Error::MissingDescription)?
            .cost
            .clone()
            .ok_or(Error::MissingCost)?
            .intermediate_sum
            .clone();
        trace!(
            "initial plan:\n{}",
            initial_plan
                .get_desc()
                .ok_or(Error::MissingDescription)?
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
                let alt_plan = self.make_exec_plan_with_order(db, family_operators[i].clone())?;
                let alt_cost = alt_plan
                    .get_desc()
                    .ok_or(Error::MissingDescription)?
                    .cost
                    .clone()
                    .ok_or(Error::MissingCost)?
                    .intermediate_sum;
                trace!(
                    "alternatives plan: \n{}",
                    initial_plan
                        .get_desc()
                        .ok_or(Error::MissingDescription)?
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
        node_search_desc: Rc<NodeSearchDesc>,
        desc: Option<&Desc>,
        op_spec: &'a OperatorSpec,
        db: &'a GraphDB,
    ) -> Option<Box<ExecutionNode<Item = Vec<Match>> + 'a>> {
        // check if we can replace this node search with a generic "all nodes from either of these components" search
        let node_search_cost: &CostEstimate = desc?.cost.as_ref()?;

        // get the necessary components and count the number of nodes in these components
        let components = op_spec.necessary_components();
        if components.len() > 0 {

            let mut estimated_component_search = 0;
            
            let mut estimation_valid = false;
            for c in components.iter() {
                if let Some(gs) = db.get_graphstorage(c) {
                    // check if we can apply an even more restrictive edge annotation search
                    if let Some(edge_anno_spec) = op_spec.get_edge_anno_spec() {
                        let anno_storage : &AnnoStorage<Edge> = gs.get_anno_storage();  
                        if let Some(edge_anno_est) = edge_anno_spec.guess_max_count(&anno_storage, &db.strings) {
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
                return Some(Box::new(NodeSearch::new_partofcomponentsearch(
                    db,
                    node_search_desc,
                    desc,
                    components,
                    op_spec.get_edge_anno_spec(),
                )));
            }
        }

        return None;
    }

    fn make_exec_plan_with_order(
        &'a self,
        db: &'a GraphDB,
        operator_order: Vec<usize>,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>, Error> {
        // TODO: parallization mapping

        let mut node2component: BTreeMap<usize, usize> = BTreeMap::new();

        // 1. add all nodes

        // Create a map where the key is the component number
        // and move all nodes with their index as component number.
        let mut component2exec: BTreeMap<usize, Box<ExecutionNode<Item = Vec<Match>> + 'a>> =
            BTreeMap::new();
        let mut node2cost: BTreeMap<usize, CostEstimate> = BTreeMap::new();

        {
            for node_nr in 0..self.nodes.len() {
                let n_spec = &self.nodes[node_nr];
                let mut node_search = NodeSearch::from_spec(n_spec.clone(), node_nr, db).ok_or(
                    Error::ImpossibleSearch(format!(
                        "could not create node search for node {} ({})",
                        node_nr, n_spec
                    )),
                )?;
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

                // move to map
                component2exec.insert(node_nr, Box::new(node_search));
            }
        }

        // 2. add the joins which produce the results in operand order
        for i in operator_order.into_iter() {
            let op_entry: &OperatorEntry<'a> = &self.operators[i];

            let op: Box<Operator> = op_entry.op.create_operator(db).ok_or(
                Error::ImpossibleSearch(format!("could not create operator {:?}", op_entry)),
            )?;

            let (spec_idx_left, spec_idx_right) =
                optimized_operand_order(&op_entry, &op, &node2cost);

            let component_left = node2component
                .get(&spec_idx_left)
                .ok_or(Error::ImpossibleSearch(format!(
                    "no component for node #{}",
                    spec_idx_left + 1
                )))?
                .clone();
            let component_right = node2component
                .get(&spec_idx_right)
                .ok_or(Error::ImpossibleSearch(format!(
                    "no component for node #{}",
                    spec_idx_right + 1
                )))?
                .clone();

            // get the original execution node
            let exec_left: Box<ExecutionNode<Item = Vec<Match>> + 'a> = component2exec
                .remove(&component_left)
                .ok_or(Error::ImpossibleSearch(format!(
                    "no execution node for component {}",
                    component_left
                )))?;

            let opt_exec_left = if let Some(node_search) = exec_left.as_nodesearch() {
                self.optimize_node_search_by_operator(
                    node_search.get_node_search_desc(),
                    exec_left.get_desc(),
                    op_entry.op.as_ref(),
                    db,
                )
            } else {
                None
            };

            let exec_left = if let Some(opt) = opt_exec_left {
                opt
            } else {
                exec_left
            };

            let idx_left = exec_left
                .get_desc()
                .ok_or(Error::MissingDescription)?
                .node_pos
                .get(&spec_idx_left)
                .ok_or(Error::OperatorIdxNotFound)?
                .clone();

            let new_exec: Box<ExecutionNode<Item = Vec<Match>>> =
                if component_left == component_right {
                    // don't create new tuples, only filter the existing ones
                    // TODO: check if LHS or RHS is better suited as filter input iterator
                    let idx_right = exec_left
                        .get_desc()
                        .ok_or(Error::MissingDescription)?
                        .node_pos
                        .get(&spec_idx_right)
                        .ok_or(Error::OperatorIdxNotFound)?
                        .clone();

                    let filter = BinaryFilter::new(
                        exec_left,
                        idx_left,
                        idx_right,
                        spec_idx_left + 1,
                        spec_idx_right + 1,
                        op,
                        db,
                    );
                    Box::new(filter)
                } else {
                    let exec_right = component2exec.remove(&component_right).ok_or(
                        Error::ImpossibleSearch(format!(
                            "no execution node for component {}",
                            component_right
                        )),
                    )?;
                    let idx_right = exec_right
                        .get_desc()
                        .ok_or(Error::MissingDescription)?
                        .node_pos
                        .get(&spec_idx_right)
                        .ok_or(Error::OperatorIdxNotFound)?
                        .clone();

                    if exec_right.as_nodesearch().is_some() {
                        // use index join
                        let join = IndexJoin::new(
                            exec_left,
                            idx_left,
                            spec_idx_left + 1,
                            spec_idx_right + 1,
                            op,
                            exec_right.as_nodesearch().unwrap().get_node_search_desc(),
                            &db,
                            exec_right.get_desc(),
                        );
                        Box::new(join)
                    } else if exec_left.as_nodesearch().is_some() && op.is_commutative() {
                        // avoid a nested loop join by switching the operand and using and index join
                        let join = IndexJoin::new(
                            exec_right,
                            idx_right,
                            spec_idx_right + 1,
                            spec_idx_left + 1,
                            op,
                            exec_left.as_nodesearch().unwrap().get_node_search_desc(),
                            &db,
                            exec_left.get_desc(),
                        );
                        Box::new(join)

                    } else {
                        // use nested loop as "fallback"

                        let join = NestedLoop::new(
                            exec_left,
                            exec_right,
                            idx_left,
                            idx_right,
                            spec_idx_left + 1,
                            spec_idx_right + 1,
                            op,
                            db,
                        );

                        Box::new(join)
                    }
                };

            let new_component_nr = new_exec
                .get_desc()
                .ok_or(Error::ImpossibleSearch(String::from(
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
                    return Err(Error::ComponentsNotConnected);
                }
            }
        }

        let first_component_id = first_component_id.ok_or(Error::ImpossibleSearch(String::from(
            "no component in query at all",
        )))?;
        return component2exec
            .remove(&first_component_id)
            .ok_or(Error::ImpossibleSearch(String::from(
                "could not find execution node for query component",
            )));
    }

    pub fn make_exec_node(
        &'a self,
        db: &'a GraphDB,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>, Error> {
        let operator_order = self.optimize_join_order_heuristics(db)?;
        return self.make_exec_plan_with_order(db, operator_order);
    }
}
