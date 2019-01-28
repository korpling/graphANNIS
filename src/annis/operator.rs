use crate::annis::db::AnnotationStorage;
use crate::annis::db::{Graph, Match};
use crate::annis::types::{Component, Edge};
use std;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum EdgeAnnoSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
    },
    NotExactValue {
        ns: Option<String>,
        name: String,
        val: String,
    },
    RegexValue {
        ns: Option<String>,
        name: String,
        val: String,
    },
    NotRegexValue {
        ns: Option<String>,
        name: String,
        val: String,
    },
}

impl std::fmt::Display for EdgeAnnoSearchSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let qname = if let Some(ref ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                if let Some(ref val) = val {
                    write!(f, "{}=\"{}\"", qname, val)
                } else {
                    write!(f, "{}", qname)
                }
            }
            EdgeAnnoSearchSpec::NotExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let qname = if let Some(ref ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                write!(f, "{}!=\"{}\"", qname, val)
            }
            EdgeAnnoSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let qname = if let Some(ref ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                write!(f, "{}=/{}/", qname, val)
            }
            EdgeAnnoSearchSpec::NotRegexValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let qname = if let Some(ref ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                write!(f, "{}!=/{}/", qname, val)
            }
        }
    }
}

impl EdgeAnnoSearchSpec {
    pub fn guess_max_count(&self, anno_storage: &AnnotationStorage<Edge>) -> usize {
        match self {
            EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                if let Some(val) = val {
                    let val = val.clone();
                    return anno_storage.guess_max_count(ns.clone(), name.clone(), &val, &val);
                } else {
                    return anno_storage.number_of_annotations_by_name(ns.clone(), name.clone());
                }                
            }
            EdgeAnnoSearchSpec::NotExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let val = val.clone();
                let total = anno_storage.number_of_annotations_by_name(ns.clone(), name.clone());
                return total - anno_storage.guess_max_count(ns.clone(), name.clone(), &val, &val);
            }
            EdgeAnnoSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let val = val.clone();
                return anno_storage.guess_max_count_regex(ns.clone(), name.clone(), &val);
            }
            EdgeAnnoSearchSpec::NotRegexValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let total = anno_storage.number_of_annotations_by_name(ns.clone(), name.clone());
                return total - anno_storage.guess_max_count_regex(ns.clone(), name.clone(), &val);
            }
        }
    }
}

pub enum EstimationType {
    SELECTIVITY(f64),
    MIN,
}

pub trait BinaryOperator: std::fmt::Display + Send + Sync {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>>;

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool;

    fn is_reflexive(&self) -> bool {
        true
    }

    fn get_inverse_operator(&self) -> Option<Box<BinaryOperator>> {
        None
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::SELECTIVITY(0.1)
    }

    fn edge_anno_selectivity(&self) -> Option<f64> {
        None
    }
}

pub trait BinaryOperatorSpec: std::fmt::Debug {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component>;

    fn create_operator(&self, db: &Graph) -> Option<Box<BinaryOperator>>;

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        None
    }


    fn is_binding(&self) -> bool {
        true
    }
}

pub trait UnaryOperatorSpec: std::fmt::Debug {
    fn necessary_components(&self, db: &Graph) -> HashSet<Component>;

    fn create_operator(&self, db: &Graph) -> Option<Box<UnaryOperator>>;
}

pub trait UnaryOperator : std::fmt::Display + Send + Sync {

    fn filter_match(&self, m: &Match) -> bool;

    fn estimation_type(&self) -> EstimationType {
        EstimationType::SELECTIVITY(0.1)
    }
}
