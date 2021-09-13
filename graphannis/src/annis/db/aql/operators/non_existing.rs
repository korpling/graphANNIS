use std::fmt::{Debug, Display};

use graphannis_core::{annostorage::AnnotationStorage, types::NodeID};

use crate::{
    annis::{
        db::exec::MatchFilterFunc,
        operator::{BinaryOperator, BinaryOperatorSpec, UnaryOperator, UnaryOperatorSpec},
    },
    AnnotationGraph,
};

pub struct NonExistingUnaryOperatorSpec {
    pub filter: Vec<MatchFilterFunc>,
    pub negated_op: Box<dyn BinaryOperatorSpec>,
}

impl Debug for NonExistingUnaryOperatorSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonExistingUnaryOperatorSpec")
            .field("negated_op", &self.negated_op)
            .finish()
    }
}

impl UnaryOperatorSpec for NonExistingUnaryOperatorSpec {
    fn necessary_components(
        &self,
        g: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        self.negated_op.necessary_components(g)
    }

    fn create_operator<'b>(
        &'b self,
        g: &'b AnnotationGraph,
    ) -> Option<Box<dyn crate::annis::operator::UnaryOperator + 'b>> {
        let op = NonExistingUnaryOperator {
            negated_op: self.negated_op.create_operator(g)?,
            filter: &self.filter,
            node_annos: g.get_node_annos(),
        };
        Some(Box::new(op))
    }
}

struct NonExistingUnaryOperator<'a> {
    negated_op: BinaryOperator<'a>,
    node_annos: &'a dyn AnnotationStorage<NodeID>,
    filter: &'a Vec<MatchFilterFunc>,
}

impl<'a> Display for NonExistingUnaryOperator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!",)?;
        self.negated_op.fmt(f)?;
        Ok(())
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperator<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        match &self.negated_op {
            BinaryOperator::Base(_) => todo!(),
            BinaryOperator::Index(op) => !op
                .retrieve_matches(&m)
                .any(|m| self.filter.iter().all(|f| f(&m, self.node_annos))),
        }
    }
}
