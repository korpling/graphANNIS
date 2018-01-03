use {Match};
use graphdb::GraphDB;
use operator::{Operator, OperatorSpec};
use exec::{ExecutionNode};
use exec::indexjoin::IndexJoin;
use exec::nestedloop::NestedLoop;
use exec::nodesearch::NodeSearch;
use exec::binary_filter::BinaryFilter;

use super::disjunction::Disjunction;

use std::collections::BTreeMap;

pub enum Error {
    ImpossibleQuery,
    MissingDescription,
}

struct OperatorEntry {
    op : Box<OperatorSpec>,
    idx_left : usize,
    idx_right : usize,
    original_order : usize,
}

pub struct Conjunction<'a> {
    nodes : Vec<NodeSearch<'a>>,
    operators : Vec<OperatorEntry>,
}

impl<'a> Conjunction<'a> {
    pub fn new() -> Conjunction<'a> {
        Conjunction {
            nodes: vec![],
            operators: vec![],
        }
    }

    pub fn into_disjunction(self) -> Disjunction<'a> {
        Disjunction::new(self)
    }

    pub fn add_node(&mut self, node : NodeSearch<'a>) -> usize {
        let idx = self.nodes.len();

        // TODO allow wrapping with an "any node anno" search
        self.nodes.push(node);

        idx
    }

    pub fn add_operator(&mut self, op : Box<OperatorSpec>, idx_left : usize, idx_right : usize) {
        let original_order = self.operators.len();
        self.operators.push(OperatorEntry {
            op,
            idx_left,
            idx_right,
            original_order,
        });

    }

    pub fn make_exec_node(mut self, db : &'a GraphDB) -> Result<Box<ExecutionNode<Item=Vec<Match>>>, Error> {

        let mut node2component : BTreeMap<usize, usize> = BTreeMap::new();
        // TODO: handle cost estimations

        // Create a map where the key is the component number
        // and move all nodes with their index as component number.
        let mut component2exec : BTreeMap<usize, Box<ExecutionNode<Item=Vec<Match>>>> = BTreeMap::new();
        {
            let mut node_nr : usize = 0;
            for n in self.nodes.drain(..) {
                node2component.insert(node_nr, node_nr);
                component2exec.insert(node_nr, Box::new(n));
                node_nr += 1;
            }
        }
         
        // add the joins which produce the results
        for op_entry in self.operators.drain(..) {

            let component_left = node2component.get(&op_entry.idx_left).ok_or(Error::ImpossibleQuery)?.clone();
            let component_right = node2component.get(&op_entry.idx_right).ok_or(Error::ImpossibleQuery)?.clone();

            // TODO: parallization mapping
            let exec_left = component2exec.remove(&component_left).ok_or(Error::ImpossibleQuery)?;
            let exec_right = component2exec.remove(&component_right).ok_or(Error::ImpossibleQuery)?;

            let idx_left = exec_left.get_desc().ok_or(Error::MissingDescription)?
                    .node_pos.get(&op_entry.idx_left).unwrap_or(&0).clone();
            let idx_right = exec_right.get_desc().ok_or(Error::MissingDescription)?
                .node_pos.get(&op_entry.idx_right).unwrap_or(&0).clone();

            let new_exec : Box<ExecutionNode<Item = Vec<Match>>> = if component_left == component_right {
                // don't create new tuples, only filter the existing ones
                // TODO: check if LHS or RHS is better suited as filter input iterator
                let op : Box<Operator> = op_entry.op.create_operator(db).ok_or(Error::ImpossibleQuery)?;
                let filter = BinaryFilter::new(exec_left, idx_left, idx_right, op);
                Box::new(filter)
            } else if exec_right.as_nodesearch().is_some() {
                // TODO: use cost estimation to check if an IndexJoin is actually better

                // use index join
                unimplemented!()

            } else {
                // use nested loop as "fallback"

                // TODO: check if LHS and RHS should be switched
                
                let op : Box<Operator> = op_entry.op.create_operator(db).ok_or(Error::ImpossibleQuery)?;
                let join = NestedLoop::new(exec_left, exec_right, idx_left, idx_right, op);
                
                Box::new(join)
            };
            
        }

        unimplemented!()
    }

}