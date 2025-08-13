use super::db::{aql::model::AnnotationComponentType, exec::CostEstimate};
use crate::{AnnotationGraph, errors::Result, graph::Match};
use graphannis_core::{annostorage::EdgeAnnotationStorage, types::Component};
use std::{collections::HashSet, fmt::Display};

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
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
            EdgeAnnoSearchSpec::ExactValue { ns, name, val } => {
                let qname = if let Some(ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                if let Some(val) = val {
                    write!(f, "{}=\"{}\"", qname, val)
                } else {
                    write!(f, "{}", qname)
                }
            }
            EdgeAnnoSearchSpec::NotExactValue { ns, name, val } => {
                let qname = if let Some(ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                write!(f, "{}!=\"{}\"", qname, val)
            }
            EdgeAnnoSearchSpec::RegexValue { ns, name, val } => {
                let qname = if let Some(ns) = ns {
                    format!("{}:{}", ns, name)
                } else {
                    name.clone()
                };

                write!(f, "{}=/{}/", qname, val)
            }
            EdgeAnnoSearchSpec::NotRegexValue { ns, name, val } => {
                let qname = if let Some(ns) = ns {
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
    pub fn guess_max_count(&self, anno_storage: &dyn EdgeAnnotationStorage) -> Result<usize> {
        match self {
            EdgeAnnoSearchSpec::ExactValue { ns, name, val } => {
                let result = if let Some(val) = val {
                    anno_storage.guess_max_count(ns.as_ref().map(String::as_str), name, val, val)?
                } else {
                    anno_storage
                        .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name)?
                };
                Ok(result)
            }
            EdgeAnnoSearchSpec::NotExactValue { ns, name, val } => {
                let total = anno_storage
                    .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name)?;
                let result = total
                    - anno_storage.guess_max_count(
                        ns.as_ref().map(String::as_str),
                        name,
                        val,
                        val,
                    )?;
                Ok(result)
            }
            EdgeAnnoSearchSpec::RegexValue { ns, name, val } => Ok(
                anno_storage.guess_max_count_regex(ns.as_ref().map(String::as_str), name, val)?
            ),
            EdgeAnnoSearchSpec::NotRegexValue { ns, name, val } => {
                let total = anno_storage
                    .number_of_annotations_by_name(ns.as_ref().map(String::as_str), name)?;
                let result = total
                    - anno_storage.guess_max_count_regex(
                        ns.as_ref().map(String::as_str),
                        name,
                        val,
                    )?;
                Ok(result)
            }
        }
    }
}

/// Represents the different strategies to estimate the output of size of applying an operator.
#[derive(Clone)]
pub enum EstimationType {
    /// Estimate using the given selectivity.
    /// This means the cross product of the input sizes is multiplied with this factor to get the output size.
    Selectivity(f64),
    /// Use the smallest one of the input sizes to estimate the output size.
    Min,
}

pub trait BinaryOperatorBase: std::fmt::Display + Send + Sync {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool>;

    fn is_reflexive(&self) -> bool {
        true
    }

    fn get_inverse_operator<'a>(
        &self,
        _graph: &'a AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'a>>> {
        Ok(None)
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        Ok(EstimationType::Selectivity(0.1))
    }

    fn edge_anno_selectivity(&self) -> Result<Option<f64>> {
        Ok(None)
    }
}

/// Holds an instance of one of the possibly binary operator types.
pub enum BinaryOperator<'a> {
    /// Base implementation, usable as filter and for nested loop joins
    Base(Box<dyn BinaryOperatorBase + 'a>),
    /// Implementation that can be used in an index join
    Index(Box<dyn BinaryOperatorIndex + 'a>),
}

impl Display for BinaryOperator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Base(op) => op.fmt(f),
            BinaryOperator::Index(op) => op.fmt(f),
        }
    }
}

impl BinaryOperatorBase for BinaryOperator<'_> {
    fn filter_match(
        &self,
        lhs: &graphannis_core::annostorage::Match,
        rhs: &graphannis_core::annostorage::Match,
    ) -> Result<bool> {
        match self {
            BinaryOperator::Base(op) => op.filter_match(lhs, rhs),
            BinaryOperator::Index(op) => op.filter_match(lhs, rhs),
        }
    }

    fn is_reflexive(&self) -> bool {
        match self {
            BinaryOperator::Base(op) => op.is_reflexive(),
            BinaryOperator::Index(op) => op.is_reflexive(),
        }
    }

    fn get_inverse_operator<'b>(
        &self,
        graph: &'b AnnotationGraph,
    ) -> Result<Option<BinaryOperator<'b>>> {
        match self {
            BinaryOperator::Base(op) => op.get_inverse_operator(graph),
            BinaryOperator::Index(op) => op.get_inverse_operator(graph),
        }
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        match self {
            BinaryOperator::Base(op) => op.estimation_type(),
            BinaryOperator::Index(op) => op.estimation_type(),
        }
    }

    fn edge_anno_selectivity(&self) -> Result<Option<f64>> {
        match self {
            BinaryOperator::Base(op) => op.edge_anno_selectivity(),
            BinaryOperator::Index(op) => op.edge_anno_selectivity(),
        }
    }
}

/// A binary operator that can be used in an [`IndexJoin`](crate::annis::db::exec::indexjoin::IndexJoin).
pub trait BinaryOperatorIndex: BinaryOperatorBase {
    fn retrieve_matches<'a>(&'a self, lhs: &Match) -> Box<dyn Iterator<Item = Result<Match>> + 'a>;

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase;
}

pub trait BinaryOperatorSpec: std::fmt::Debug + Send + Sync {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>>;

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>>;

    fn get_edge_anno_spec(&self) -> Option<EdgeAnnoSearchSpec> {
        None
    }

    fn is_binding(&self) -> bool {
        true
    }

    #[cfg(test)]
    fn into_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any>;

    #[cfg(test)]
    fn any_ref(&self) -> &dyn std::any::Any;
}

pub trait UnaryOperatorSpec: std::fmt::Debug {
    fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>>;

    fn create_operator<'a>(
        &'a self,
        db: &'a AnnotationGraph,
    ) -> Result<Box<dyn UnaryOperator + 'a>>;
}

pub trait UnaryOperator: std::fmt::Display {
    fn filter_match(&self, m: &Match) -> Result<bool>;

    fn estimation_type(&self) -> EstimationType {
        EstimationType::Selectivity(0.1)
    }
}
