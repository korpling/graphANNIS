use annis::annostorage::AnnoStorage;
use annis::db::graphstorage::{GraphStatistic, GraphStorage};
use annis::db::AnnotationStorage;
use annis::db::{Graph, ANNIS_NS};
use annis::operator::{EdgeAnnoSearchSpec, EstimationType, Operator, OperatorSpec};
use annis::types::{AnnoKey, AnnoKeyID, Component, ComponentType, Edge, Match, NodeID};
use std;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct BaseEdgeOpSpec {
    pub components: Vec<Component>,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
    pub is_reflexive: bool,
    pub op_str: Option<String>,
}

struct BaseEdgeOp {
    gs: Vec<Arc<GraphStorage>>,
    spec: BaseEdgeOpSpec,
    node_annos: Arc<AnnoStorage<NodeID>>,
    node_type_key: AnnoKey,
    inverse: bool,
}

impl BaseEdgeOp {
    pub fn new(db: &Graph, spec: BaseEdgeOpSpec) -> Option<BaseEdgeOp> {
        let mut gs: Vec<Arc<GraphStorage>> = Vec::new();
        for c in spec.components.iter() {
            gs.push(db.get_graphstorage(c)?);
        }
        Some(BaseEdgeOp {
            gs,
            spec,
            node_annos: db.node_annos.clone(),
            node_type_key: db.get_node_type_key(),
            inverse: false,
        })
    }
}

impl OperatorSpec for BaseEdgeOpSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        self.components.clone()
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<Operator>> {
        let optional_op = BaseEdgeOp::new(db, self.clone());
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        self.edge_anno.clone()
    }
}

fn check_edge_annotation(
    edge_anno: &Option<EdgeAnnoSearchSpec>,
    gs: &GraphStorage,
    source: &NodeID,
    target: &NodeID,
) -> bool {
    match edge_anno {
        Some(EdgeAnnoSearchSpec::ExactValue { ns, name, val }) => {
            for a in gs
                .get_edge_annos(&Edge {
                    source: source.clone(),
                    target: target.clone(),
                }).into_iter()
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
            return false;
        }
        None => {
            return true;
        }
    };
}

impl BaseEdgeOp {}

impl std::fmt::Display for BaseEdgeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let anno_frag = if let Some(ref edge_anno) = self.spec.edge_anno {
            format!("[{}]", edge_anno)
        } else {
            String::from("")
        };

        let range_frag = super::format_range(self.spec.min_dist, self.spec.max_dist);

        if let Some(ref op_str) = self.spec.op_str {
            if self.inverse {
                write!(f, "{}\u{20D6}{}{}", op_str, range_frag, anno_frag)
            } else {
                write!(f, "{}{}{}", op_str, range_frag, anno_frag)
            }
        } else {
            write!(f, "?")
        }
    }
}

impl Operator for BaseEdgeOp {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let lhs = lhs.clone();
        let spec = self.spec.clone();

        if self.gs.len() == 1 {
            // directly return all matched nodes since when having only one component
            // no duplicates are possible
            let result: VecDeque<Match> = if self.inverse {
                self.gs[0]
                    .find_connected_inverse(&lhs.node, spec.min_dist, spec.max_dist)
                    .fuse()
                    .filter(move |candidate| {
                        check_edge_annotation(
                            &self.spec.edge_anno,
                            self.gs[0].as_ref(),
                            candidate,
                            &lhs.clone().node,
                        )
                    }).map(|n| Match {
                        node: n,
                        anno_key: AnnoKeyID::default(),
                    }).collect()
            } else {
                self.gs[0]
                    .find_connected(&lhs.node, spec.min_dist, spec.max_dist)
                    .fuse()
                    .filter(move |candidate| {
                        check_edge_annotation(
                            &self.spec.edge_anno,
                            self.gs[0].as_ref(),
                            &lhs.clone().node,
                            candidate,
                        )
                    }).map(|n| Match {
                        node: n,
                        anno_key: AnnoKeyID::default(),
                    }).collect()
            };
            return Box::new(result.into_iter());
        } else {
            let mut all: Vec<Match> = if self.inverse {
                self.gs
                    .iter()
                    .flat_map(move |e| {
                        let lhs = lhs.clone();

                        e.as_ref()
                            .find_connected_inverse(&lhs.node, spec.min_dist, spec.max_dist)
                            .fuse()
                            .filter(move |candidate| {
                                check_edge_annotation(
                                    &self.spec.edge_anno,
                                    e.as_ref(),
                                    candidate,
                                    &lhs.clone().node,
                                )
                            }).map(|n| Match {
                                node: n,
                                anno_key: AnnoKeyID::default(),
                            })
                    }).collect()
            } else {
                self.gs
                    .iter()
                    .flat_map(move |e| {
                        let lhs = lhs.clone();

                        e.as_ref()
                            .find_connected(&lhs.node, spec.min_dist, spec.max_dist)
                            .fuse()
                            .filter(move |candidate| {
                                check_edge_annotation(
                                    &self.spec.edge_anno,
                                    e.as_ref(),
                                    &lhs.clone().node,
                                    candidate,
                                )
                            }).map(|n| Match {
                                node: n,
                                anno_key: AnnoKeyID::default(),
                            })
                    }).collect()
            };
            all.sort_unstable();
            all.dedup();
            return Box::new(all.into_iter());
        }
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        for e in self.gs.iter() {
            if self.inverse {
                if e.is_connected(&rhs.node, &lhs.node, self.spec.min_dist, self.spec.max_dist) {
                    if check_edge_annotation(&self.spec.edge_anno, e.as_ref(), &rhs.node, &lhs.node)
                    {
                        return true;
                    }
                }
            } else {
                if e.is_connected(&lhs.node, &rhs.node, self.spec.min_dist, self.spec.max_dist) {
                    if check_edge_annotation(&self.spec.edge_anno, e.as_ref(), &lhs.node, &rhs.node)
                    {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    fn is_reflexive(&self) -> bool {
        self.spec.is_reflexive
    }

    fn get_inverse_operator(&self) -> Option<Box<Operator>> {
        // Check if all graph storages have the same inverse cost.
        // If not, we don't provide an inverse operator, because the plans would not account for the different costs
        for g in self.gs.iter() {
            if !g.inverse_has_same_cost() {
                return None;
            }
        }
        let edge_op = BaseEdgeOp {
            gs: self.gs.clone(),
            spec: self.spec.clone(),
            node_annos: self.node_annos.clone(),
            node_type_key: self.node_type_key.clone(),
            inverse: !self.inverse,
        };
        Some(Box::new(edge_op))
    }

    fn estimation_type(&self) -> EstimationType {
        if self.gs.is_empty() {
            // will not find anything
            return EstimationType::SELECTIVITY(0.0);
        }

        let max_nodes: f64 = self.node_annos.guess_max_count(
            Some(self.node_type_key.ns.clone()),
            self.node_type_key.name.clone(),
            "node",
            "node",
        ) as f64;

        let mut worst_sel: f64 = 0.0;

        for g in self.gs.iter() {
            let g: &Arc<GraphStorage> = g;

            let mut gs_selectivity = 0.01;

            if let Some(stats) = g.get_statistics() {
                let stats: &GraphStatistic = stats;
                if stats.cyclic {
                    // can get all other nodes
                    return EstimationType::SELECTIVITY(1.0);
                }
                // get number of nodes reachable from min to max distance
                let max_path_length = std::cmp::min(self.spec.max_dist, stats.max_depth) as i32;
                let min_path_length = std::cmp::max(0, self.spec.min_dist - 1) as i32;

                if stats.avg_fan_out > 1.0 {
                    // Assume two complete k-ary trees (with the average fan-out as k)
                    // as defined in "Thomas Cormen: Introduction to algorithms (2009), page 1179)
                    // with the maximum and minimum height. Calculate the number of nodes for both complete trees and
                    // subtract them to get an estimation of the number of nodes that fullfull the path length criteria.
                    let k = stats.avg_fan_out;

                    let reachable_max: f64 = ((k.powi(max_path_length) - 1.0) / (k - 1.0)).ceil();
                    let reachable_min: f64 = ((k.powi(min_path_length) - 1.0) / (k - 1.0)).ceil();

                    let reachable = reachable_max - reachable_min;

                    gs_selectivity = reachable / max_nodes;
                } else {
                    // We can't use the formula for complete k-ary trees because we can't divide by zero and don't want negative
                    // numbers. Use the simplified estimation with multiplication instead.
                    let reachable_max: f64 = (stats.avg_fan_out * (max_path_length as f64)).ceil();
                    let reachable_min: f64 = (stats.avg_fan_out * (min_path_length as f64)).ceil();

                    gs_selectivity = (reachable_max - reachable_min) / max_nodes;
                }
            }

            if worst_sel < gs_selectivity {
                worst_sel = gs_selectivity;
            }
        } // end for

        return EstimationType::SELECTIVITY(worst_sel);
    }

    fn edge_anno_selectivity(&self) -> Option<f64> {
        if let Some(ref edge_anno) = self.spec.edge_anno {
            let mut worst_sel = 0.0;
            for g in self.gs.iter() {
                let g: &Arc<GraphStorage> = g;
                let anno_storage = g.get_anno_storage();
                let num_of_annos = anno_storage.number_of_annotations();
                if num_of_annos == 0 {
                    // we won't be able to find anything if there are no annotations
                    return Some(0.0);
                } else {
                    let EdgeAnnoSearchSpec::ExactValue { val, ns, name } = edge_anno;

                    let guessed_count = if let Some(val) = val {
                        anno_storage.guess_max_count(ns.clone(), name.clone(), val, val)
                    } else {
                        anno_storage.num_of_annotations(ns.clone(), name.clone())
                    };

                    let g_sel: f64 = (guessed_count as f64) / (num_of_annos as f64);
                    if g_sel > worst_sel {
                        worst_sel = g_sel;
                    }
                }
            }
            return Some(worst_sel);
        } else {
            return Some(1.0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct DominanceSpec {
    pub name: String,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
}

impl OperatorSpec for DominanceSpec {
    fn necessary_components(&self, db: &Graph) -> Vec<Component> {
        db.get_all_components(Some(ComponentType::Dominance), Some(&self.name))
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<Operator>> {
        let components = db.get_all_components(Some(ComponentType::Dominance), Some(&self.name));
        let op_str = if self.name.is_empty() {
            String::from(">")
        } else {
            format!(">{} ", &self.name)
        };
        let base = BaseEdgeOpSpec {
            op_str: Some(op_str),
            components,
            min_dist: self.min_dist,
            max_dist: self.max_dist,
            edge_anno: self.edge_anno.clone(),
            is_reflexive: true,
        };
        base.create_operator(db)
    }
}

#[derive(Debug, Clone)]
pub struct PointingSpec {
    pub name: String,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
}

impl OperatorSpec for PointingSpec {
    fn necessary_components(&self, db: &Graph) -> Vec<Component> {
        db.get_all_components(Some(ComponentType::Pointing), Some(&self.name))
    }

    fn create_operator<'b>(&self, db: &Graph) -> Option<Box<Operator>> {
        let components = db.get_all_components(Some(ComponentType::Pointing), Some(&self.name));
        let op_str = if self.name.is_empty() {
            String::from("->")
        } else {
            format!("->{} ", self.name)
        };

        let base = BaseEdgeOpSpec {
            components,
            min_dist: self.min_dist,
            max_dist: self.max_dist,
            edge_anno: self.edge_anno.clone(),
            is_reflexive: true,
            op_str: Some(op_str),
        };
        base.create_operator(db)
    }
}

#[derive(Debug, Clone)]
pub struct PartOfSubCorpusSpec {
    pub min_dist: usize,
    pub max_dist: usize,
}

impl OperatorSpec for PartOfSubCorpusSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        let components = vec![Component {
            ctype: ComponentType::PartOfSubcorpus,
            layer: String::from(ANNIS_NS),
            name: String::from(""),
        }];
        components
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<Operator>> {
        let components = vec![Component {
            ctype: ComponentType::PartOfSubcorpus,
            layer: String::from(ANNIS_NS),
            name: String::from(""),
        }];
        let base = BaseEdgeOpSpec {
            op_str: Some(String::from("@")),
            components,
            min_dist: self.min_dist,
            max_dist: self.max_dist,
            edge_anno: None,
            is_reflexive: false,
        };

        base.create_operator(db)
    }
}
