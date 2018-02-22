use {Component, Match};
use graphdb::GraphDB;
use operator::{Operator, OperatorSpec};
use exec::{Desc, ExecutionNode};
use exec::indexjoin::IndexJoin;
use exec::nestedloop::NestedLoop;
use exec::nodesearch::{NodeSearch, NodeSearchSpec};
use exec::binary_filter::BinaryFilter;

use super::disjunction::Disjunction;

use std::collections::BTreeMap;
use std::iter::FromIterator;

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
        let mut rng = XorShiftRng::from_seed([4711,1,2,3]);
        let mut dist = Range::new(0, self.operators.len());

        let mut best_operator_order = Vec::from_iter(0..self.operators.len());

        // TODO: cache the base estimates
        let initial_plan = self.make_exec_plan_with_order(db, best_operator_order.clone())?;
        let mut best_cost = initial_plan.get_desc().ok_or(Error::MissingDescription)?.cost.clone().ok_or(Error::MissingCost)?.intermediate_sum.clone();

        let num_new_generations = 4;
        let max_unsuccessful_tries = 5*self.operators.len();
        let mut unsucessful = 0;
        while unsucessful < max_unsuccessful_tries {

            let mut family_operators : Vec<Vec<usize>> = Vec::new();
            family_operators.reserve(num_new_generations+1);

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
                tmp_operators.swap(a,b);
                family_operators.push(tmp_operators);
            }

            let mut found_better_plan = false;
            for i in 1..family_operators.len() {
                let alt_plan = self.make_exec_plan_with_order(db, family_operators[i].clone())?;
                let alt_cost = alt_plan.get_desc().ok_or(Error::MissingDescription)?.cost.clone().ok_or(Error::MissingCost)?.intermediate_sum;

                if alt_cost < best_cost {
                    best_operator_order = family_operators[i].clone();
                    found_better_plan = true;

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

    fn make_exec_plan_with_order(&self, db: &'a GraphDB, operator_order : Vec<usize>) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>, Error>  {
        // TODO: parallization mapping

        let mut node2component: BTreeMap<usize, usize> = BTreeMap::new();

        // 1. add all nodes

        // Create a map where the key is the component number
        // and move all nodes with their index as component number.
        let mut component2exec: BTreeMap<usize, Box<ExecutionNode<Item = Vec<Match>>>> =
            BTreeMap::new();
        {
            let mut node_nr: usize = 0;
            for n_spec in self.nodes.iter() {
                let mut n = NodeSearch::from_spec(n_spec.clone(), node_nr, db).ok_or(
                    Error::ImpossibleSearch(format!(
                        "could not create node search for node {} ({})",
                        node_nr, n_spec
                    )),
                )?;
                node2component.insert(node_nr, node_nr);

                let (orig_query_frag, orig_impl_desc, cost) = if let Some(d) = n.get_desc() {
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
                n.set_desc(Some(new_desc));

                // move to map
                component2exec.insert(node_nr, Box::new(n));
                node_nr += 1;
            }
        }

        // 2. add the joins which produce the results in operand order
        for i in operator_order.into_iter() {
            let op_entry = &self.operators[i];
        
            let component_left = node2component
                .get(&op_entry.idx_left)
                .ok_or(Error::ImpossibleSearch(format!(
                    "no component for node #{}",
                    op_entry.idx_left + 1
                )))?
                .clone();
            let component_right = node2component
                .get(&op_entry.idx_right)
                .ok_or(Error::ImpossibleSearch(format!(
                    "no component for node #{}",
                    op_entry.idx_right + 1
                )))?
                .clone();

            let exec_left = component2exec.remove(&component_left).ok_or(
                Error::ImpossibleSearch(format!(
                    "no execution node for component {}",
                    component_left
                )),
            )?;
            let idx_left = exec_left
                .get_desc()
                .ok_or(Error::MissingDescription)?
                .node_pos
                .get(&op_entry.idx_left)
                .ok_or(Error::OperatorIdxNotFound)?
                .clone();

            let op: Box<Operator> = op_entry.op.create_operator(db).ok_or(
                Error::ImpossibleSearch(format!("could not create operator {:?}", op_entry)),
            )?;

            let new_exec: Box<ExecutionNode<Item = Vec<Match>>> =
                if component_left == component_right {
                    // don't create new tuples, only filter the existing ones
                    // TODO: check if LHS or RHS is better suited as filter input iterator
                    let idx_right = exec_left
                        .get_desc()
                        .ok_or(Error::MissingDescription)?
                        .node_pos
                        .get(&op_entry.idx_right)
                        .ok_or(Error::OperatorIdxNotFound)?
                        .clone();

                    let filter = BinaryFilter::new(
                        exec_left,
                        idx_left,
                        idx_right,
                        op_entry.idx_left + 1,
                        op_entry.idx_right + 1,
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
                        .get(&op_entry.idx_right)
                        .ok_or(Error::OperatorIdxNotFound)?
                        .clone();

                    if exec_right.as_nodesearch().is_some() {
                        // TODO: use cost estimation to check if an IndexJoin is really better

                        // use index join
                        let join = IndexJoin::new(
                            exec_left,
                            idx_left,
                            op_entry.idx_left + 1,
                            op_entry.idx_right + 1,
                            op,
                            exec_right.as_nodesearch().unwrap().get_node_search_desc(),
                            &db,
                            exec_right.get_desc(),
                        );
                        Box::new(join)
                    } else {
                        // use nested loop as "fallback"

                        // TODO: check if LHS and RHS should be switched

                        let join = NestedLoop::new(
                            exec_left,
                            exec_right,
                            idx_left,
                            idx_right,
                            op_entry.idx_left + 1,
                            op_entry.idx_right + 1,
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
        self,
        db: &'a GraphDB,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>, Error> {
        let operator_order = self.optimize_join_order_heuristics(db)?;
        return self.make_exec_plan_with_order(db, operator_order);
    }
}
