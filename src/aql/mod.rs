pub mod ast;
pub mod normalize;
pub mod operators;
pub mod parser;

use errors::*;
use operator::OperatorSpec;
use exec::nodesearch::NodeSearchSpec;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use std::collections::HashMap;
use std::collections::BTreeMap;

fn make_operator_spec(op: ast::BinaryOpSpec) -> Box<OperatorSpec> {
    match op {
        ast::BinaryOpSpec::Dominance(spec) => Box::new(spec),
        ast::BinaryOpSpec::Pointing(spec) => Box::new(spec),
        ast::BinaryOpSpec::Precedence(spec) => Box::new(spec),
        ast::BinaryOpSpec::Overlap(spec) => Box::new(spec),
        ast::BinaryOpSpec::IdenticalCoverage(spec) => Box::new(spec),
    }
}

pub fn parse<'a>(query_as_aql: &str) -> Result<Disjunction<'a>> {
    let ast = parser::DisjunctionParser::new().parse(query_as_aql);
    match ast {
        Ok(mut ast) => {
            // make sure AST is in DNF
            normalize::to_disjunctive_normal_form(&mut ast);

            // map all conjunctions and its literals
            let mut alternatives: Vec<Conjunction> = Vec::new();
            for c in ast.into_iter() {
                let mut q = Conjunction::new();
                // collect and sort all node searches according to their start position in the text
                let mut pos_to_node : BTreeMap<usize, (NodeSearchSpec, Option<String>)> = BTreeMap::default();
                for f in c.iter() {
                    if let ast::Factor::Literal(literal) = f {
                        match literal {
                            ast::Literal::NodeSearch { spec, pos, variable } => {
                                if let Some(pos) = pos {
                                    pos_to_node.insert(pos.start, (spec.clone(), variable.clone()));
                                }
                            },
                            ast::Literal::BinaryOp { lhs, rhs, .. } => {

                                if let ast::Operand::Literal{spec, pos, variable} = lhs {
                                    pos_to_node.entry(pos.start).or_insert_with(|| (spec.as_ref().clone(), variable.clone()));
                                }
                                if let ast::Operand::Literal{spec, pos, variable} = rhs {
                                    pos_to_node.entry(pos.start).or_insert_with(|| (spec.as_ref().clone(), variable.clone()));
                                }                            
                            }
                        };
                    }
                }

                // add all nodes specs in order of their start position
                let mut pos_to_node_id: HashMap<usize, String> = HashMap::default();
                for (start_pos,(node_spec, variable)) in pos_to_node.into_iter() {
                    let variable = variable.as_ref().map(|s| &**s);
                    let idx = q.add_node(node_spec, variable);
                    pos_to_node_id.insert(start_pos, idx);
                }

                // finally add all operators

                for f in c.into_iter() {
                    if let ast::Factor::Literal(literal) = f {
                        if let ast::Literal::BinaryOp { lhs, op, rhs, .. } = literal {

                            let idx_left = match lhs {
                                ast::Operand::Literal { spec, pos, .. } => {
                                    pos_to_node_id.entry(pos.start).or_insert_with(|| q.add_node(spec.as_ref().clone(), None)).clone()
                                },
                                ast::Operand::NodeRef(node_ref) => {
                                    match node_ref {
                                        ast::NodeRef::ID(id) => id.to_string(),
                                        ast::NodeRef::Name(name) => name, 
                                    }
                                }
                            };

                            let idx_right = match rhs {
                                ast::Operand::Literal { spec, pos, .. } => {
                                    pos_to_node_id.entry(pos.start).or_insert_with(|| q.add_node(spec.as_ref().clone(), None)).clone()
                                },
                                ast::Operand::NodeRef(node_ref) => {
                                    match node_ref {
                                        ast::NodeRef::ID(id) => id.to_string(),
                                        ast::NodeRef::Name(name) => name, 
                                    }
                                }
                            };

                            q.add_operator(make_operator_spec(op), &idx_left, &idx_right)?;
                        }
                    }
                }

                // add the conjunction to the disjunction
                alternatives.push(q);
            }
            return Ok(Disjunction::new(alternatives));
        }
        Err(e) => {
            return Err(format!("{}", e).into());
        }
    };
}
