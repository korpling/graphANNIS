pub mod ast;
pub mod normalize;
pub mod operators;
pub mod parser;

use errors::*;
use operator::OperatorSpec;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use std::collections::HashMap;

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
            // TODO: handle manually named variables
            let mut alternatives: Vec<Conjunction> = Vec::new();
            for c in ast.into_iter() {
                let mut q = Conjunction::new();
                let mut pos_to_node: HashMap<ast::Pos, usize> = HashMap::default();
                for f in c.into_iter() {
                    // add all nodes and remember their position in the input text
                    if let ast::Factor::Literal(literal) = f {
                        match literal {
                            ast::Literal::NodeSearch { spec, pos } => {
                                if let Some(pos) = pos {
                                    let idx = q.add_node(spec, None);
                                    pos_to_node.insert(pos, idx);
                                }
                            }
                            ast::Literal::BinaryOp { lhs, op, rhs, .. } => {

                                let lhs_idx = match lhs {
                                    ast::Operand::Literal { spec, pos } => {
                                        pos_to_node.entry(pos).or_insert_with(|| q.add_node(spec.as_ref().clone(), None)).clone()
                                    },
                                    ast::Operand::NodeRef(node_ref) => {
                                        match node_ref {
                                            ast::NodeRef::ID(id) => id,
                                            ast::NodeRef::Name(name) => unimplemented!(), 
                                        }
                                    }
                                };

                                let rhs_idx = match rhs {
                                    ast::Operand::Literal { spec, pos } => {
                                        pos_to_node.entry(pos).or_insert_with(|| q.add_node(spec.as_ref().clone(), None)).clone()
                                    },
                                    ast::Operand::NodeRef(node_ref) => {
                                        match node_ref {
                                            ast::NodeRef::ID(id) => id,
                                            ast::NodeRef::Name(name) => unimplemented!(), 
                                        }
                                    }
                                };

                                // add the operator itself
                                q.add_operator(make_operator_spec(op), lhs_idx, rhs_idx);
                            }
                        };
                    }
                }
            }
            return Ok(Disjunction::new(alternatives));
        }
        Err(e) => {
            return Err(format!("{}", e).into());
        }
    };
}
