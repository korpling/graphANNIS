mod ast;
mod normalize;
pub mod operators;
lalrpop_mod!(#[allow(clippy)] parser, "/annis/db/aql/parser.rs");

use annis::db::aql::operators::edge_op::PartOfSubCorpusSpec;
use annis::db::aql::operators::identical_node::IdenticalNodeSpec;
use annis::db::exec::nodesearch::NodeSearchSpec;
use annis::db::query::conjunction::Conjunction;
use annis::db::query::disjunction::Disjunction;
use annis::errors::*;
use annis::operator::OperatorSpec;
use annis::types::{LineColumn, LineColumnRange};
use lalrpop_util::ParseError;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub fn parse<'a>(query_as_aql: &str) -> Result<Disjunction<'a>> {
    let ast = parser::DisjunctionParser::new().parse(query_as_aql);
    match ast {
        Ok(mut ast) => {
            let offsets = get_line_offsets(query_as_aql);

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

                let mut pos_to_endpos: BTreeMap<usize, usize> = BTreeMap::default();

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
                                    pos_to_endpos.insert(pos.start, pos.end);
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
                                    pos_to_endpos
                                        .entry(pos.start)
                                        .or_insert_with(|| pos.end);
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
                                    pos_to_endpos
                                        .entry(pos.start)
                                        .or_insert_with(|| pos.end);
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

                    let start = get_line_and_column_for_pos(start_pos, &offsets);
                    let end = if let Some(end_pos) = pos_to_endpos.get(&start_pos) {
                        Some(get_line_and_column_for_pos(*end_pos, &offsets))
                    } else {
                        None
                    };

                    let idx = q.add_node_from_query(
                        node_spec,
                        variable,
                        Some(LineColumnRange { start, end }),
                    );
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
                        if let ast::Literal::BinaryOp { lhs, op, rhs, pos } = literal {
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

                            let op_pos: Option<LineColumnRange> = if let Some(pos) = pos {
                                Some(LineColumnRange {
                                    start: get_line_and_column_for_pos(pos.start, &offsets),
                                    end: Some(get_line_and_column_for_pos(pos.end, &offsets)),
                                })
                            } else {
                                None
                            };

                            q.add_operator_from_query(
                                make_operator_spec(op),
                                &idx_left,
                                &idx_right,
                                op_pos,
                            )?;
                        }
                    }
                }

                // add the conjunction to the disjunction
                alternatives.push(q);
            }
            Ok(Disjunction::new(alternatives))
        }
        Err(e) => {
            let mut desc = match e {
                ParseError::InvalidToken { .. } => "Invalid token detected.",
                ParseError::ExtraToken { .. } => "Extra token at end of query.",
                ParseError::UnrecognizedToken { .. } => "Unexpected token in query.",
                ParseError::User { error } => error,
            }.to_string();
            let location = extract_location(&e, query_as_aql);
            match e {
                ParseError::UnrecognizedToken { expected, .. } => {
                    if !expected.is_empty() {
                        //TODO: map token regular expressions and IDs (like IDENT_NODE) to human readable descriptions
                        desc.push_str("Expected one of: ");
                        desc.push_str(&expected.join(","));
                    }
                }
                _ => {}
            };
            Err(ErrorKind::AQLSyntaxError(desc, location).into())
        }
    }
}

fn make_operator_spec(op: ast::BinaryOpSpec) -> Box<OperatorSpec> {
    match op {
        ast::BinaryOpSpec::Dominance(spec) => Box::new(spec),
        ast::BinaryOpSpec::Pointing(spec) => Box::new(spec),
        ast::BinaryOpSpec::Precedence(spec) => Box::new(spec),
        ast::BinaryOpSpec::Overlap(spec) => Box::new(spec),
        ast::BinaryOpSpec::IdenticalCoverage(spec) => Box::new(spec),
        ast::BinaryOpSpec::PartOfSubCorpus(spec) => Box::new(spec),
        ast::BinaryOpSpec::Inclusion(spec) => Box::new(spec),
        ast::BinaryOpSpec::IdenticalNode(spec) => Box::new(spec),
    }
}

fn get_line_offsets(input: &str) -> BTreeMap<usize, usize> {
    let mut offsets = BTreeMap::default();

    let mut o = 0;
    let mut l = 1;
    for line in input.split("\n") {
        offsets.insert(o, l);
        o += line.len() + 1;
        l += 1;
    }

    offsets
}

pub fn get_line_and_column_for_pos(
    pos: usize,
    offset_to_line: &BTreeMap<usize, usize>,
) -> LineColumn {
    // get the offset for the position by searching for all offsets smaller than the position and taking the last one
    offset_to_line
        .range(..pos + 1)
        .rev()
        .map(|(offset, line)| {
            // column starts with 1 at line offset
            let column: usize = pos - offset + 1;
            LineColumn {
                line: *line,
                column,
            }
        }).next()
        .unwrap_or(LineColumn { line: 0, column: 0 })
}

fn extract_location<'a>(
    e: &ParseError<usize, parser::Token<'a>, &'static str>,
    input: &'a str,
) -> Option<LineColumnRange> {
    let offsets = get_line_offsets(input);

    let from_to: Option<LineColumnRange> = match e {
        ParseError::InvalidToken { location } => Some(LineColumnRange {
            start: get_line_and_column_for_pos(*location, &offsets),
            end: None,
        }),
        ParseError::ExtraToken { token } => {
            let start = get_line_and_column_for_pos(token.0, &offsets);
            let end = get_line_and_column_for_pos(token.2 - 1, &offsets);
            Some(LineColumnRange {
                start,
                end: Some(end),
            })
        }
        ParseError::UnrecognizedToken { token, .. } => {
            if let Some(token) = token {
                let start = get_line_and_column_for_pos(token.0, &offsets);
                let end = get_line_and_column_for_pos(token.2 - 1, &offsets);
                Some(LineColumnRange {
                    start,
                    end: Some(end),
                })
            } else {
                // set to end of query
                let start = get_line_and_column_for_pos(input.len() - 1, &offsets);
                Some(LineColumnRange { start, end: None })
            }
        }
        ParseError::User { .. } => None,
    };
    from_to
}
