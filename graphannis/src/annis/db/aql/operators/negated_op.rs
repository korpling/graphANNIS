use std::{fmt::Display, sync::Arc};

use crate::{
    annis::{
        db::exec::CostEstimate,
        operator::{BinaryOperator, BinaryOperatorBase, BinaryOperatorSpec, EstimationType},
    },
    errors::Result,
    AnnotationGraph,
};
use graphannis_core::annostorage::Match;

#[derive(Debug)]
pub struct NegatedOpSpec {
    pub negated_op: Arc<dyn BinaryOperatorSpec>,
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

    fn create_operator<'a>(
        &self,
        db: &'a AnnotationGraph,
        cost_estimate: Option<(&CostEstimate, &CostEstimate)>,
    ) -> Result<BinaryOperator<'a>> {
        let negated_op = self.negated_op.create_operator(db, cost_estimate)?;
        let op = NegatedOp { negated_op };
        Ok(BinaryOperator::Base(Box::new(op)))
    }

    #[cfg(test)]
    fn into_any(self: Arc<Self>) -> Arc<dyn std::any::Any> {
        self
    }

    #[cfg(test)]
    fn any_ref(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct NegatedOp<'a> {
    negated_op: BinaryOperator<'a>,
}

impl Display for NegatedOp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!",)?;
        self.negated_op.fmt(f)?;
        Ok(())
    }
}

impl BinaryOperatorBase for NegatedOp<'_> {
    fn filter_match(&self, lhs: &Match, rhs: &Match) -> Result<bool> {
        // Invert the filtered logic by the actual operator
        let orig = self.negated_op.filter_match(lhs, rhs)?;
        Ok(!orig)
    }

    fn is_reflexive(&self) -> bool {
        self.negated_op.is_reflexive()
    }

    fn estimation_type(&self) -> Result<EstimationType> {
        match self.negated_op.estimation_type()? {
            EstimationType::Selectivity(orig_sel) => {
                Ok(EstimationType::Selectivity(1.0 - orig_sel))
            }
            EstimationType::Min => Ok(EstimationType::Min),
        }
    }
}
