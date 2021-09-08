mod ast;
pub mod conjunction;
pub mod disjunction;
pub mod model;
pub mod operators;

use boolean_expression::Expr;
lalrpop_mod!(
    #[allow(clippy::all)]
    parser,
    "/annis/db/aql/parser.rs"
);

use crate::annis::db::aql::conjunction::Conjunction;
use crate::annis::db::aql::disjunction::Disjunction;
use crate::annis::db::aql::operators::{
    EqualValueSpec, IdenticalNodeSpec, NegatedOpSpec, PartOfSubCorpusSpec, RangeSpec,
};
use crate::annis::db::exec::nodesearch::NodeSearchSpec;
use crate::annis::errors::*;
use crate::annis::operator::{BinaryOperatorSpec, UnaryOperatorSpec};
use crate::annis::types::{LineColumn, LineColumnRange};
use lalrpop_util::ParseError;
use std::collections::BTreeMap;
use std::collections::HashMap;

thread_local! {
    static AQL_PARSER: parser::DisjunctionParser = parser::DisjunctionParser::new();
}

#[derive(Clone, Default, Debug)]
pub struct Config {
    pub use_parallel_joins: bool,
}

fn map_conjunction(
    c: Vec<ast::Literal>,
    offsets: &BTreeMap<usize, usize>,
    var_idx_offset: usize,
    quirks_mode: bool,
) -> Result<Conjunction> {
    let mut q = Conjunction::with_offset(var_idx_offset);
    // collect and sort all node searches according to their start position in the text
    let (pos_to_node, pos_to_endpos) = calculate_node_positions(&c, offsets, quirks_mode)?;

    // add all nodes specs in order of their start position
    let mut pos_to_node_id = add_node_specs_by_start(&mut q, pos_to_node, pos_to_endpos, offsets)?;

    // add all unary operators as filter(s) to the referenced nodes
    for literal in c.iter() {
        if let ast::Literal::UnaryOp { node_ref, op, pos } = literal {
            let var = match node_ref {
                ast::NodeRef::ID(id) => id.to_string(),
                ast::NodeRef::Name(name) => name.clone(),
            };

            let op_pos: Option<LineColumnRange> = pos.as_ref().map(|pos| LineColumnRange {
                start: get_line_and_column_for_pos(pos.start, offsets),
                end: Some(get_line_and_column_for_pos(pos.end, offsets)),
            });

            q.add_unary_operator_from_query(make_unary_operator_spec(op.clone()), &var, op_pos)?;
        }
    }

    let mut num_pointing_or_dominance_joins: HashMap<String, usize> = HashMap::default();

    // finally add all binary operators
    for literal in c {
        if let ast::Literal::BinaryOp {
            lhs,
            mut op,
            rhs,
            pos,
            negated,
        } = literal
        {
            let var_left = match lhs {
                ast::Operand::Literal { spec, pos, .. } => pos_to_node_id
                    .entry(pos.start)
                    .or_insert_with(|| q.add_node(spec.as_ref().clone(), None))
                    .clone(),
                ast::Operand::NodeRef(node_ref) => match node_ref {
                    ast::NodeRef::ID(id) => id.to_string(),
                    ast::NodeRef::Name(name) => name,
                },
            };

            let var_right = match rhs {
                ast::Operand::Literal { spec, pos, .. } => pos_to_node_id
                    .entry(pos.start)
                    .or_insert_with(|| q.add_node(spec.as_ref().clone(), None))
                    .clone(),
                ast::Operand::NodeRef(node_ref) => match node_ref {
                    ast::NodeRef::ID(id) => id.to_string(),
                    ast::NodeRef::Name(name) => name,
                },
            };

            let op_pos: Option<LineColumnRange> = pos.map(|pos| LineColumnRange {
                start: get_line_and_column_for_pos(pos.start, offsets),
                end: Some(get_line_and_column_for_pos(pos.end, offsets)),
            });

            let spec_left = q.resolve_variable(&var_left, op_pos.clone())?;
            let spec_right = q.resolve_variable(&var_right, op_pos.clone())?;

            if quirks_mode {
                match op {
                    ast::BinaryOpSpec::Dominance(_) | ast::BinaryOpSpec::Pointing(_) => {
                        let entry_lhs = num_pointing_or_dominance_joins
                            .entry(var_left.clone())
                            .or_insert(0);
                        *entry_lhs += 1;
                        let entry_rhs = num_pointing_or_dominance_joins
                            .entry(var_right.clone())
                            .or_insert(0);
                        *entry_rhs += 1;
                    }
                    ast::BinaryOpSpec::Precedence(ref mut spec) => {
                        // limit unspecified .* precedence to 50
                        spec.dist = if let RangeSpec::Unbound = spec.dist {
                            RangeSpec::Bound {
                                min_dist: 1,
                                max_dist: 50,
                            }
                        } else {
                            spec.dist.clone()
                        };
                    }
                    ast::BinaryOpSpec::Near(ref mut spec) => {
                        // limit unspecified ^* near-by operator to 50
                        spec.dist = if let RangeSpec::Unbound = spec.dist {
                            RangeSpec::Bound {
                                min_dist: 1,
                                max_dist: 50,
                            }
                        } else {
                            spec.dist.clone()
                        };
                    }
                    _ => {}
                }
            }
            let mut op_spec = make_binary_operator_spec(op, spec_left.clone(), spec_right.clone())?;
            if negated {
                op_spec = Box::new(NegatedOpSpec {
                    spec_left,
                    spec_right,
                    negated_op: op_spec,
                });
            }
            q.add_operator_from_query(op_spec, &var_left, &var_right, op_pos, !quirks_mode)?;
        }
    }

    if quirks_mode {
        // Add additional nodes to the query to emulate the old behavior of distributing
        // joins for pointing and dominance operators on different query nodes.
        // Iterate over the query nodes in their order as given by the query.
        for (_, orig_var) in pos_to_node_id.iter() {
            let num_joins = num_pointing_or_dominance_joins.get(orig_var).unwrap_or(&0);
            // add an additional node for each extra join and join this artificial node with identity relation
            for _ in 1..*num_joins {
                if let Ok(node_spec) = q.resolve_variable(orig_var, None) {
                    let new_var = q.add_node(node_spec, None);
                    q.add_operator(Box::new(IdenticalNodeSpec {}), orig_var, &new_var, false)?;
                }
            }
        }
    }

    Ok(q)
}

type PosToNodeMap = BTreeMap<usize, (NodeSearchSpec, Option<String>)>;
type PosToEndPosMap = BTreeMap<usize, usize>;

fn calculate_node_positions(
    c: &[ast::Literal],
    offsets: &BTreeMap<usize, usize>,
    quirks_mode: bool,
) -> Result<(PosToNodeMap, PosToEndPosMap)> {
    let mut pos_to_node = BTreeMap::default();
    let mut pos_to_endpos = BTreeMap::default();

    for literal in c {
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
                    pos_to_node
                        .entry(pos.start)
                        .or_insert_with(|| (spec.as_ref().clone(), variable.clone()));
                    pos_to_endpos.entry(pos.start).or_insert_with(|| pos.end);
                }
                if let ast::Operand::Literal {
                    spec,
                    pos,
                    variable,
                } = rhs
                {
                    pos_to_node
                        .entry(pos.start)
                        .or_insert_with(|| (spec.as_ref().clone(), variable.clone()));
                    pos_to_endpos.entry(pos.start).or_insert_with(|| pos.end);
                }
            }
            ast::Literal::UnaryOp { .. } => {
                // can only have node reference, not a literal
            }
            ast::Literal::LegacyMetaSearch { pos, .. } => {
                if !quirks_mode {
                    let start = get_line_and_column_for_pos(pos.start, offsets);
                    let end = Some(get_line_and_column_for_pos(
                        pos.start + "meta::".len() - 1,
                        offsets,
                    ));
                    return Err(GraphAnnisError::AQLSyntaxError( AQLError {
                        desc: "Legacy metadata search is no longer allowed. Use the @* operator and normal attribute search instead.".into(),
                        location: Some(LineColumnRange {start, end}),
                    }));
                }
            }
        };
    }

    Ok((pos_to_node, pos_to_endpos))
}

fn add_node_specs_by_start(
    q: &mut Conjunction,
    pos_to_node: BTreeMap<usize, (NodeSearchSpec, Option<String>)>,
    pos_to_endpos: BTreeMap<usize, usize>,
    offsets: &BTreeMap<usize, usize>,
) -> Result<BTreeMap<usize, String>> {
    let mut pos_to_node_id: BTreeMap<usize, String> = BTreeMap::default();
    for (start_pos, (node_spec, variable)) in pos_to_node {
        let start = get_line_and_column_for_pos(start_pos, offsets);
        let end = pos_to_endpos
            .get(&start_pos)
            .map(|end_pos| get_line_and_column_for_pos(*end_pos, offsets));

        let idx = q.add_node_from_query(
            node_spec,
            variable.as_deref(),
            Some(LineColumnRange { start, end }),
            true,
        );
        pos_to_node_id.insert(start_pos, idx.clone());
    }

    Ok(pos_to_node_id)
}

fn add_legacy_metadata_constraints(
    q: &mut Conjunction,
    legacy_meta_search: Vec<(NodeSearchSpec, ast::Pos)>,
    first_node_pos: Option<String>,
) -> Result<()> {
    {
        let mut first_meta_idx: Option<String> = None;
        // TODO: add warning to the user not to use this construct anymore
        for (spec, _pos) in legacy_meta_search {
            // add an artificial node that describes the document/corpus node
            let meta_node_idx = q.add_node_from_query(spec, None, None, false);
            if let Some(first_meta_idx) = first_meta_idx.clone() {
                // avoid nested loops by joining additional meta nodes with a "identical node"
                q.add_operator(
                    Box::new(IdenticalNodeSpec {}),
                    &first_meta_idx,
                    &meta_node_idx,
                    true,
                )?;
            } else if let Some(first_node_pos) = first_node_pos.clone() {
                first_meta_idx = Some(meta_node_idx.clone());
                // add a special join to the first node of the query
                q.add_operator(
                    Box::new(PartOfSubCorpusSpec {
                        dist: RangeSpec::Unbound,
                    }),
                    &first_node_pos,
                    &meta_node_idx,
                    true,
                )?;
                // Also make sure the matched node is actually a document
                // (the @* could match anything in the hierarchy, including the toplevel corpus)
                let doc_anno_idx = q.add_node_from_query(
                    NodeSearchSpec::ExactValue {
                        ns: Some("annis".to_string()),
                        name: "doc".to_string(),
                        val: None,
                        is_meta: true,
                    },
                    None,
                    None,
                    false,
                );
                q.add_operator(
                    Box::new(IdenticalNodeSpec {}),
                    &meta_node_idx,
                    &doc_anno_idx,
                    true,
                )?;
            }
        }
    }
    Ok(())
}

fn find_all_children_for_and(expr: &ast::Expr, followers: &mut Vec<ast::Literal>) {
    match expr {
        Expr::Terminal(l) => {
            followers.push(l.clone());
        }
        Expr::And(lhs, rhs) => {
            find_all_children_for_and(lhs, followers);
            find_all_children_for_and(rhs, followers);
        }
        _ => {}
    }
}

fn find_all_children_for_or(expr: &ast::Expr, followers: &mut Vec<ast::Expr>) {
    match expr {
        Expr::Or(lhs, rhs) => {
            find_all_children_for_or(lhs, followers);
            find_all_children_for_or(rhs, followers);
        }
        _ => {
            // add the expression itself
            followers.push(expr.clone());
        }
    }
}

fn get_alternatives_from_dnf(expr: ast::Expr) -> Vec<Vec<ast::Literal>> {
    if expr.is_and() {
        let mut followers = Vec::new();
        find_all_children_for_and(&expr, &mut followers);
        return vec![followers];
    } else if expr.is_or() {
        let mut non_or_roots = Vec::new();
        find_all_children_for_or(&expr, &mut non_or_roots);

        let mut result = Vec::new();
        for root in non_or_roots {
            if root.is_and() {
                let mut followers = Vec::new();
                find_all_children_for_and(&root, &mut followers);
                result.push(followers);
            } else if let Expr::Terminal(t) = root {
                result.push(vec![t]);
            }
        }
        return result;
    } else if let Expr::Terminal(t) = expr {
        return vec![vec![t]];
    }
    vec![]
}

pub fn parse<'a>(query_as_aql: &str, quirks_mode: bool) -> Result<Disjunction> {
    let ast = AQL_PARSER.with(|p| p.parse(query_as_aql));
    match ast {
        Ok(ast) => {
            let offsets = get_line_offsets(query_as_aql);

            // make sure AST is in DNF
            let ast: ast::Expr = ast.simplify_via_laws();
            let ast = get_alternatives_from_dnf(ast);

            let mut legacy_meta_search: Vec<(NodeSearchSpec, ast::Pos)> = Vec::new();
            if quirks_mode {
                for conjunction in &ast {
                    for literal in conjunction {
                        if let ast::Literal::LegacyMetaSearch { spec, pos } = literal {
                            legacy_meta_search.push((spec.clone(), pos.clone()));
                        }
                    }
                }
            }

            // map all conjunctions and its literals
            let mut alternatives: Vec<Conjunction> = Vec::new();
            let mut var_idx_offset = 0;
            for c in ast {
                // add the conjunction to the disjunction
                let mut mapped = map_conjunction(c, &offsets, var_idx_offset, quirks_mode)?;

                if quirks_mode {
                    // apply the meta constraints from all conjunctions to conjunctions
                    let first_node_pos = mapped.get_variable_by_pos(0);
                    add_legacy_metadata_constraints(
                        &mut mapped,
                        legacy_meta_search.clone(),
                        first_node_pos,
                    )?;
                }
                var_idx_offset += mapped.num_of_nodes();

                alternatives.push(mapped);
            }

            Ok(Disjunction::new(alternatives))
        }
        Err(e) => {
            let mut desc = match e {
                ParseError::InvalidToken { .. } => "Invalid token detected.",
                ParseError::ExtraToken { .. } => "Extra token at end of query.",
                ParseError::UnrecognizedToken { .. } => "Unexpected token in query.",
                ParseError::UnrecognizedEOF { .. } => "Unexpected end of query.",
                ParseError::User { error } => error,
            }
            .to_string();
            let location = extract_location(&e, query_as_aql);
            if let ParseError::UnrecognizedToken { expected, .. } = e {
                if !expected.is_empty() {
                    //TODO: map token regular expressions and IDs (like IDENT_NODE) to human readable descriptions
                    desc.push_str(" Expected one of: ");
                    desc.push_str(&expected.join(","));
                }
            }
            Err(GraphAnnisError::AQLSyntaxError(AQLError { desc, location }))
        }
    }
}
fn make_binary_operator_spec(
    op: ast::BinaryOpSpec,
    spec_left: NodeSearchSpec,
    spec_right: NodeSearchSpec,
) -> Result<Box<dyn BinaryOperatorSpec>> {
    let op_spec: Box<dyn BinaryOperatorSpec> = match op {
        ast::BinaryOpSpec::Dominance(spec) => Box::new(spec),
        ast::BinaryOpSpec::Pointing(spec) => Box::new(spec),
        ast::BinaryOpSpec::Precedence(spec) => Box::new(spec),
        ast::BinaryOpSpec::Near(spec) => Box::new(spec),
        ast::BinaryOpSpec::Overlap(spec) => Box::new(spec),
        ast::BinaryOpSpec::IdenticalCoverage(spec) => Box::new(spec),
        ast::BinaryOpSpec::PartOfSubCorpus(spec) => Box::new(spec),
        ast::BinaryOpSpec::Inclusion(spec) => Box::new(spec),
        ast::BinaryOpSpec::LeftAlignment(spec) => Box::new(spec),
        ast::BinaryOpSpec::RightAlignment(spec) => Box::new(spec),
        ast::BinaryOpSpec::IdenticalNode(spec) => Box::new(spec),
        ast::BinaryOpSpec::ValueComparison(cmp) => match cmp {
            ast::ComparisonOperator::Equal => Box::new(EqualValueSpec {
                spec_left,
                spec_right,
                negated: false,
            }),
            ast::ComparisonOperator::NotEqual => Box::new(EqualValueSpec {
                spec_left,
                spec_right,
                negated: true,
            }),
        },
    };
    Ok(op_spec)
}

fn make_unary_operator_spec(op: ast::UnaryOpSpec) -> Box<dyn UnaryOperatorSpec> {
    match op {
        ast::UnaryOpSpec::Arity(spec) => Box::new(spec),
    }
}

fn get_line_offsets(input: &str) -> BTreeMap<usize, usize> {
    let mut offsets = BTreeMap::default();

    let mut o = 0;
    let mut l = 1;
    for line in input.split('\n') {
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
        .range(..=pos)
        .rev()
        .map(|(offset, line)| {
            // column starts with 1 at line offset
            let column: usize = pos - offset + 1;
            LineColumn {
                line: *line,
                column,
            }
        })
        .next()
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
            let start = get_line_and_column_for_pos(token.0, &offsets);
            let end = get_line_and_column_for_pos(token.2 - 1, &offsets);
            Some(LineColumnRange {
                start,
                end: Some(end),
            })
        }
        ParseError::UnrecognizedEOF { .. } => {
            // set to end of query
            let start = get_line_and_column_for_pos(input.len() - 1, &offsets);
            Some(LineColumnRange { start, end: None })
        }
        ParseError::User { .. } => None,
    };
    from_to
}
