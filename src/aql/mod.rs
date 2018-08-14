pub mod ast;
pub mod normalize;
pub mod operators;
pub mod parser;

use errors::*;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use operator::OperatorSpec;

fn make_operator_spec(op : ast::BinaryOpSpec) -> Box<OperatorSpec> {
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
            let mut alternatives : Vec<Conjunction> = Vec::new();
            for c in ast.into_iter() {
                let mut q = Conjunction::new();
                for f in c.into_iter() {
                    let mut last_lhs_literal : Option<usize> = None;
                    if let ast::Factor::Literal(literal) = f {
                        match literal {
                            ast::Literal::NodeSearch { spec, pos } => {
                                last_lhs_literal = None;
                                q.add_node(spec, None);
                            },
                            ast::Literal::BinaryOp { lhs, op, rhs, pos } => {
                                if let (ast::Operand::Literal(lhs_node), ast::Operand::Literal(rhs_node)) = (lhs, rhs) {
                                    // only add the LHS if not already added
                                    let idx_left = if let Some(last_lhs_idx) = last_lhs_literal {
                                        last_lhs_idx
                                    } else {
                                        let idx = q.add_node(lhs_node.as_ref().clone(), None);
                                        last_lhs_literal = Some(idx);
                                        idx
                                    };
                                    // always add the RHS
                                    let idx_right = q.add_node(rhs_node.as_ref().clone(), None);
                                    q.add_operator(make_operator_spec(op), idx_left, idx_right);
                                }
                            }
                        };
                    }
                }
            }
            unimplemented!()
//            return Ok(Disjunction::new(alternatives));
        }
        Err(e) => {
            return Err(format!("{}", e).into());
        }
    };
}
