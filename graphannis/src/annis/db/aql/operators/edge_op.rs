use crate::annis::db::aql::{model::AnnotationComponentType, operators::RangeSpec};
use crate::annis::errors::GraphAnnisError;
use crate::annis::operator::{
    BinaryOperator, BinaryOperatorBase, BinaryOperatorIndex, BinaryOperatorSpec,
    EdgeAnnoSearchSpec, EstimationType,
};
use crate::errors::Result;
use crate::graph::{GraphStatistic, GraphStorage, Match};
use crate::{try_as_boxed_iter, AnnotationGraph};
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_ANNO_KEY, NODE_TYPE_KEY},
    types::{Component, Edge, NodeID},
};
use itertools::Itertools;
use std::any::Any;
use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct BaseEdgeOpSpec {
    pub components: Vec<Component<AnnotationComponentType>>,
    pub dist: RangeSpec,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
    pub is_reflexive: bool,
    pub op_str: Option<String>,
}

struct BaseEdgeOp {
    gs: Vec<Arc<dyn GraphStorage>>,
    spec: BaseEdgeOpSpec,
    max_nodes_estimate: usize,
    inverse: bool,
}

impl BaseEdgeOp {
    pub fn new(db: &AnnotationGraph, spec: BaseEdgeOpSpec) -> Option<BaseEdgeOp> {
        let mut gs: Vec<Arc<dyn GraphStorage>> = Vec::new();
        for c in &spec.components {
            gs.push(db.get_graphstorage(c)?);
        }
        Some(BaseEdgeOp {
            gs,
            spec,
            max_nodes_estimate: db.get_node_annos().guess_max_count(
                Some(&NODE_TYPE_KEY.ns),
                &NODE_TYPE_KEY.name,
                "node",
                "node",
            ),
            inverse: false,
        })
    }
}

impl BinaryOperatorSpec for BaseEdgeOpSpec {
    fn necessary_components(
        &self,
        _db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::from_iter(self.components.clone())
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let optional_op = BaseEdgeOp::new(db, self.clone());
        optional_op.map(|op| BinaryOperator::Index(Box::new(op)))
    }

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        self.edge_anno.clone()
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

fn check_edge_annotation(
    edge_anno: &Option<EdgeAnnoSearchSpec>,
    gs: &dyn GraphStorage,
    source: NodeID,
    target: NodeID,
) -> bool {
    match edge_anno {
        Some(EdgeAnnoSearchSpec::ExactValue { ns, name, val }) => {
            for a in gs
                .get_anno_storage()
                .get_annotations_for_item(&Edge { source, target })
            {
                if name != &a.key.name {
                    continue;
                }
                if let Some(template_ns) = ns {
                    if template_ns != &a.key.ns {
                        continue;
                    }
                }
                if let Some(template_val) = val {
                    if template_val != &*a.val {
                        continue;
                    }
                }
                // all checks passed, this edge has the correct annotation
                return true;
            }
            false
        }
        Some(EdgeAnnoSearchSpec::NotExactValue { ns, name, val }) => {
            for a in gs
                .get_anno_storage()
                .get_annotations_for_item(&Edge { source, target })
            {
                if name != &a.key.name {
                    continue;
                }
                if let Some(template_ns) = ns {
                    if template_ns != &a.key.ns {
                        continue;
                    }
                }
                if val.as_str() == a.val.as_str() {
                    continue;
                }

                // all checks passed, this edge has the correct annotation
                return true;
            }
            false
        }
        Some(EdgeAnnoSearchSpec::RegexValue { ns, name, val }) => {
            let full_match_pattern = graphannis_core::util::regex_full_match(val);
            let re = regex::Regex::new(&full_match_pattern);
            if let Ok(re) = re {
                for a in gs
                    .get_anno_storage()
                    .get_annotations_for_item(&Edge { source, target })
                {
                    if name != &a.key.name {
                        continue;
                    }
                    if let Some(template_ns) = ns {
                        if template_ns != &a.key.ns {
                            continue;
                        }
                    }

                    if !re.is_match(&*a.val) {
                        continue;
                    }

                    // all checks passed, this edge has the correct annotation
                    return true;
                }
            }
            false
        }
        Some(EdgeAnnoSearchSpec::NotRegexValue { ns, name, val }) => {
            let full_match_pattern = graphannis_core::util::regex_full_match(val);
            let re = regex::Regex::new(&full_match_pattern);
            if let Ok(re) = re {
                for a in gs
                    .get_anno_storage()
                    .get_annotations_for_item(&Edge { source, target })
                {
                    if name != &a.key.name {
                        continue;
                    }
                    if let Some(template_ns) = ns {
                        if template_ns != &a.key.ns {
                            continue;
                        }
                    }

                    if re.is_match(&*a.val) {
                        continue;
                    }

                    // all checks passed, this edge has the correct annotation
                    return true;
                }
            }
            false
        }
        None => true,
    }
}

impl BaseEdgeOp {}

impl std::fmt::Display for BaseEdgeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let anno_frag = if let Some(ref edge_anno) = self.spec.edge_anno {
            format!("[{}]", edge_anno)
        } else {
            String::from("")
        };

        if let Some(ref op_str) = self.spec.op_str {
            if self.inverse {
                write!(f, "{}\u{20D6}{}{}", op_str, self.spec.dist, anno_frag)
            } else {
                write!(f, "{}{}{}", op_str, self.spec.dist, anno_frag)
            }
        } else {
            write!(f, "?")
        }
    }
}

impl BinaryOperatorBase for BaseEdgeOp {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        for e in &self.gs {
            if self.inverse {
                if e.is_connected(
                    rhs.node,
                    lhs.node,
                    self.spec.dist.min_dist(),
                    self.spec.dist.max_dist(),
                )? && check_edge_annotation(&self.spec.edge_anno, e.as_ref(), rhs.node, lhs.node)
                {
                    return Ok(true);
                }
            } else if e.is_connected(
                lhs.node,
                rhs.node,
                self.spec.dist.min_dist(),
                self.spec.dist.max_dist(),
            )? && check_edge_annotation(
                &self.spec.edge_anno,
                e.as_ref(),
                lhs.node,
                rhs.node,
            ) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_reflexive(&self) -> bool {
        self.spec.is_reflexive
    }

    fn get_inverse_operator<'a>(&self, _graph: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        // Check if all graph storages have the same inverse cost.
        // If not, we don't provide an inverse operator, because the plans would not account for the different costs
        for g in &self.gs {
            if !g.inverse_has_same_cost() {
                return None;
            }
            if let Some(stat) = g.get_statistics() {
                // If input and output estimations are too different, also don't provide a more costly inverse operator
                if stat.inverse_fan_out_99_percentile > stat.fan_out_99_percentile {
                    return None;
                }
            }
        }
        let edge_op = BaseEdgeOp {
            gs: self.gs.clone(),
            spec: self.spec.clone(),
            max_nodes_estimate: self.max_nodes_estimate,
            inverse: !self.inverse,
        };
        Some(BinaryOperator::Index(Box::new(edge_op)))
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        if self.gs.is_empty() {
            // will not find anything
            return Ok(EstimationType::Selectivity(0.0));
        }

        let max_nodes: f64 = self.max_nodes_estimate as f64;

        let mut worst_sel: f64 = 0.0;

        for g in &self.gs {
            let g: &Arc<dyn GraphStorage> = g;

            let mut gs_selectivity = 0.01;

            if let Some(stats) = g.get_statistics() {
                let stats: &GraphStatistic = stats;
                if stats.cyclic {
                    // can get all other nodes
                    return Ok(EstimationType::Selectivity(1.0));
                }
                // get number of nodes reachable from min to max distance
                let max_dist = match self.spec.dist.max_dist() {
                    std::ops::Bound::Unbounded => usize::max_value(),
                    std::ops::Bound::Included(max_dist) => max_dist,
                    std::ops::Bound::Excluded(max_dist) => max_dist - 1,
                };
                let max_path_length = std::cmp::min(max_dist, stats.max_depth) as i32;
                let min_path_length = std::cmp::max(0, self.spec.dist.min_dist() - 1) as i32;

                if stats.avg_fan_out > 1.0 {
                    // Assume two complete k-ary trees (with the average fan-out
                    // as k) as defined in "Thomas Cormen: Introduction to
                    // algorithms (2009), page 1179) with the maximum and
                    // minimum height. Calculate the number of nodes for both
                    // complete trees and subtract them to get an estimation of
                    // the number of nodes that fullfull the path length
                    // criteria.
                    let k = stats.avg_fan_out;

                    let reachable_max: f64 = ((k.powi(max_path_length) - 1.0) / (k - 1.0)).ceil();
                    let reachable_min: f64 = ((k.powi(min_path_length) - 1.0) / (k - 1.0)).ceil();

                    let reachable = reachable_max - reachable_min;

                    gs_selectivity = reachable / max_nodes;
                } else {
                    // We can't use the formula for complete k-ary trees because
                    // we can't divide by zero and don't want negative numbers.
                    // Use the simplified estimation with multiplication
                    // instead.
                    let reachable_max: f64 =
                        (stats.avg_fan_out * f64::from(max_path_length)).ceil();
                    let reachable_min: f64 =
                        (stats.avg_fan_out * f64::from(min_path_length)).ceil();

                    gs_selectivity = (reachable_max - reachable_min) / max_nodes;
                }
            }

            if worst_sel < gs_selectivity {
                worst_sel = gs_selectivity;
            }
        } // end for

        Ok(EstimationType::Selectivity(worst_sel))
    }

    fn edge_anno_selectivity(&self) -> Result<Option<f64>> {
        if let Some(ref edge_anno) = self.spec.edge_anno {
            let mut worst_sel = 0.0;
            for g in &self.gs {
                let g: &Arc<dyn GraphStorage> = g;
                let anno_storage = g.get_anno_storage();
                let num_of_annos = anno_storage.number_of_annotations()?;
                if num_of_annos == 0 {
                    // we won't be able to find anything if there are no
                    // annotations
                    return Ok(Some(0.0));
                } else {
                    let guessed_count = match edge_anno {
                        EdgeAnnoSearchSpec::ExactValue { val, ns, name } => {
                            if let Some(val) = val {
                                anno_storage.guess_max_count(
                                    ns.as_ref().map(String::as_str),
                                    name,
                                    val,
                                    val,
                                )
                            } else {
                                anno_storage.number_of_annotations_by_name(
                                    ns.as_ref().map(String::as_str),
                                    name,
                                )?
                            }
                        }
                        EdgeAnnoSearchSpec::NotExactValue { val, ns, name } => {
                            let total = anno_storage.number_of_annotations_by_name(
                                ns.as_ref().map(String::as_str),
                                name,
                            )?;
                            total
                                - anno_storage.guess_max_count(
                                    ns.as_ref().map(String::as_str),
                                    name,
                                    val,
                                    val,
                                )
                        }
                        EdgeAnnoSearchSpec::RegexValue { val, ns, name } => anno_storage
                            .guess_max_count_regex(ns.as_ref().map(String::as_str), name, val),
                        EdgeAnnoSearchSpec::NotRegexValue { val, ns, name } => {
                            let total = anno_storage.number_of_annotations_by_name(
                                ns.as_ref().map(String::as_str),
                                name,
                            )?;
                            total
                                - anno_storage.guess_max_count_regex(
                                    ns.as_ref().map(String::as_str),
                                    name,
                                    val,
                                )
                        }
                    };
                    let g_sel: f64 = (guessed_count as f64) / (num_of_annos as f64);
                    if g_sel > worst_sel {
                        worst_sel = g_sel;
                    }
                }
            }
            Ok(Some(worst_sel))
        } else {
            Ok(Some(1.0))
        }
    }
}

impl BinaryOperatorIndex for BaseEdgeOp {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>>> {
        let lhs = lhs.clone();
        let spec = self.spec.clone();

        if self.gs.len() == 1 {
            // directly return all matched nodes since when having only one component
            // no duplicates are possible
            let result: VecDeque<_> = if self.inverse {
                self.gs[0]
                    .find_connected_inverse(lhs.node, spec.dist.min_dist(), spec.dist.max_dist())
                    .fuse()
                    .filter_ok(move |candidate| {
                        check_edge_annotation(
                            &self.spec.edge_anno,
                            self.gs[0].as_ref(),
                            *candidate,
                            lhs.clone().node,
                        )
                    })
                    .map_ok(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    })
                    .map(|m| m.map_err(GraphAnnisError::from))
                    .collect()
            } else {
                self.gs[0]
                    .find_connected(lhs.node, spec.dist.min_dist(), spec.dist.max_dist())
                    .fuse()
                    .filter_ok(move |candidate| {
                        check_edge_annotation(
                            &self.spec.edge_anno,
                            self.gs[0].as_ref(),
                            lhs.clone().node,
                            *candidate,
                        )
                    })
                    .map_ok(|n| Match {
                        node: n,
                        anno_key: DEFAULT_ANNO_KEY.clone(),
                    })
                    .map(|m| m.map_err(GraphAnnisError::from))
                    .collect()
            };
            Box::new(result.into_iter())
        } else {
            let all: Result<Vec<_>> = if self.inverse {
                self.gs
                    .iter()
                    .flat_map(move |e| {
                        let lhs = lhs.clone();

                        e.as_ref()
                            .find_connected_inverse(
                                lhs.node,
                                spec.dist.min_dist(),
                                spec.dist.max_dist(),
                            )
                            .fuse()
                            .filter_ok(move |candidate| {
                                check_edge_annotation(
                                    &self.spec.edge_anno,
                                    e.as_ref(),
                                    *candidate,
                                    lhs.clone().node,
                                )
                            })
                            .map_ok(|n| Match {
                                node: n,
                                anno_key: DEFAULT_ANNO_KEY.clone(),
                            })
                    })
                    .map(|m| m.map_err(GraphAnnisError::from))
                    .collect()
            } else {
                self.gs
                    .iter()
                    .flat_map(move |e| {
                        let lhs = lhs.clone();

                        e.as_ref()
                            .find_connected(lhs.node, spec.dist.min_dist(), spec.dist.max_dist())
                            .fuse()
                            .filter_ok(move |candidate| {
                                check_edge_annotation(
                                    &self.spec.edge_anno,
                                    e.as_ref(),
                                    lhs.clone().node,
                                    *candidate,
                                )
                            })
                            .map_ok(|n| Match {
                                node: n,
                                anno_key: DEFAULT_ANNO_KEY.clone(),
                            })
                    })
                    .map(|m| m.map_err(GraphAnnisError::from))
                    .collect()
            };
            let mut all = try_as_boxed_iter!(all);
            all.sort_unstable();
            all.dedup();
            Box::new(all.into_iter().map(Ok))
        }
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DominanceSpec {
    pub name: String,
    pub dist: RangeSpec,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
}

impl BinaryOperatorSpec for DominanceSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::from_iter(
            db.get_all_components(Some(AnnotationComponentType::Dominance), Some(&self.name)),
        )
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let components =
            db.get_all_components(Some(AnnotationComponentType::Dominance), Some(&self.name));
        let op_str = if self.name.is_empty() {
            String::from(">")
        } else {
            format!(">{} ", &self.name)
        };
        let base = BaseEdgeOpSpec {
            op_str: Some(op_str),
            components,
            dist: self.dist.clone(),
            edge_anno: self.edge_anno.clone(),
            is_reflexive: true,
        };
        base.create_operator(db)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointingSpec {
    pub name: String,
    pub dist: RangeSpec,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
}

impl BinaryOperatorSpec for PointingSpec {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::from_iter(
            db.get_all_components(Some(AnnotationComponentType::Pointing), Some(&self.name)),
        )
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let components =
            db.get_all_components(Some(AnnotationComponentType::Pointing), Some(&self.name));
        let op_str = if self.name.is_empty() {
            String::from("->")
        } else {
            format!("->{} ", self.name)
        };

        let base = BaseEdgeOpSpec {
            components,
            dist: self.dist.clone(),
            edge_anno: self.edge_anno.clone(),
            is_reflexive: true,
            op_str: Some(op_str),
        };
        base.create_operator(db)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartOfSubCorpusSpec {
    pub dist: RangeSpec,
}

impl BinaryOperatorSpec for PartOfSubCorpusSpec {
    fn necessary_components(
        &self,
        _db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        let mut components = HashSet::default();
        components.insert(Component::new(
            AnnotationComponentType::PartOf,
            ANNIS_NS.into(),
            "".into(),
        ));
        components
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        let components = vec![Component::new(
            AnnotationComponentType::PartOf,
            ANNIS_NS.into(),
            "".into(),
        )];
        let base = BaseEdgeOpSpec {
            op_str: Some(String::from("@")),
            components,
            dist: self.dist.clone(),
            edge_anno: None,
            is_reflexive: false,
        };

        base.create_operator(db)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}
