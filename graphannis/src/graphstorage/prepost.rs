use multimap::MultiMap;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::any::Any;
use std::cmp::Ord;
use std::ops::AddAssign;
use std::clone::Clone;
use std::convert::From;
use std;

use num::{Num,FromPrimitive};

use {NodeID, Edge, Annotation, AnnoKey, Match};
use super::{GraphStorage, GraphStatistic};
use annostorage::AnnoStorage;
use graphdb::{GraphDB};
use dfs::{CycleSafeDFS, DFSStep};

#[derive(PartialOrd, PartialEq, Ord,Eq,Clone)]
pub struct PrePost<OrderT,LevelT> {
    pub pre : OrderT,
    pub post : OrderT,
    pub level : LevelT,
}

pub struct PrePostOrderStorage<OrderT, LevelT> {
    //type PrePostSpec = PrePost<OrderT, LevelT>;

    node_to_order : MultiMap<NodeID, PrePost<OrderT,LevelT>>,
    order_to_node : BTreeMap<PrePost<OrderT,LevelT>,NodeID>,
    annos: AnnoStorage<Edge>,
    stats : Option<GraphStatistic>,
}

struct NodeStackEntry<OrderT, LevelT>
{
  pub id : NodeID,
  pub order : PrePost<OrderT,LevelT>,
}


impl<OrderT, LevelT>  PrePostOrderStorage<OrderT,LevelT> 
where OrderT : Num + Ord + AddAssign + Clone, 
    LevelT : Num + Ord + Clone + From<usize> {

    pub fn new() -> PrePostOrderStorage<OrderT, LevelT> {
        PrePostOrderStorage {
            node_to_order: MultiMap::new(),
            order_to_node: BTreeMap::new(),
            annos: AnnoStorage::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) {
        self.node_to_order.clear();
        self.order_to_node.clear();
    }


    fn enter_node(current_order : &mut OrderT, node_id : &NodeID, level : LevelT, node_stack : &mut NStack<OrderT,LevelT>) {
        let new_entry = NodeStackEntry {
            id: node_id.clone(),
            order : PrePost {
                pre: current_order.clone(),
                level: level,
                post : OrderT::zero(),
            },
        };
        current_order.add_assign(OrderT::one());
        node_stack.push_front(new_entry);
    }

    fn exit_node(&mut self, current_order : &mut OrderT, node_stack : &mut NStack<OrderT,LevelT>) {
         // find the correct pre/post entry and update the post-value
        if let Some(entry) = node_stack.front_mut() {
            entry.order.post = current_order.clone();
            current_order.add_assign(OrderT::one());

            self.node_to_order.insert(entry.id, entry.order.clone());
            self.order_to_node.insert(entry.order.clone(), entry.id);

        }
        node_stack.pop_front();
    }
}

type NStack<OrderT,LevelT> = std::collections::LinkedList<NodeStackEntry<OrderT,LevelT>>;


impl<OrderT: 'static, LevelT : 'static> GraphStorage for  PrePostOrderStorage<OrderT,LevelT> 
where OrderT : Send + Sync + Ord + Num + AddAssign + Clone, 
    LevelT : Send + Sync + Ord + Num + Clone + FromPrimitive + From<usize> {



    fn get_outgoing_edges<'a>(&'a self, source: &NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        unimplemented!()
    }

    fn get_edge_annos(&self, edge : &Edge) -> Vec<Annotation> {
        unimplemented!()
    }
    
    fn find_connected<'a>(
        &'a self,
        source: &NodeID,
        min_distance: usize,
        max_distance: usize,
    ) -> Box<Iterator<Item = NodeID> + 'a> {
        unimplemented!()
    }

    fn distance(&self, source: &NodeID, target: &NodeID) -> Option<usize> {
        unimplemented!()
    }
    fn is_connected(&self, source: &NodeID, target: &NodeID, min_distance: usize, max_distance: usize) -> bool {
        unimplemented!()
    }

    fn copy(&mut self, db : &GraphDB, orig : &GraphStorage) {

        self.clear();

        // find all roots of the component
        let mut roots : HashSet<NodeID> = HashSet::new();
        let node_name_key : AnnoKey = db.get_node_name_key();
        let nodes : Box<Iterator<Item = Match>> = 
            db.node_annos.exact_anno_search(Some(node_name_key.ns), node_name_key.name, None);

        // first add all nodes that are a source of an edge as possible roots
        for m in nodes {
            let m : Match = m;
            let n = m.node;
            // insert all nodes to the root candidate list which are part of this component
            if orig.get_outgoing_edges(&n).next().is_some() {
                roots.insert(n);
            }
        }

        let nodes : Box<Iterator<Item = Match>> = 
            db.node_annos.exact_anno_search(Some(node_name_key.ns), node_name_key.name, None);
        for m in nodes {
            let m : Match = m;

            let source = m.node;

            let out_edges = orig.get_outgoing_edges(&source);
            for target in out_edges {
                // remove the nodes that have an incoming edge from the root list
                roots.remove(&target);

                // add the edge annotations for this edge
                let e = Edge {source, target};
                let edge_annos = orig.get_edge_annos(&e);
                for a in edge_annos.into_iter() {
                    self.annos.insert(e.clone(), a);
                }
            }
        }

        let mut current_order = OrderT::zero();
        // traverse the graph for each sub-component
        for start_node in roots.iter() {
            let mut last_distance : usize = 0;

            let mut node_stack : NStack<OrderT,LevelT> = NStack::new();

            PrePostOrderStorage::enter_node(&mut current_order, start_node, LevelT::zero(), &mut node_stack);

            let dfs = CycleSafeDFS::new(orig, start_node, 1, usize::max_value());
            for step in dfs {
                let step : DFSStep = step;
                if step.distance > last_distance {
                    // first visited, set pre-order
                    if let Some(dist) = LevelT::from_usize(step.distance) {
                        PrePostOrderStorage::enter_node(&mut current_order, start_node, dist, &mut node_stack);
                    }
                } else {
                    // Neighbour node, the last subtree was iterated completly, thus the last node
                    // can be assigned a post-order.
                    // The parent node must be at the top of the node stack,
                    // thus exit every node which comes after the parent node.
                    // Distance starts with 0 but the stack size starts with 1.
                    while node_stack.len() > step.distance {
                        self.exit_node(&mut current_order, &mut node_stack);
                    }
                    // new node
                    if let Some(dist) = LevelT::from_usize(step.distance) {
                        PrePostOrderStorage::enter_node(&mut current_order, &step.node, dist, &mut node_stack);
                    }
                }
                last_distance = step.distance;
            } // end for each DFS step

            while !node_stack.is_empty() {
                self.exit_node(&mut current_order,&mut node_stack);
            }
        } // end for each root

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics(&db.strings);
    }


    fn get_anno_storage(&self) -> &AnnoStorage<Edge> {
        unimplemented!()
    }

    fn as_any(&self) -> &Any {self}

}
