use super::RangeSpec;
use crate::annis::operator::EstimationType;
use crate::{
    annis::{
        db::aql::model::AnnotationComponentType,
        operator::{UnaryOperator, UnaryOperatorSpec},
    },
    errors::Result,
    graph::{GraphStorage, Match},
    AnnotationGraph,
};
use graphannis_core::types::{Component, NodeID};
use std::collections::HashSet;
use std::sync::Arc;

use rustc_hash::FxHashSet;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AritySpec {
    pub children: RangeSpec,
}

impl UnaryOperatorSpec for AritySpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut result = HashSet::default();
        result.extend(db.get_all_components(Some(AnnotationComponentType::Dominance), None));
        result.extend(db.get_all_components(Some(AnnotationComponentType::Pointing), None));
        result
    }

    fn create_operator(&self, db: &AnnotationGraph) -> Option<Box<dyn UnaryOperator>> {
        // collect all relevant graph storages
        let mut graphstorages = Vec::default();

        for component in db.get_all_components(Some(AnnotationComponentType::Dominance), None) {
            if let Some(gs) = db.get_graphstorage(&component) {
                graphstorages.push(gs);
            }
        }
        for component in db.get_all_components(Some(AnnotationComponentType::Pointing), None) {
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
    graphstorages: Vec<Arc<dyn GraphStorage>>,
    allowed_range: RangeSpec,
}

impl std::fmt::Display for ArityOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ":arity={}", self.allowed_range)
    }
}

impl UnaryOperator for ArityOperator {
    fn filter_match(&self, m: &Match) -> Result<bool> {
        let mut children: FxHashSet<NodeID> = FxHashSet::default();
        for gs in self.graphstorages.iter() {
            for out in gs.get_outgoing_edges(m.node) {
                let out = out?;
                children.insert(out);
            }
        }

        let num_children = children.len();
        if num_children >= self.allowed_range.min_dist() {
            match self.allowed_range.max_dist() {
                std::ops::Bound::Unbounded => Ok(true),
                std::ops::Bound::Included(max_dist) => Ok(num_children <= max_dist),
                std::ops::Bound::Excluded(max_dist) => Ok(num_children < max_dist),
            }
        } else {
            Ok(false)
        }
    }

    fn estimation_type(&self) -> EstimationType {
        if let RangeSpec::Bound { min_dist, max_dist } = self.allowed_range {
            let mut min_matches_any = false;
            let mut max_sel: f64 = 0.0;

            for gs in self.graphstorages.iter() {
                if let Some(stats) = gs.get_statistics() {
                    if min_dist <= stats.max_fan_out {
                        min_matches_any = true;
                    }

                    let max_fan_out = stats.max_fan_out;
                    // clip to asssumed maximum
                    let max_dist = std::cmp::min(max_dist, max_fan_out);
                    let min_dist = std::cmp::min(min_dist, max_dist);

                    // TODO: we would need a histogram of the distribution of outgoing edges
                    // for guessing the the number of matching nodes. For now just assume it is much
                    // more likely to have a larger range instead of a single value
                    let spec_range_len = max_dist - min_dist + 1;
                    let stats_range_len = max_fan_out;
                    let sel = spec_range_len as f64 / (stats_range_len as f64);
                    max_sel = max_sel.max(sel);
                } else {
                    // use default
                    max_sel = max_sel.max(0.1);
                }
            }

            if min_matches_any {
                if max_sel >= 1.0 {
                    EstimationType::Selectivity(1.0)
                } else {
                    EstimationType::Selectivity(max_sel)
                }
            } else {
                // no graph storages has the minimum required amount of outgoing edges
                EstimationType::Selectivity(0.0)
            }
        } else {
            // this range spec allows any number of outgoing edges
            EstimationType::Selectivity(1.0)
        }
    }
}
