use crate::AnnotationGraph;
use crate::{
    annis::{db::aql::model::AnnotationComponentType, operator::*},
    graph::Match,
};
use graphannis_core::{graph::DEFAULT_ANNO_KEY, types::Component};
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct IdenticalNodeSpec;

impl BinaryOperatorSpec for IdenticalNodeSpec {
    fn necessary_components(
        &self,
        _db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        HashSet::default()
    }

    fn create_operator<'a>(&self, _db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        Some(BinaryOperator::Index(Box::new(IdenticalNode {})))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }

    fn any_ref(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct IdenticalNode;

impl std::fmt::Display for IdenticalNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_ident_")
    }
}

impl BinaryOperatorBase for IdenticalNode {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        lhs.node == rhs.node
    }

    fn estimation_type(&self) -> EstimationType {
        EstimationType::Min
    }

    fn get_inverse_operator<'a>(&self, _graph: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        Some(BinaryOperator::Index(Box::new(self.clone())))
    }
}

impl BinaryOperatorIndex for IdenticalNode {
    fn retrieve_matches(&self, lhs: &Match) -> Box<dyn Iterator<Item = Match>> {
        Box::new(std::iter::once(Match {
            node: lhs.node,
            anno_key: DEFAULT_ANNO_KEY.clone(),
        }))
    }

    fn as_binary_operator(&self) -> &dyn BinaryOperatorBase {
        self
    }
}
