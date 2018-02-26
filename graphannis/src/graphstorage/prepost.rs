use multimap::MultiMap;
use std::collections::BTreeMap;
use std::any::Any;
use std;

use {NodeID, Edge, Annotation};
use super::GraphStorage;
use annostorage::AnnoStorage;


#[derive(PartialOrd, PartialEq, Ord,Eq)]
pub struct PrePost<OrderT,LevelT> {
    pub pre : OrderT,
    pub post : OrderT,
    pub level : LevelT,
}

pub struct PrePostOrderStorage<OrderT, LevelT> {
    //type PrePostSpec = PrePost<OrderT, LevelT>;

    node_to_order : MultiMap<NodeID, PrePost<OrderT,LevelT>>,
    order_to_node : BTreeMap<PrePost<OrderT,LevelT>,NodeID>,
}

impl<OrderT, LevelT>  PrePostOrderStorage<OrderT,LevelT> 
where OrderT : std::cmp::Ord, 
    LevelT : std::cmp::Ord {

    pub fn new() -> PrePostOrderStorage<OrderT, LevelT> {
        PrePostOrderStorage {
            node_to_order: MultiMap::new(),
            order_to_node: BTreeMap::new(),
        }
    }
}

impl<OrderT: 'static, LevelT : 'static> GraphStorage for  PrePostOrderStorage<OrderT,LevelT> 
where OrderT : Send + Sync, 
    LevelT : Send + Sync {

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

    fn copy(&mut self, orig : &GraphStorage) {
        unimplemented!()
    }

    fn get_anno_storage(&self) -> &AnnoStorage<Edge> {
        unimplemented!()
    }

    fn as_any(&self) -> &Any {self}

}
