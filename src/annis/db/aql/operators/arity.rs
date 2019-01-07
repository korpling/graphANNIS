use super::RangeSpec;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::operator::{UnaryOperator, UnaryOperatorSpec};
use crate::graph::{Component, ComponentType, Match, NodeID};
use crate::Graph;
use std::sync::Arc;

use rustc_hash::FxHashSet;
use std;

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
    fn create_operator(&self, db: &Graph) -> Option<Box<UnaryOperator>> {
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

        Some(Box::new(ArityOperator {
            graphstorages,
            allowed_range: self.children.clone(),
        }))
    }
}

struct ArityOperator {
    graphstorages: Vec<Arc<GraphStorage>>,
    allowed_range: RangeSpec,
}


impl UnaryOperator for ArityOperator {

    fn filter_match(&self, m: &Match) -> bool {
        let mut children: FxHashSet<NodeID> = FxHashSet::default();
            for gs in self.graphstorages.iter() {
                for out in gs.get_outgoing_edges(m.node) {
                    children.insert(out);
                }
            }

            let num_children = children.len();
            if num_children >= self.allowed_range.min_dist() {
                match self.allowed_range.max_dist() {
                    std::ops::Bound::Unbounded => true,
                    std::ops::Bound::Included(max_dist) => num_children <= max_dist,
                    std::ops::Bound::Excluded(max_dist) => num_children < max_dist,
                }
            } else {
                false
            }
    }
}
