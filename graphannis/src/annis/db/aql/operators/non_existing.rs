use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    sync::Arc,
};

use graphannis_core::graph::{ANNIS_NS, NODE_NAME};

use crate::{
    annis::{
        db::{
            aql::{conjunction::Conjunction, Config},
            exec::nodesearch::NodeSearchSpec,
        },
        errors::Result,
        operator::{BinaryOperatorSpec, UnaryOperator, UnaryOperatorSpec},
    },
    AnnotationGraph,
};

#[derive(Debug)]
pub struct NonExistingUnaryOperatorSpec {
    pub op: Arc<dyn BinaryOperatorSpec>,
    pub target: NodeSearchSpec,
    pub target_left: bool,
}

impl NonExistingUnaryOperatorSpec {
    fn create_subquery_conjunction(&self) -> Result<Conjunction> {
        let mut subquery = Conjunction::new();
        let var_flex = subquery.add_node(
            NodeSearchSpec::ExactValue {
                ns: Some(ANNIS_NS.into()),
                name: NODE_NAME.into(),
                val: None,
                is_meta: false,
            },
            None,
        );
        let var_target = subquery.add_node(self.target.clone(), None);
        if self.target_left {
            subquery.add_operator(self.op.clone(), &var_target, &var_flex, true)?;
        } else {
            subquery.add_operator(self.op.clone(), &var_flex, &var_target, true)?;
        }

        Ok(subquery)
    }
}

impl UnaryOperatorSpec for NonExistingUnaryOperatorSpec {
    fn necessary_components(
        &self,
        g: &crate::AnnotationGraph,
    ) -> std::collections::HashSet<
        graphannis_core::types::Component<crate::model::AnnotationComponentType>,
    > {
        if let Ok(subquery) = self.create_subquery_conjunction() {
            subquery.necessary_components(g)
        } else {
            HashSet::default()
        }
    }

    fn create_operator<'b>(
        &'b self,
        g: &'b AnnotationGraph,
    ) -> Option<Box<dyn crate::annis::operator::UnaryOperator + 'b>> {
        let config = Config {
            use_parallel_joins: false,
        };
        // Create a conjunction from the definition
        let subquery = self.create_subquery_conjunction().ok()?;
        // make an initial plan and store the operator order and apply it every time
        // (this should aoid costly optimization in each step)
        let operator_order = subquery.optimize_join_order_heuristics(g, &config).ok()?;

        let op = NonExistingUnaryOperator {
            //            subquery: Arc::new(Mutex::new(subquery)),
            operator_order,
            graph: g,
            config,
        };
        Some(Box::new(op))
    }
}

struct NonExistingUnaryOperator<'a> {
    // TODO: this could be replaced with an execution plan and a *mutable* reference to the node ID search
    //subquery: Arc<Conjunction>,
    operator_order: Vec<usize>,
    graph: &'a AnnotationGraph,
    config: Config,
}

impl<'a> Display for NonExistingUnaryOperator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " !(?)")?;
        Ok(())
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperator<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        todo!()
        // let subquery = self.subquery.lock().unwrap();
        // // TODO: Replace the first node annotation value with the ID of our match
        // // Create an execution plan from the conjunction
        // if let Ok(plan) = subquery.make_exec_plan_with_order(
        //     self.graph,
        //     &self.config,
        //     self.operator_order.clone(),
        // ) {
        //     // Only return true of no match was found
        //     plan.next().is_none()
        // } else {
        //     false
        // }
    }
}
