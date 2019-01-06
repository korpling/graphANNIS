use super::RangeSpec;
use crate::annis::operator::UnaryOperatorSpec;
use crate::graph::{ComponentType, Component, Match, NodeID};
use crate::Graph;

use std;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AritySpec {
    pub children: RangeSpec,
}


impl UnaryOperatorSpec for AritySpec {
    fn necessary_components(&self, db: &Graph) -> Vec<Component> {
        let mut result = Vec::default();
        result.extend(db.get_all_components(Some(ComponentType::Dominance), None));
        result.extend(db.get_all_components(Some(ComponentType::Coverage), None));
        result
    }
    fn create_filter(&self, db: &Graph) -> Box<Fn(&Match) -> bool> {
        // collect all relevant graph storages
        let mut graphstorages = Vec::default();

        for component in db.get_all_components(Some(ComponentType::Dominance), None) {
            if let Some(gs) = db.get_graphstorage(&component) {
                graphstorages.push(gs);
            }
        }
        for component in db.get_all_components(Some(ComponentType::Coverage), None) {
            if let Some(gs) = db.get_graphstorage(&component) {
                graphstorages.push(gs);
            }
        }

        let allowed_range = self.children.clone();

        // create the filter which collects all child-nodes in all graphstorages
        let filter = move |m : &Match| -> bool {
            let mut children : FxHashSet<NodeID> = FxHashSet::default();
            for gs in graphstorages.iter() {
                for out in gs.get_outgoing_edges(m.node) {
                    children.insert(out);
                }
            }

            let num_children = children.len();
            if num_children >= allowed_range.min_dist() {
                match allowed_range.max_dist() {
                    std::ops::Bound::Unbounded => true,
                    std::ops::Bound::Included(max_dist) => num_children <= max_dist,
                    std::ops::Bound::Excluded(max_dist) => num_children < max_dist,
                }
            } else {
                false
            }
        };
        Box::new(filter)
    }
}