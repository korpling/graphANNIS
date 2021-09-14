use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    sync::Arc,
};

use graphannis_core::graph::{ANNIS_NS, NODE_NAME, NODE_NAME_KEY};

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

#[derive(Debug, Clone)]
pub struct NonExistingUnaryOperatorSpec {
    pub op: Arc<dyn BinaryOperatorSpec>,
    pub target: NodeSearchSpec,
    pub target_left: bool,
}

impl NonExistingUnaryOperatorSpec {
    fn create_subquery_conjunction(&self, node_name: String) -> Result<Conjunction> {
        let mut subquery = Conjunction::new();
        let var_flex = subquery.add_node(
            NodeSearchSpec::ExactValue {
                ns: Some(ANNIS_NS.into()),
                name: NODE_NAME.into(),
                val: Some(node_name),
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
        if let Ok(subquery) = self.create_subquery_conjunction(String::default()) {
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
        let subquery = self.create_subquery_conjunction(String::default()).ok()?;
        // Make an initial plan and store the operator order and apply it every time
        // (this should aoid costly optimization in each step)
        let operator_order = subquery.optimize_join_order_heuristics(g, &config).ok()?;

        let query_fragment = if let Some(op) = self.op.create_operator(g) {
            if self.target_left {
                format!("{} {} x", self.target, op)
            } else {
                format!("x {} {}", op, self.target)
            }
        } else {
            "unknown subquery".into()
        };

        let op = NonExistingUnaryOperator {
            spec: self.clone(),
            operator_order,
            graph: g,
            config,
            query_fragment,
        };
        Some(Box::new(op))
    }
}

struct NonExistingUnaryOperator<'a> {
    spec: NonExistingUnaryOperatorSpec,
    operator_order: Vec<usize>,
    graph: &'a AnnotationGraph,
    config: Config,
    query_fragment: String,
}

impl<'a> Display for NonExistingUnaryOperator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " !({})", self.query_fragment)
    }
}

impl<'a> UnaryOperator for NonExistingUnaryOperator<'a> {
    fn filter_match(&self, m: &graphannis_core::annostorage::Match) -> bool {
        if let Some(match_name) = self
            .graph
            .get_node_annos()
            .get_value_for_item(&m.node, &NODE_NAME_KEY)
        {
            if let Ok(subquery) = self.spec.create_subquery_conjunction(match_name.into()) {
                // Create an execution plan from the conjunction
                if let Ok(mut plan) = subquery.make_exec_plan_with_order(
                    self.graph,
                    &self.config,
                    self.operator_order.clone(),
                ) {
                    // Only return true of no match was found
                    return plan.next().is_none();
                }
            }
        }
        false
    }
}
