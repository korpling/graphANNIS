use {AnnoKey, Annotation, Component, ComponentType, Edge, Match, NodeID};
use graphstorage::GraphStorage;
use graphdb::GraphDB;
use operator::{Operator, OperatorSpec};
use util;
use std::rc::Rc;
use stringstorage::StringStorage;

#[derive(Clone)]
pub enum EdgeAnnoSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
    },
}

impl EdgeAnnoSearchSpec {
    pub fn get_anno(&self, strings: &StringStorage) -> Option<Annotation> {
        match self {
            &EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let ns = if let &Some(ref s) = ns {strings.find_id(s)?.clone()} else {0};
                let val = if let &Some(ref s) = val {strings.find_id(s)?.clone()} else {0};
                let name = strings.find_id(name)?.clone();
                let mut anno = Annotation {
                    key: AnnoKey {
                        ns, name,
                    },
                    val,
                };

                Some(anno)
            },
        }
    }
}

#[derive(Clone)]
struct BaseEdgeOpSpec {
    pub components: Vec<Component>,
    pub min_dist: usize,
    pub max_dist: usize,
    pub edge_anno: Option<EdgeAnnoSearchSpec>,
}

struct BaseEdgeOp {
    gs: Vec<Rc<GraphStorage>>,
    edge_anno : Option<Annotation>,
    spec: BaseEdgeOpSpec,
}

impl BaseEdgeOp {
    pub fn new(db: &GraphDB, spec: BaseEdgeOpSpec) -> Option<BaseEdgeOp> {
        let mut gs: Vec<Rc<GraphStorage>> = Vec::new();
        for c in spec.components.iter() {
            gs.push(db.get_graphstorage(c)?);
        }
        let edge_anno = if let Some(a) = spec.edge_anno.as_ref() {Some(a.get_anno(&db.strings)?)} else {None};
        Some(BaseEdgeOp { gs, edge_anno, spec})
    }
}

impl OperatorSpec for BaseEdgeOpSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.components.clone()
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        let optional_op = BaseEdgeOp::new(db, self.clone());
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
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
                        anno: Annotation {
                            key: AnnoKey { ns: 0, name: 0 },
                            val: 0,
                        },
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
}

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
        let components = db.get_all_components(ComponentType::Dominance, name);
        DominanceSpec {
            base: BaseEdgeOpSpec {
                components,
                min_dist,
                max_dist,
                edge_anno,
            },
        }
    }
}


impl OperatorSpec for DominanceSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.base.necessary_components()
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        self.base.create_operator(db)
    }
}

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
        let components = db.get_all_components(ComponentType::Pointing, name);
        DominanceSpec {
            base: BaseEdgeOpSpec {
                components,
                min_dist,
                max_dist,
                edge_anno,
            },
        }
    }
}


impl OperatorSpec for PointingSpec {
    fn necessary_components(&self) -> Vec<Component> {
        self.base.necessary_components()
    }

    fn create_operator<'b>(&self, db: &'b GraphDB) -> Option<Box<Operator + 'b>> {
        self.base.create_operator(db)
    }
}
