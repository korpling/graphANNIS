use std::fmt::Display;

use crate::{
    annis::{
        db::exec::nodesearch::NodeSearchSpec,
        operator::{BinaryOperator, BinaryOperatorBase, BinaryOperatorSpec, EstimationType},
    },
    AnnotationGraph,
};
use graphannis_core::annostorage::Match;

#[derive(Debug)]
pub struct NegatedOpSpec {
    pub spec_left: NodeSearchSpec,
    pub spec_right: NodeSearchSpec,
    pub negated_op: Box<dyn BinaryOperatorSpec>,
}

impl BinaryOperatorSpec for NegatedOpSpec {
    fn is_binding(&self) -> bool {
        false
    }

    fn necessary_components(
        &self,
        db: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        self.negated_op.necessary_components(db)
    }

    fn create_operator<'a>(&self, db: &'a AnnotationGraph) -> Option<BinaryOperator<'a>> {
        if let Some(negated_op) = self.negated_op.create_operator(db) {
            let op = NegatedOp { negated_op };
            Some(BinaryOperator::Base(Box::new(op)))
        } else {
            None
        }
    }
}

pub struct NegatedOp<'a> {
    negated_op: BinaryOperator<'a>,
}

impl<'a> Display for NegatedOp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!",)?;
        self.negated_op.fmt(f)?;
        Ok(())
    }
}

impl<'a> BinaryOperatorBase for NegatedOp<'a> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        // Invert the filtered logic by the actual operator
        !self.negated_op.filter_match(lhs, rhs)
    }

    fn is_reflexive(&self) -> bool {
        self.negated_op.is_reflexive()
    }

    fn estimation_type(&self) -> EstimationType {
        match self.negated_op.estimation_type() {
            EstimationType::SELECTIVITY(orig_sel) => EstimationType::SELECTIVITY(1.0 - orig_sel),
            EstimationType::MIN => EstimationType::MIN,
        }
    }
}
