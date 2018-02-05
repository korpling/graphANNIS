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

#[derive(Debug)]
pub enum Error {
    ImpossibleSearch,
    MissingDescription,
    ComponentsNotConnected,
    OperatorIdxNotFound,
}

struct OperatorEntry<'a> {
    op: Box<OperatorSpec + 'a>,
    idx_left: usize,
    idx_right: usize,
    /*    original_order: usize, */
}

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

    pub fn make_exec_node(
        mut self,
        db: &'a GraphDB,
    ) -> Result<Box<ExecutionNode<Item = Vec<Match>> + 'a>, Error> {
        // TODO: handle cost estimations
        // TODO: parallization mapping

        let mut node2component: BTreeMap<usize, usize> = BTreeMap::new();

        // 1. add all nodes

        // Create a map where the key is the component number
        // and move all nodes with their index as component number.
        let mut component2exec: BTreeMap<usize, Box<ExecutionNode<Item = Vec<Match>>>> =
            BTreeMap::new();
        {
            let mut node_nr: usize = 0;
            for n_spec in self.nodes.drain(..) {
                let mut n =
                    NodeSearch::from_spec(n_spec, node_nr, db).ok_or(Error::ImpossibleSearch)?;
                node2component.insert(node_nr, node_nr);

                let (orig_query_frag, orig_impl_desc) = if let Some(d) = n.get_desc() {
                    (d.query_fragment.clone(), d.impl_description.clone())
                } else {
                    (String::from(""), String::from(""))
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
                };
                n.set_desc(Some(new_desc));

                // move to map
                component2exec.insert(node_nr, Box::new(n));
                node_nr += 1;
            }
        }

        // 2. add the joins which produce the results
        for op_entry in self.operators.drain(..) {
            let component_left = node2component
                .get(&op_entry.idx_left)
                .ok_or(Error::ImpossibleSearch)?
                .clone();
            let component_right = node2component
                .get(&op_entry.idx_right)
                .ok_or(Error::ImpossibleSearch)?
                .clone();

            let exec_left = component2exec
                .remove(&component_left)
                .ok_or(Error::ImpossibleSearch)?;
            let exec_right = component2exec
                .remove(&component_right)
                .ok_or(Error::ImpossibleSearch)?;

            let idx_left = exec_left
                .get_desc()
                .ok_or(Error::MissingDescription)?
                .node_pos
                .get(&op_entry.idx_left)
                .ok_or(Error::OperatorIdxNotFound)?
                .clone();
            let idx_right = exec_right
                .get_desc()
                .ok_or(Error::MissingDescription)?
                .node_pos
                .get(&op_entry.idx_right)
                .ok_or(Error::OperatorIdxNotFound)?
                .clone();

            let op: Box<Operator> = op_entry
                .op
                .create_operator(db)
                .ok_or(Error::ImpossibleSearch)?;

            let new_exec: Box<ExecutionNode<Item = Vec<Match>>> =
                if component_left == component_right {
                    // don't create new tuples, only filter the existing ones
                    // TODO: check if LHS or RHS is better suited as filter input iterator

                    let filter = BinaryFilter::new(exec_left, idx_left, idx_right, op);
                    Box::new(filter)
                } else if exec_right.as_nodesearch().is_some() {
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
                    );

                    Box::new(join)
                };

            let new_component_nr = new_exec
                .get_desc()
                .ok_or(Error::ImpossibleSearch)?
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

        let first_component_id = first_component_id.ok_or(Error::ImpossibleSearch)?;
        return component2exec
            .remove(&first_component_id)
            .ok_or(Error::ImpossibleSearch);
    }
}
