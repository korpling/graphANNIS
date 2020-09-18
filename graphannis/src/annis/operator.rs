use super::db::aql::model::AnnotationComponentType;
use crate::{annis::db::AnnotationStorage, graph::Match, AnnotationGraph};
use graphannis_core::types::{Component, Edge};
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
    pub fn guess_max_count(&self, anno_storage: &dyn AnnotationStorage<Edge>) -> usize {
        match self {
            EdgeAnnoSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                if let Some(val) = val {
                    anno_storage.guess_max_count(ns.as_ref().map(String::as_str), name, val, val)
                } else {
                    anno_storage
                        .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name)
                }
            }
            EdgeAnnoSearchSpec::NotExactValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let total = anno_storage
                    .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name);
                total
                    - anno_storage.guess_max_count(ns.as_ref().map(String::as_str), name, val, val)
            }
            EdgeAnnoSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
            } => anno_storage.guess_max_count_regex(ns.as_ref().map(String::as_str), name, val),
            EdgeAnnoSearchSpec::NotRegexValue {
                ref ns,
                ref name,
                ref val,
            } => {
                let total = anno_storage
                    .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name);
                total
                    - anno_storage.guess_max_count_regex(ns.as_ref().map(String::as_str), name, val)
            }
        }
    }
}

pub enum EstimationType {
    SELECTIVITY(f64),
    MIN,
}

pub trait BinaryOperator: std::fmt::Display + Send + Sync {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>>;

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool;

    fn is_reflexive(&self) -> bool {
        true
    }

    fn get_inverse_operator<'a>(
        &self,
        _graph: &'a AnnotationGraph,
    ) -> Option<Box<dyn BinaryOperator + 'a>> {
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
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>>;

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<Box<dyn BinaryOperator + 'a>>;

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        None
    }

    fn is_binding(&self) -> bool {
        true
    }
}

pub trait UnaryOperatorSpec: std::fmt::Debug {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>>;

    fn create_operator(&self, db: &AnnotationGraph) -> Option<Box<dyn UnaryOperator>>;
}

pub trait UnaryOperator: std::fmt::Display + Send + Sync {
    fn filter_match(&self, m: &Match) -> bool;

    fn estimation_type(&self) -> EstimationType {
        EstimationType::SELECTIVITY(0.1)
    }
}
