use annis::annostorage::AnnoStorage;
use annis::db::GraphDB;
use annis::types::{Component, Edge, Match};
use std;

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
            &EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
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
    pub fn guess_max_count(&self, anno_storage: &AnnoStorage<Edge>) -> Option<usize> {
        match self {
            &EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let val = val.clone()?;
                if let Some(ns) = ns.clone() {
                    return Some(anno_storage.guess_max_count(
                        Some(ns.clone()),
                        name.clone(),
                        &val,
                        &val,
                    ));
                } else {
                    return Some(anno_storage.guess_max_count(None, name.clone(), &val, &val));
                }
            }
        }
    }
}

pub enum EstimationType {
    SELECTIVITY(f64),
    MIN,
}

pub trait Operator: std::fmt::Display + Send + Sync {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>>;

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool;

    fn is_reflexive(&self) -> bool {
        true
    }

    fn get_inverse_operator(&self) -> Option<Box<Operator>> {
        None
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::SELECTIVITY(0.1)
    }

    fn edge_anno_selectivity(&self) -> Option<f64> {
        None
    }
}

pub trait OperatorSpec: std::fmt::Debug {
    fn necessary_components(&self, db: &GraphDB) -> Vec<Component>;

    fn create_operator(&self, db: &GraphDB) -> Option<Box<Operator>>;

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        None
    }
}
