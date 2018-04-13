use annostorage::AnnoStorage;
use {AnnoKey, Annotation, Component, ComponentType, Edge, Match, NodeID};
use graphstorage::{GraphStorage, GraphStatistic};
use graphdb::{GraphDB, ANNIS_NS};
use operator::{Operator, OperatorSpec, EstimationType};
use util;
use std;
use std::sync::Arc;
use stringstorage::StringStorage;

#[derive(Clone, Debug)]
pub enum EdgeAnnoSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
    },
}

impl std::fmt::Display for EdgeAnnoSearchSpec {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         match self {
             &EdgeAnnoSearchSpec::ExactValue{ref ns, ref name, ref val} => {
                 let qname = if let &Some(ref ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };
                
                if let &Some(ref val) = val {
                    write!(f, "{}={}", qname, val)
                } else {
                    write!(f, "{}", qname)
                }
             }
         }
    }
}

impl EdgeAnnoSearchSpec {
    pub fn get_anno(&self, strings: &StringStorage) -> Option<Annotation> {
        match self {
            &EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let ns = if let &Some(ref s) = ns {
                    strings.find_id(s)?.clone()
                } else {
                    0
                };
                let val = if let &Some(ref s) = val {
                    strings.find_id(s)?.clone()
                } else {
                    0
                };
                let name = strings.find_id(name)?.clone();
                let mut anno = Annotation {
                    key: AnnoKey { ns, name },
                    val,
                };

                Some(anno)
            }
        }
    }

    pub fn guess_max_count(&self, anno_storage : &AnnoStorage<Edge>, strings : &StringStorage) -> Option<usize> {
        match self {
            &EdgeAnnoSearchSpec::ExactValue{ref ns, ref name, ref val} => {
                let val = val.clone()?;
                let name_id = strings.find_id(&name)?;
                if let Some(ns) = ns.clone() {
                    let ns_id = strings.find_id(&ns)?;
                    return Some(anno_storage.guess_max_count(Some(ns_id.clone()), name_id.clone(), &val, &val));
                } else {
                    return Some(anno_storage.guess_max_count(None, name_id.clone(), &val, &val)); 
                }            
            }
        }
    }
}

#[derive(Clone, Debug)]
struct BaseEdgeOpSpec {
    pub components: Vec<Component>,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
    pub is_reflexive: bool,
    pub query_fragment: Option<String>,
}

struct BaseEdgeOp {
    gs: Vec<Arc<GraphStorage>>,
    edge_anno: Option<Annotation>,
    spec: BaseEdgeOpSpec,
    node_annos: Arc<AnnoStorage<NodeID>>,
    strings: Arc<StringStorage>,
    node_type_key: AnnoKey,
}

impl BaseEdgeOp {
    pub fn new(db: &GraphDB, spec: BaseEdgeOpSpec) -> Option<BaseEdgeOp> {
        let mut gs: Vec<Arc<GraphStorage>> = Vec::new();
        for c in spec.components.iter() {
            gs.push(db.get_graphstorage(c)?);
        }
        let edge_anno = if let Some(a) = spec.edge_anno.as_ref() {
            Some(a.get_anno(&db.strings)?)
        } else {
            None
        };
        Some(BaseEdgeOp {
            gs,
            edge_anno,
            spec,
            node_annos: db.node_annos.clone(),
            strings: db.strings.clone(),
            node_type_key: db.get_node_type_key(),
        })
    }
}

impl OperatorSpec for BaseEdgeOpSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.components.clone()
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        let optional_op = BaseEdgeOp::new(db, self.clone());
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> { self.edge_anno.clone() }
}

fn check_edge_annotation(
    edge_anno: &Option<Annotation>,
    gs: &GraphStorage,
    source: &NodeID,
    target: &NodeID,
) -> bool {
    if edge_anno.is_none() {
        return true;
    }

    let anno_template: &Annotation = edge_anno.as_ref().unwrap();
    if anno_template.val == 0 || anno_template.val == <NodeID>::max_value() {
        // must be a valid value
        return false;
    } else {
        // check if the edge has the correct annotation first
        for a in gs.get_edge_annos(&Edge {
            source: source.clone(),
            target: target.clone(),
        }) {
            if util::check_annotation_equal(&anno_template, &a) {
                return true;
            }
        }
    }
    return false;
}

impl BaseEdgeOp {}

impl std::fmt::Display for BaseEdgeOp {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         if let Some(ref query_fragment) = self.spec.query_fragment {
             write!(f, "{}", query_fragment)
         } else {
            write!(f, "?")
         }
    }
}

impl Operator for BaseEdgeOp {
    fn retrieve_matches<'b>(&'b self, lhs: &Match) -> Box<Iterator<Item = Match> + 'b> {
        let lhs = lhs.clone();

        let it_all_gs = self.gs.iter().flat_map(move |e| {
            let lhs = lhs.clone();
            let spec = self.spec.clone();

            e.as_ref()
                .find_connected(&lhs.node, spec.min_dist, spec.max_dist)
                .filter(move |candidate| {
                    check_edge_annotation(&self.edge_anno, e.as_ref(), &lhs.clone().node, candidate)
                })
                .map(|n| {
                    Match {
                        node: n,
                        anno: Annotation::default(),
                    }
                })
        });

        if self.gs.len() == 1 {
            // directly return all matched nodes since when having only one component
            // no duplicates are possible
            return Box::new(it_all_gs);
        } else {
            // collect all intermediate results and remove duplicates
            let mut all: Vec<Match> = it_all_gs.collect();
            all.sort_unstable();
            all.dedup();
            return Box::new(all.into_iter());
        }
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        for e in self.gs.iter() {
            if e.is_connected(&lhs.node, &rhs.node, self.spec.min_dist, self.spec.max_dist) {
                if check_edge_annotation(&self.edge_anno, e.as_ref(), &lhs.node, &rhs.node) {
                    return true;
                }
            }
        }
        return false;
    }

    fn is_reflexive(&self) -> bool {
        self.spec.is_reflexive
    }

    fn estimation_type(&self) -> EstimationType {
        if self.gs.is_empty() {
            // will not find anything
            return EstimationType::SELECTIVITY(0.0);
        }

        
        let max_nodes : f64 = self.node_annos.guess_max_count(Some(self.node_type_key.ns), self.node_type_key.name, "node", "node") as f64;

        let mut worst_sel : f64 = 0.0;

        for g  in self.gs.iter() {
            let g : &Arc<GraphStorage> = g;

            let mut gs_selectivity = 0.01;

            if let Some(stats) = g.get_statistics() {
                let stats : &GraphStatistic = stats;
                if stats.cyclic {
                    // can get all other nodes
                    return EstimationType::SELECTIVITY(1.0);
                }
                // get number of nodes reachable from min to max distance
                let max_path_length = std::cmp::min(self.spec.max_dist, stats.max_depth) as i32;
                let min_path_length = std::cmp::max(0, self.spec.min_dist-1) as i32;

                if stats.avg_fan_out > 1.0 {
                     // Assume two complete k-ary trees (with the average fan-out as k)
                    // as defined in "Thomas Cormen: Introduction to algorithms (2009), page 1179)
                    // with the maximum and minimum height. Calculate the number of nodes for both complete trees and
                    // subtract them to get an estimation of the number of nodes that fullfull the path length criteria.
                    let k = stats.avg_fan_out;

                    let reachable_max : f64 = ((k.powi(max_path_length) - 1.0) / (k - 1.0)).ceil();
                    let reachable_min : f64 = ((k.powi(min_path_length) - 1.0) / (k - 1.0)).ceil();

                    let reachable = reachable_max - reachable_min;

                    gs_selectivity = reachable / max_nodes;
                } else {
                    // We can't use the formula for complete k-ary trees because we can't divide by zero and don't want negative
                    // numbers. Use the simplified estimation with multiplication instead.
                    let reachable_max : f64 = (stats.avg_fan_out * (max_path_length as f64)).ceil();
                    let reachable_min : f64 = (stats.avg_fan_out * (min_path_length as f64)).ceil();

                    gs_selectivity = (reachable_max - reachable_min) / max_nodes;
                }
            }

            if worst_sel < gs_selectivity {
                worst_sel = gs_selectivity;
            }
        }  // end for

        return EstimationType::SELECTIVITY(worst_sel);
    }

    fn edge_anno_selectivity(&self) -> Option<f64> {
        if let Some(ref edge_anno) = self.edge_anno {
            let edge_anno : Annotation = edge_anno.clone();
            if edge_anno == Annotation::default() {
                return Some(1.0)
            } else {
                let mut worst_sel = 0.0;
                for g  in self.gs.iter() {
                    let g : &Arc<GraphStorage> = g;
                    let anno_storage = g.get_anno_storage();
                    let num_of_annos = anno_storage.len();
                    if num_of_annos == 0 {
                        // we won't be able to find anything if there are no annotations
                        return Some(0.0);
                    } else {
                        
                        if let Some(val_str) = self.strings.str(edge_anno.val) {
                            let ns = if edge_anno.key.ns == 0 {None} else {Some(edge_anno.key.name)};
                            let guessed_count = anno_storage.guess_max_count(ns, edge_anno.key.name, val_str, val_str);

                            let g_sel : f64 = (guessed_count as f64) / (num_of_annos as f64);
                            if g_sel > worst_sel {
                                worst_sel = g_sel;
                            }
                        } else {
                            // if value string is unknown, there is nothing to find
                            return Some(0.0);
                        }
                    }
                }
                return Some(worst_sel);
            }
        }
        return None;
    }
}

#[derive(Debug,Clone)]
pub struct DominanceSpec {
    base: BaseEdgeOpSpec,
}

impl DominanceSpec {
    pub fn new(
        db: &GraphDB,
        name: &str,
        min_dist: usize,
        max_dist: usize,
        edge_anno: Option<EdgeAnnoSearchSpec>,
    ) -> DominanceSpec {
        let components = db.get_all_components(Some(ComponentType::Dominance), Some(name));
        let frag = if let Some(ref edge_anno) = edge_anno {
            format!(">{}[{}]", name, edge_anno)
        } else {
            format!(">{} {},{}", name, min_dist.clone(), max_dist.clone())
        };
        DominanceSpec {
            base: BaseEdgeOpSpec {
                query_fragment: Some(frag),
                components,
                min_dist,
                max_dist,
                edge_anno,
                is_reflexive: true,
            },
        }
    }
}


impl OperatorSpec for DominanceSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.base.necessary_components()
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        self.base.create_operator(db)
    }
}

#[derive(Debug,Clone)]
pub struct PointingSpec {
    base: BaseEdgeOpSpec,
}

impl PointingSpec {
    pub fn new(
        db: &GraphDB,
        name: &str,
        min_dist: usize,
        max_dist: usize,
        edge_anno: Option<EdgeAnnoSearchSpec>,
    ) -> DominanceSpec {
        let components = db.get_all_components(Some(ComponentType::Pointing), Some(name));
        let frag = if let Some(ref edge_anno) = edge_anno {
            format!("->{}[{}]", name, edge_anno)
        } else {
            format!("->{} {},{}", name, min_dist.clone(), max_dist.clone())
        };
        DominanceSpec {
            base: BaseEdgeOpSpec {
                components,
                min_dist,
                max_dist,
                edge_anno,
                is_reflexive: true,
                query_fragment: Some(frag),
            },
        }
    }
}


impl OperatorSpec for PointingSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.base.necessary_components()
    }

    fn create_operator<'b>(&self, db: &GraphDB) -> Option<Box<Operator>> {
        self.base.create_operator(db)
    }
}

#[derive(Debug,Clone)]
pub struct PartOfSubCorpusSpec {
    base: BaseEdgeOpSpec,
}

impl PartOfSubCorpusSpec {
    pub fn new(min_dist : usize, max_dist: usize) -> PartOfSubCorpusSpec {
        let components = vec![
            Component {
                ctype: ComponentType::PartOfSubcorpus,
                layer: String::from(ANNIS_NS),
                name: String::from(""),
            },
        ];
        PartOfSubCorpusSpec {
            base: BaseEdgeOpSpec {
                query_fragment: Some(format!("@{},{}", min_dist.clone(), max_dist.clone())),
                components,
                min_dist,
                max_dist,
                edge_anno: None,
                is_reflexive: false,
            },
        }
    }
}


impl OperatorSpec for PartOfSubCorpusSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.base.necessary_components()
    }

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>> {
        self.base.create_operator(db)
    }
}
