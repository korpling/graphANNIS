use super::RangeSpec;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::operator::EstimationType;
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
        result.extend(db.get_all_components(Some(ComponentType::Pointing), None));
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
        for component in db.get_all_components(Some(ComponentType::Pointing), None) {
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

impl std::fmt::Display for ArityOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ":arity={}", self.allowed_range)
    }
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

    fn estimation_type(&self) -> EstimationType {
        if let RangeSpec::Bound { min_dist, max_dist } = self.allowed_range {
            let mut min_matches_any = false;
            let mut gs_with_stats = 0;
            let mut sum_max_fan_out = 0;
            for gs in self.graphstorages.iter() {
                if let Some(stats) = gs.get_statistics() {
                    gs_with_stats += 1;
                    sum_max_fan_out += stats.fan_out_99_percentile;
                    if min_dist <= stats.max_fan_out {
                        min_matches_any = true;
                    }
                }
            }
            
            if min_matches_any {
                let max_fan_out = (sum_max_fan_out as f64 / gs_with_stats as f64).round() as usize;

                // clip to asssumed maximum
                let max_dist = std::cmp::min(max_dist, max_fan_out);

                // TODO: we would need a histogram of the distribution of outgoing edges 
                // for guessing the the number of matching nodes. For now just assume it is much 
                // more likely to have a larger range instead of a single value
                let spec_range_len = max_dist - min_dist + 1;
                let stats_range_len = max_fan_out;

                let sel = spec_range_len as f64 / (stats_range_len as f64);
                if sel >= 1.0 {
                    EstimationType::SELECTIVITY(1.0)
                } else {
                    EstimationType::SELECTIVITY(sel)
                }
             } else {
                // no graph storages has the minimum required amount of outgoing edges
                EstimationType::SELECTIVITY(0.0)
            }
        } else {
            // this range spec allows any number of outgoing edges
            EstimationType::SELECTIVITY(1.0)
        }
    }
}
