use std::fmt::Display;

use crate::{
    annis::{
        db::exec::nodesearch::NodeSearchSpec,
        operator::{BinaryOperator, BinaryOperatorSpec, UnaryOperator, UnaryOperatorSpec},
    },
    AnnotationGraph,
};

#[derive(Debug)]
pub struct NonExistingUnaryOperatorSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
    pub negated_op: Box<dyn BinaryOperatorSpec>,
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

    fn create_operator<'a>(
        &'a self,
        g: &'a AnnotationGraph,
    ) -> Option<Box<dyn crate::annis::operator::UnaryOperator + 'a>> {
        let op = NonExistingUnaryOperator {
            negated_op: self.negated_op.create_operator(g)?,
        };
        Some(Box::new(op))
    }
}

struct NonExistingUnaryOperator<'a> {
    negated_op: BinaryOperator<'a>,
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
        todo!()
    }
}
