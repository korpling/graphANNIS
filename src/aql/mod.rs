pub mod ast;
pub mod normalize;
pub mod operators;
pub mod parser;

use aql::operators::edge_op::PartOfSubCorpusSpec;
use aql::operators::identical_node::IdenticalNodeSpec;
use errors::*;
use exec::nodesearch::NodeSearchSpec;
use operator::OperatorSpec;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use std::collections::BTreeMap;
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
            let mut alternatives: Vec<Conjunction> = Vec::new();
            for c in ast.into_iter() {
                let mut q = Conjunction::new();
                // collect and sort all node searches according to their start position in the text
                let mut pos_to_node: BTreeMap<
                    usize,
                    (NodeSearchSpec, Option<String>),
                > = BTreeMap::default();

                let mut legacy_meta_search: Vec<(NodeSearchSpec, ast::Pos)> = Vec::new();

                for f in c.iter() {
                    if let ast::Factor::Literal(literal) = f {
                        match literal {
                            ast::Literal::NodeSearch {
                                spec,
                                pos,
                                variable,
                            } => {
                                if let Some(pos) = pos {
                                    pos_to_node.insert(pos.start, (spec.clone(), variable.clone()));
                                }
                            }
                            ast::Literal::BinaryOp { lhs, rhs, .. } => {
                                if let ast::Operand::Literal {
                                    spec,
                                    pos,
                                    variable,
                                } = lhs
                                {
                                    pos_to_node.entry(pos.start).or_insert_with(|| {
                                        (spec.as_ref().clone(), variable.clone())
                                    });
                                }
                                if let ast::Operand::Literal {
                                    spec,
                                    pos,
                                    variable,
                                } = rhs
                                {
                                    pos_to_node.entry(pos.start).or_insert_with(|| {
                                        (spec.as_ref().clone(), variable.clone())
                                    });
                                }
                            }
                            ast::Literal::LegacyMetaSearch { spec, pos } => {
                                legacy_meta_search.push((spec.clone(), pos.clone()));
                            }
                        };
                    }
                }

                // add all nodes specs in order of their start position
                let mut first_node_pos: Option<String> = None;

                let mut pos_to_node_id: HashMap<usize, String> = HashMap::default();
                for (start_pos, (node_spec, variable)) in pos_to_node.into_iter() {
                    let variable = variable.as_ref().map(|s| &**s);
                    let idx = q.add_node(node_spec, variable);
                    pos_to_node_id.insert(start_pos, idx.clone());
                    if first_node_pos.is_none() {
                        first_node_pos = Some(idx);
                    }
                }

                // add all legacy meta searches
                {
                    let mut first_meta_idx: Option<String> = None;
                    // TODO: add warning to the user not to use this construct anymore
                    for (spec, _pos) in legacy_meta_search.into_iter() {
                        // add an artificial node that describes the document/corpus node
                        let meta_node_idx = q.add_node(spec, None);
                        if let Some(first_meta_idx) = first_meta_idx.clone() {
                            // avoid nested loops by joining additional meta nodes with a "identical node"
                            q.add_operator(
                                Box::new(IdenticalNodeSpec {}),
                                &first_meta_idx,
                                &meta_node_idx,
                            )?;
                        } else if let Some(first_node_pos) = first_node_pos.clone() {
                            first_meta_idx = Some(meta_node_idx.clone());
                            // add a special join to the first node of the query
                            q.add_operator(
                                Box::new(PartOfSubCorpusSpec {
                                    min_dist: 1,
                                    max_dist: usize::max_value(),
                                }),
                                &first_node_pos,
                                &meta_node_idx,
                            )?;
                            // Also make sure the matched node is actually a document
                            // (the @* could match anything in the hierarchy, including the toplevel corpus)
                            let doc_anno_idx = q.add_node(
                                NodeSearchSpec::ExactValue {
                                    ns: Some("annis".to_string()),
                                    name: "doc".to_string(),
                                    val: None,
                                    is_meta: true,
                                },
                                None,
                            );
                            q.add_operator(
                                Box::new(IdenticalNodeSpec {}),
                                &meta_node_idx,
                                &doc_anno_idx,
                            )?;
                        }
                    }
                }

                // finally add all operators

                for f in c.into_iter() {
                    if let ast::Factor::Literal(literal) = f {
                        if let ast::Literal::BinaryOp { lhs, op, rhs, .. } = literal {
                            let idx_left = match lhs {
                                ast::Operand::Literal { spec, pos, .. } => pos_to_node_id
                                    .entry(pos.start)
                                    .or_insert_with(|| q.add_node(spec.as_ref().clone(), None))
                                    .clone(),
                                ast::Operand::NodeRef(node_ref) => match node_ref {
                                    ast::NodeRef::ID(id) => id.to_string(),
                                    ast::NodeRef::Name(name) => name,
                                },
                            };

                            let idx_right = match rhs {
                                ast::Operand::Literal { spec, pos, .. } => pos_to_node_id
                                    .entry(pos.start)
                                    .or_insert_with(|| q.add_node(spec.as_ref().clone(), None))
                                    .clone(),
                                ast::Operand::NodeRef(node_ref) => match node_ref {
                                    ast::NodeRef::ID(id) => id.to_string(),
                                    ast::NodeRef::Name(name) => name,
                                },
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
