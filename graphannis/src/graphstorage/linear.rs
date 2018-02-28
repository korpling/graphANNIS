use multimap::MultiMap;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::Bound::*;
use std::any::Any;
use std::cmp::Ord;
use std::ops::AddAssign;
use std::clone::Clone;
use std;


use {NodeID, Edge, Annotation, AnnoKey, Match, NumValue};
use super::{GraphStorage, GraphStatistic};
use annostorage::AnnoStorage;
use graphdb::{GraphDB};
use dfs::{CycleSafeDFS, DFSStep};

#[derive(Serialize, Deserialize,Clone)]
struct RelativePosition<PosT> {
    pub root : NodeID,
    pub pos : PosT,
}

#[derive(Serialize, Deserialize,Clone)]
pub struct LinearGraphStorage<PosT : NumValue> {
    node_to_pos : BTreeMap<NodeID,RelativePosition<PosT>>,
    node_chains : BTreeMap<NodeID,Vec<NodeID>>,
    annos: AnnoStorage<Edge>,
    stats : Option<GraphStatistic>,
}



impl<OrderT>  LinearGraphStorage<OrderT> 
where OrderT : NumValue {

    pub fn new() -> LinearGraphStorage<OrderT> {
        LinearGraphStorage {
            node_to_pos : BTreeMap::new(),
            node_chains : BTreeMap::new(),
            annos: AnnoStorage::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) {
        self.node_to_pos.clear();
        self.node_chains.clear();
        self.annos.clear();
        self.stats = None;
    }

}


impl<OrderT: 'static> GraphStorage for  LinearGraphStorage<OrderT> 
where OrderT : NumValue {



    fn get_outgoing_edges<'a>(&'a self, source: &NodeID) -> Box<Iterator<Item = NodeID> + 'a> {
        unimplemented!()
    }

    fn get_edge_annos(&self, edge : &Edge) -> Vec<Annotation> {
        return self.annos.get_all(edge);
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
        if source == target {
            return Some(0);
        }

        unimplemented!()
    }
    fn is_connected(&self, source: &NodeID, target: &NodeID, min_distance: usize, max_distance: usize) -> bool {
        
        unimplemented!()
    }

    fn copy(&mut self, db : &GraphDB, orig : &GraphStorage) {

        self.clear();

        unimplemented!();

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics(&db.strings);
    }


    fn get_anno_storage(&self) -> &AnnoStorage<Edge> {
        &self.annos
    }

    fn as_any(&self) -> &Any {self}

    fn get_statistics(&self) -> Option<&GraphStatistic> {self.stats.as_ref()}

}
