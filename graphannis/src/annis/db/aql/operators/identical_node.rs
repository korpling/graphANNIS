use crate::AnnotationGraph;
use crate::{
    annis::{db::aql::model::AnnotationComponentType, operator::*},
    graph::Match,
};
use graphannis_core::{graph::DEFAULT_ANNO_KEY, types::Component};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalNodeSpec;

impl BinaryOperatorSpec for IdenticalNodeSpec {
    fn necessary_components(
        &self,
        _db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::default()
    }

    fn create_operator<'a>(
        &self,
        _db: &'a AnnotationGraph,
    ) -> Option<Box<dyn BinaryOperator + 'a>> {
        Some(Box::new(IdenticalNode {}))
    }
}

#[derive(Clone, Debug)]
pub struct IdenticalNode;

impl std::fmt::Display for IdenticalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_ident_")
    }
}

impl BinaryOperator for IdenticalNode {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        lhs.node == rhs.node
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::MIN
    }

    fn get_inverse_operator<'a>(
        &self,
        _graph: &'a AnnotationGraph,
    ) -> Option<Box<dyn BinaryOperator + 'a>> {
        Some(Box::new(self.clone()))
    }
}

impl BinaryIndexOperator for IdenticalNode {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        Box::new(std::iter::once(Match {
            node: lhs.node,
            anno_key: DEFAULT_ANNO_KEY.clone(),
        }))
    }
}
