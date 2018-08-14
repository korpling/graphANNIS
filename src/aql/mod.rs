pub mod ast;
pub mod normalize;
pub mod operators;
pub mod parser;

use errors::*;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;

pub fn parse<'a>(query_as_aql: &str) -> Result<Disjunction<'a>> {
    let ast = parser::DisjunctionParser::new().parse(query_as_aql);
    match ast {
        Ok(mut ast) => {
            // make sure AST is in DNF
            normalize::to_disjunctive_normal_form(&mut ast);

            // map all conjunctions and its literals
            // TODO: handle manually named variables
            let mut alternatives = Vec::new();
            for c in ast.into_iter() {
                let mut q = Conjunction::new();
                for f in c.into_iter() {
                    if let ast::Factor::Literal(literal) = f {
                        match literal {
                            ast::Literal::NodeSearch { spec, pos } => {
                                q.add_node(spec, None);
                            },
                            ast::Literal::BinaryOp { lhs, op, rhs, pos } => {
                                if let ast::Operand::Literal(lhs_node) = lhs {
                                    //q.add_node(lhs_node, None);
                                }
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
