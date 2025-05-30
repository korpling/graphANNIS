use super::MatchValueFilterFunc;
use super::{ExecutionNode, ExecutionNodeDesc, NodeSearchDesc};
use crate::annis::db::aql::model::AnnotationComponentType;
use crate::annis::db::exec::tokensearch;
use crate::annis::db::exec::tokensearch::AnyTokenSearch;
use crate::annis::errors::*;
use crate::annis::operator::EdgeAnnoSearchSpec;
use crate::annis::types::LineColumnRange;
use crate::annis::util::TimeoutCheck;
use crate::AnnotationGraph;
use crate::{
    annis::{db::aql::model::TOKEN_KEY, util},
    graph::Match,
};
use graphannis_core::annostorage::EdgeAnnotationStorage;
use graphannis_core::errors::GraphAnnisCoreError;
use graphannis_core::graph::{ANNIS_NS, NODE_NAME};
use graphannis_core::{
    annostorage::{MatchGroup, ValueSearch},
    graph::{storage::GraphStorage, NODE_TYPE_KEY},
    types::{Component, NodeID},
};
use itertools::Itertools;
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<dyn Iterator<Item = Result<MatchGroup>> + 'a>,

    desc: Option<ExecutionNodeDesc>,
    node_search_desc: Arc<NodeSearchDesc>,
    is_sorted: bool,
    timeout: TimeoutCheck,
}
struct CommonArguments<'a> {
    query_fragment: String,
    node_nr: usize,
    output_size_cache: &'a mut HashMap<usize, usize>,
    timeout: TimeoutCheck,
}

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum NodeSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
        is_meta: bool,
    },
    NotExactValue {
        ns: Option<String>,
        name: String,
        val: String,
        is_meta: bool,
    },
    RegexValue {
        ns: Option<String>,
        name: String,
        val: String,
        is_meta: bool,
    },
    NotRegexValue {
        ns: Option<String>,
        name: String,
        val: String,
        is_meta: bool,
    },
    ExactTokenValue {
        val: String,
        leafs_only: bool,
    },
    NotExactTokenValue {
        val: String,
    },
    RegexTokenValue {
        val: String,
        leafs_only: bool,
    },
    NotRegexTokenValue {
        val: String,
    },
    AnyToken,
    AnyNode,
}

impl NodeSearchSpec {
    pub fn necessary_components(
        &self,
        db: &AnnotationGraph,
    ) -> HashSet<Component<AnnotationComponentType>> {
        match self {
            NodeSearchSpec::AnyToken => tokensearch::AnyTokenSearch::necessary_components(db),
            NodeSearchSpec::ExactTokenValue {
                leafs_only: true, ..
            }
            | NodeSearchSpec::RegexTokenValue {
                leafs_only: true, ..
            }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. } => db
                .get_all_components(Some(AnnotationComponentType::Coverage), None)
                .into_iter()
                .collect(),
            _ => HashSet::default(),
        }
    }

    /// Get the annotatiom qualified name needed to execute a search with this specification.
    pub fn get_anno_qname(&self) -> (Option<String>, Option<String>) {
        match self {
            NodeSearchSpec::ExactValue { ns, name, .. }
            | NodeSearchSpec::NotExactValue { ns, name, .. }
            | NodeSearchSpec::RegexValue { ns, name, .. }
            | NodeSearchSpec::NotRegexValue { ns, name, .. } => {
                (ns.to_owned(), Some(name.to_owned()))
            }
            NodeSearchSpec::ExactTokenValue { .. }
            | NodeSearchSpec::NotExactTokenValue { .. }
            | NodeSearchSpec::RegexTokenValue { .. }
            | NodeSearchSpec::NotRegexTokenValue { .. }
            | NodeSearchSpec::AnyToken => (
                Some(TOKEN_KEY.ns.clone().into()),
                Some(TOKEN_KEY.name.clone().into()),
            ),
            NodeSearchSpec::AnyNode => (
                Some(NODE_TYPE_KEY.ns.clone().into()),
                Some(NODE_TYPE_KEY.name.clone().into()),
            ),
        }
    }

    /// Creates a vector of value filter functions for this node annotation search.
    pub fn get_value_filter(
        &self,
        g: &AnnotationGraph,
        location_in_query: Option<LineColumnRange>,
    ) -> Result<Vec<MatchValueFilterFunc>> {
        let mut filters: Vec<MatchValueFilterFunc> = Vec::new();
        match self {
            NodeSearchSpec::ExactValue { val, .. } => {
                if let Some(val) = val {
                    let val: String = val.to_owned();
                    filters.push(Box::new(move |m, node_annos| {
                        if let Some(anno_val) =
                            node_annos.get_value_for_item(&m.node, &m.anno_key)?
                        {
                            Ok(anno_val == val.as_str())
                        } else {
                            Ok(false)
                        }
                    }));
                }
            }
            NodeSearchSpec::NotExactValue { val, .. } => {
                let val: String = val.to_owned();
                filters.push(Box::new(move |m, node_annos| {
                    if let Some(anno_val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                        Ok(anno_val != val.as_str())
                    } else {
                        Ok(false)
                    }
                }));
            }
            NodeSearchSpec::RegexValue { val, .. } => {
                let full_match_pattern = graphannis_core::util::regex_full_match(val);
                let re = regex::Regex::new(&full_match_pattern);
                match re {
                    Ok(re) => {
                        filters.push(Box::new(move |m, node_annos| {
                            if let Some(val) =
                                node_annos.get_value_for_item(&m.node, &m.anno_key)?
                            {
                                Ok(re.is_match(&val))
                            } else {
                                Ok(false)
                            }
                        }));
                    }
                    Err(e) => {
                        return Err(GraphAnnisError::AQLSemanticError(AQLError {
                            desc: format!("/{}/ -> {}", val, e),
                            location: location_in_query,
                        }));
                    }
                }
            }
            NodeSearchSpec::NotRegexValue { val, .. } => {
                let full_match_pattern = graphannis_core::util::regex_full_match(val);
                let re = regex::Regex::new(&full_match_pattern);
                match re {
                    Ok(re) => {
                        filters.push(Box::new(move |m, node_annos| {
                            if let Some(val) =
                                node_annos.get_value_for_item(&m.node, &m.anno_key)?
                            {
                                Ok(!re.is_match(&val))
                            } else {
                                Ok(false)
                            }
                        }));
                    }
                    Err(e) => {
                        return Err(GraphAnnisError::AQLSemanticError(AQLError {
                            desc: format!("/{}/ -> {}", val, e),
                            location: location_in_query,
                        }));
                    }
                }
            }
            NodeSearchSpec::ExactTokenValue { val, leafs_only } => {
                let val = val.clone();
                filters.push(Box::new(move |m, node_annos| {
                    if let Some(anno_val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                        Ok(anno_val == val.as_str())
                    } else {
                        Ok(false)
                    }
                }));
                if *leafs_only {
                    filters.push(create_token_leaf_filter(g));
                }
            }
            NodeSearchSpec::NotExactTokenValue { val } => {
                let val = val.clone();
                filters.push(Box::new(move |m, node_annos| {
                    if let Some(anno_val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                        Ok(anno_val != val.as_str())
                    } else {
                        Ok(false)
                    }
                }));
                filters.push(create_token_leaf_filter(g));
            }
            NodeSearchSpec::RegexTokenValue { val, leafs_only } => {
                let full_match_pattern = graphannis_core::util::regex_full_match(val);
                let re = regex::Regex::new(&full_match_pattern);
                match re {
                    Ok(re) => filters.push(Box::new(move |m, node_annos| {
                        if let Some(val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                            Ok(re.is_match(&val))
                        } else {
                            Ok(false)
                        }
                    })),
                    Err(e) => {
                        return Err(GraphAnnisError::AQLSemanticError(AQLError {
                            desc: format!("/{}/ -> {}", val, e),
                            location: location_in_query,
                        }));
                    }
                };
                if *leafs_only {
                    filters.push(create_token_leaf_filter(g));
                }
            }
            NodeSearchSpec::NotRegexTokenValue { val } => {
                let full_match_pattern = graphannis_core::util::regex_full_match(val);
                let re = regex::Regex::new(&full_match_pattern);
                match re {
                    Ok(re) => filters.push(Box::new(move |m, node_annos| {
                        if let Some(val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                            Ok(!re.is_match(&val))
                        } else {
                            Ok(false)
                        }
                    })),
                    Err(e) => {
                        return Err(GraphAnnisError::AQLSemanticError(AQLError {
                            desc: format!("/{}/ -> {}", val, e),
                            location: location_in_query,
                        }));
                    }
                };
                filters.push(create_token_leaf_filter(g));
            }
            NodeSearchSpec::AnyToken => {
                filters.push(create_token_leaf_filter(g));
            }
            NodeSearchSpec::AnyNode => {
                let filter_func: MatchValueFilterFunc = Box::new(move |m, node_annos| {
                    if let Some(val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                        Ok(val == "node")
                    } else {
                        Ok(false)
                    }
                });
                filters.push(filter_func)
            }
        }

        Ok(filters)
    }
}

impl fmt::Display for NodeSearchSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeSearchSpec::ExactValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => {
                if let (Some(ns), Some(val)) = (ns, val) {
                    write!(f, "{}:{}=\"{}\"", ns, name, val)
                } else if let Some(ns) = ns {
                    write!(f, "{}:{}", ns, name)
                } else if let Some(val) = val {
                    write!(f, "{}=\"{}\"", name, val)
                } else {
                    write!(f, "{}", name)
                }
            }
            NodeSearchSpec::NotExactValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => {
                if let Some(ref ns) = ns {
                    write!(f, "{}:{}!=\"{}\"", ns, name, &val)
                } else {
                    write!(f, "{}!=\"{}\"", name, &val)
                }
            }
            NodeSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => {
                if let Some(ns) = ns {
                    write!(f, "{}:{}=/{}/", ns, name, &val)
                } else {
                    write!(f, "{}=/{}/", name, &val)
                }
            }
            NodeSearchSpec::NotRegexValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => {
                if let Some(ref ns) = ns {
                    write!(f, "{}:{}!=/{}/", ns, name, &val)
                } else {
                    write!(f, "{}!=/{}/", name, &val)
                }
            }
            NodeSearchSpec::ExactTokenValue {
                ref val,
                ref leafs_only,
            } => {
                if *leafs_only {
                    write!(f, "tok=\"{}\"", val)
                } else {
                    write!(f, "\"{}\"", val)
                }
            }
            NodeSearchSpec::NotExactTokenValue { ref val } => write!(f, "tok!=\"{}\"", val),
            NodeSearchSpec::RegexTokenValue {
                ref val,
                ref leafs_only,
            } => {
                if *leafs_only {
                    write!(f, "tok=/{}/", val)
                } else {
                    write!(f, "/{}/", val)
                }
            }
            NodeSearchSpec::NotRegexTokenValue { ref val } => write!(f, "tok!=/{}/", val),
            NodeSearchSpec::AnyToken => write!(f, "tok"),
            NodeSearchSpec::AnyNode => write!(f, "node"),
        }
    }
}

fn create_token_leaf_filter(g: &AnnotationGraph) -> MatchValueFilterFunc {
    let cov_gs: Vec<Arc<dyn GraphStorage>> = g
        .get_all_components(Some(AnnotationComponentType::Coverage), None)
        .into_iter()
        .filter_map(|c| g.get_graphstorage(&c))
        .filter(|gs| {
            if let Some(stats) = gs.get_statistics() {
                stats.nodes > 0
            } else {
                true
            }
        })
        .collect();

    let filter_func: MatchValueFilterFunc = Box::new(move |m, _| {
        for cov in cov_gs.iter() {
            if cov.get_outgoing_edges(m.node).next().is_some() {
                return Ok(false);
            }
        }
        Ok(true)
    });
    filter_func
}

impl<'a> NodeSearch<'a> {
    pub fn from_spec(
        spec: NodeSearchSpec,
        node_nr: usize,
        db: &'a AnnotationGraph,
        location_in_query: Option<LineColumnRange>,
        output_size_cache: &mut HashMap<usize, usize>,
        timeout: TimeoutCheck,
    ) -> Result<NodeSearch<'a>> {
        let query_fragment = format!("{}", spec);

        let filters = spec.get_value_filter(db, location_in_query)?;
        let common_args = CommonArguments {
            query_fragment,
            node_nr,
            output_size_cache,
            timeout,
        };
        match spec {
            NodeSearchSpec::ExactValue {
                ns,
                name,
                val,
                is_meta,
            } => NodeSearch::new_annosearch_exact(
                db,
                (ns, name),
                val.into(),
                filters,
                is_meta,
                common_args,
            ),
            NodeSearchSpec::NotExactValue {
                ns,
                name,
                val,
                is_meta,
            } => NodeSearch::new_annosearch_exact(
                db,
                (ns, name),
                ValueSearch::NotSome(val),
                filters,
                is_meta,
                common_args,
            ),
            NodeSearchSpec::RegexValue {
                ns,
                name,
                val,
                is_meta,
            } => {
                // check if the regex can be replaced with an exact value search
                if let Some(exaxt_value) = util::exact_value_for_regex(&val) {
                    NodeSearch::new_annosearch_exact(
                        db,
                        (ns, name),
                        ValueSearch::Some(exaxt_value),
                        filters,
                        is_meta,
                        common_args,
                    )
                } else {
                    NodeSearch::new_annosearch_regex(
                        db,
                        (ns, name),
                        &val,
                        false,
                        filters,
                        is_meta,
                        common_args,
                    )
                }
            }
            NodeSearchSpec::NotRegexValue {
                ns,
                name,
                val,
                is_meta,
            } => {
                // check if the regex can be replaced with an exact value search
                if let Some(exact_value) = util::exact_value_for_regex(&val) {
                    NodeSearch::new_annosearch_exact(
                        db,
                        (ns, name),
                        ValueSearch::NotSome(exact_value),
                        filters,
                        is_meta,
                        common_args,
                    )
                } else {
                    NodeSearch::new_annosearch_regex(
                        db,
                        (ns, name),
                        &val,
                        true,
                        filters,
                        is_meta,
                        common_args,
                    )
                }
            }
            NodeSearchSpec::ExactTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                ValueSearch::Some(val),
                filters,
                leafs_only,
                false,
                common_args,
            ),
            NodeSearchSpec::NotExactTokenValue { val } => NodeSearch::new_tokensearch(
                db,
                ValueSearch::NotSome(val),
                filters,
                true,
                false,
                common_args,
            ),
            NodeSearchSpec::RegexTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                ValueSearch::Some(val),
                filters,
                leafs_only,
                true,
                common_args,
            ),
            NodeSearchSpec::NotRegexTokenValue { val } => NodeSearch::new_tokensearch(
                db,
                ValueSearch::NotSome(val),
                filters,
                true,
                true,
                common_args,
            ),
            NodeSearchSpec::AnyToken => NodeSearch::new_anytoken_search(db, common_args),
            NodeSearchSpec::AnyNode => {
                let it = db
                    .get_node_annos()
                    .exact_anno_search(
                        Some(&NODE_TYPE_KEY.ns),
                        &NODE_TYPE_KEY.name,
                        Some("node").into(),
                    )
                    .map(|n| match n {
                        Ok(n) => Ok(smallvec![n]),
                        Err(e) => Err(e.into()),
                    });

                let filter_func: MatchValueFilterFunc = Box::new(move |m, node_annos| {
                    if let Some(val) = node_annos.get_value_for_item(&m.node, &m.anno_key)? {
                        Ok(val == "node")
                    } else {
                        Ok(false)
                    }
                });

                let est_output = db.get_node_annos().guess_max_count(
                    Some(&NODE_TYPE_KEY.ns),
                    &NODE_TYPE_KEY.name,
                    "node",
                    "node",
                )?;
                let est_output = std::cmp::max(1, est_output);

                Ok(NodeSearch {
                    it: Box::new(it),
                    desc: Some(ExecutionNodeDesc::empty_with_fragment(
                        common_args.node_nr,
                        common_args.query_fragment,
                        est_output,
                    )),
                    node_search_desc: Arc::new(NodeSearchDesc {
                        qname: (
                            Some(NODE_TYPE_KEY.ns.clone().into()),
                            Some(NODE_TYPE_KEY.name.clone().into()),
                        ),
                        cond: vec![filter_func],
                        const_output: Some(NODE_TYPE_KEY.clone()),
                    }),
                    is_sorted: false,
                    timeout,
                })
            }
        }
    }

    fn new_annosearch_exact(
        db: &'a AnnotationGraph,
        qname: (Option<String>, String),
        val: ValueSearch<String>,
        filters: Vec<MatchValueFilterFunc>,
        is_meta: bool,
        common_args: CommonArguments,
    ) -> Result<NodeSearch<'a>> {
        let base_it = db.get_node_annos().exact_anno_search(
            qname.0.as_deref(),
            &qname.1,
            val.as_ref().map(String::as_str),
        );

        let const_output = if is_meta {
            Some(NODE_TYPE_KEY.clone())
        } else {
            None
        };

        let base_it: Box<dyn Iterator<Item = Result<Match>>> =
            if let Some(const_output) = const_output.clone() {
                let is_unique = db.get_node_annos().get_qnames(&qname.1)?.len() <= 1;
                // Replace the result annotation with a constant value. If a
                // node matches two different annotations (because there is no
                // namespace), this can result in duplicates which needs to be
                // filtered out.
                if is_unique {
                    Box::new(base_it.map(move |m| match m {
                        Ok(m) => Ok(Match {
                            node: m.node,
                            anno_key: const_output.clone(),
                        }),
                        Err(e) => Err(e.into()),
                    }))
                } else {
                    Box::new(
                        base_it
                            // The unique_by will merge all errors into a single
                            // error, which should be ok for our use case
                            .unique_by(|m| match m {
                                Ok(m) => Some(m.clone()),
                                Err(_) => None,
                            })
                            .map(move |m| match m {
                                Ok(m) => Ok(Match {
                                    node: m.node,
                                    anno_key: const_output.clone(),
                                }),
                                Err(e) => Err(e.into()),
                            }),
                    )
                }
            } else {
                Box::new(base_it.map(|it| it.map_err(GraphAnnisError::from)))
            };

        let est_output =
            if let Some(cached) = common_args.output_size_cache.get(&common_args.node_nr) {
                *cached
            } else {
                match val {
                    ValueSearch::Some(ref val) => {
                        if qname.0.as_deref() == Some(ANNIS_NS) && qname.1 == NODE_NAME {
                            // Our data model assumes that annis::node_name annotations are unique
                            1
                        } else {
                            db.get_node_annos().guess_max_count(
                                qname.0.as_deref(),
                                &qname.1,
                                val,
                                val,
                            )?
                        }
                    }
                    ValueSearch::NotSome(ref val) => {
                        let total = db
                            .get_node_annos()
                            .number_of_annotations_by_name(qname.0.as_deref(), &qname.1)?;
                        total
                            - db.get_node_annos().guess_max_count(
                                qname.0.as_deref(),
                                &qname.1,
                                val,
                                val,
                            )?
                    }
                    ValueSearch::Any => db
                        .get_node_annos()
                        .number_of_annotations_by_name(qname.0.as_deref(), &qname.1)?,
                }
            };

        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);
        common_args
            .output_size_cache
            .insert(common_args.node_nr, est_output);

        let it = base_it.map_ok(|n| smallvec![n]);

        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(ExecutionNodeDesc::empty_with_fragment(
                common_args.node_nr,
                common_args.query_fragment.clone(),
                est_output,
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (qname.0, Some(qname.1)),
                cond: filters,
                const_output,
            }),
            is_sorted: false,
            timeout: common_args.timeout,
        })
    }

    fn new_annosearch_regex(
        db: &'a AnnotationGraph,
        qname: (Option<String>, String),
        pattern: &str,
        negated: bool,
        filters: Vec<MatchValueFilterFunc>,
        is_meta: bool,
        common_args: CommonArguments,
    ) -> Result<NodeSearch<'a>> {
        // match_regex works only with values
        let base_it =
            db.get_node_annos()
                .regex_anno_search(qname.0.as_deref(), &qname.1, pattern, negated);

        let const_output = if is_meta {
            Some(NODE_TYPE_KEY.clone())
        } else {
            None
        };

        let base_it: Box<dyn Iterator<Item = Result<Match>>> =
            if let Some(const_output) = const_output.clone() {
                let is_unique = db.get_node_annos().get_qnames(&qname.1)?.len() <= 1;
                // Replace the result annotation with a constant value.
                // If a node matches two different annotations (because there is no namespace), this can result in duplicates which needs to be filtered out.
                if is_unique {
                    Box::new(base_it.map(move |m| match m {
                        Ok(m) => Ok(Match {
                            node: m.node,
                            anno_key: const_output.clone(),
                        }),
                        Err(e) => Err(e.into()),
                    }))
                } else {
                    Box::new(
                        base_it
                            .unique_by(|m| match m {
                                Ok(m) => Some(m.clone()),
                                Err(_) => None,
                            })
                            .map(move |m| match m {
                                Ok(m) => Ok(Match {
                                    node: m.node,
                                    anno_key: const_output.clone(),
                                }),
                                Err(e) => Err(e.into()),
                            }),
                    )
                }
            } else {
                Box::new(base_it.map(|it| it.map_err(GraphAnnisError::from)))
            };

        let est_output = if let Some(cached) =
            common_args.output_size_cache.get(&common_args.node_nr)
        {
            *cached
        } else if negated {
            let total = db
                .get_node_annos()
                .number_of_annotations_by_name(qname.0.as_deref(), &qname.1)?;
            let est_for_regex =
                db.get_node_annos()
                    .guess_max_count_regex(qname.0.as_deref(), &qname.1, pattern)?;
            if total == est_for_regex {
                // The regex estimation did  a fallback to "every value is
                // possible" and thus is not really an accurate representation
                // of the possible values. Return the worst case estimate for
                // negation (all matches) instead.
                total
            } else {
                total - est_for_regex
            }
        } else {
            db.get_node_annos()
                .guess_max_count_regex(qname.0.as_deref(), &qname.1, pattern)?
        };
        common_args
            .output_size_cache
            .insert(common_args.node_nr, est_output);

        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let it = base_it.map_ok(|n| smallvec![n]);

        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(ExecutionNodeDesc::empty_with_fragment(
                common_args.node_nr,
                common_args.query_fragment,
                est_output,
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (qname.0, Some(qname.1)),
                cond: filters,
                const_output,
            }),
            is_sorted: false,
            timeout: common_args.timeout,
        })
    }

    fn new_tokensearch(
        db: &'a AnnotationGraph,
        val: ValueSearch<String>,
        filters: Vec<MatchValueFilterFunc>,
        leafs_only: bool,
        match_regex: bool,
        common_args: CommonArguments,
    ) -> Result<NodeSearch<'a>> {
        let it_base: Box<dyn Iterator<Item = Result<Match>>> = match val {
            ValueSearch::Any => {
                let it = db
                    .get_node_annos()
                    .exact_anno_search(Some(&TOKEN_KEY.ns), &TOKEN_KEY.name, None.into())
                    .map(|it| it.map_err(GraphAnnisError::from));
                Box::new(it)
            }
            ValueSearch::Some(ref val) => {
                let it = if match_regex {
                    db.get_node_annos().regex_anno_search(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        val,
                        false,
                    )
                } else {
                    db.get_node_annos().exact_anno_search(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        ValueSearch::Some(val),
                    )
                };
                Box::new(it.map(|it| it.map_err(GraphAnnisError::from)))
            }
            ValueSearch::NotSome(ref val) => {
                let it = if match_regex {
                    db.get_node_annos().regex_anno_search(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        val,
                        true,
                    )
                } else {
                    db.get_node_annos().exact_anno_search(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        ValueSearch::NotSome(val),
                    )
                };
                Box::new(it.map(|it| it.map_err(GraphAnnisError::from)))
            }
        };

        let it_base = if leafs_only {
            let cov_gs: Vec<Arc<dyn GraphStorage>> = db
                .get_all_components(Some(AnnotationComponentType::Coverage), None)
                .into_iter()
                .filter_map(|c| db.get_graphstorage(&c))
                .filter(|gs| {
                    if let Some(stats) = gs.get_statistics() {
                        stats.nodes > 0
                    } else {
                        true
                    }
                })
                .collect();

            let it = it_base.filter_ok(move |n| {
                for cov in cov_gs.iter() {
                    if cov.get_outgoing_edges(n.node).next().is_some() {
                        return false;
                    }
                }
                true
            });
            Box::new(it)
        } else {
            it_base
        };
        // map to vector
        let it = it_base.map_ok(move |n| {
            smallvec![Match {
                node: n.node,
                anno_key: NODE_TYPE_KEY.clone(),
            }]
        });

        // TODO: is_leaf should be part of the estimation
        let est_output = match val {
            ValueSearch::Some(ref val) => {
                if match_regex {
                    db.get_node_annos().guess_max_count_regex(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        val,
                    )?
                } else {
                    db.get_node_annos().guess_max_count(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        val,
                        val,
                    )?
                }
            }
            ValueSearch::NotSome(val) => {
                let total_count = db
                    .get_node_annos()
                    .number_of_annotations_by_name(Some(&TOKEN_KEY.ns), &TOKEN_KEY.name)?;
                let positive_count = if match_regex {
                    db.get_node_annos().guess_max_count_regex(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        &val,
                    )?
                } else {
                    db.get_node_annos().guess_max_count(
                        Some(&TOKEN_KEY.ns),
                        &TOKEN_KEY.name,
                        &val,
                        &val,
                    )?
                };
                total_count - positive_count
            }
            ValueSearch::Any => db
                .get_node_annos()
                .number_of_annotations_by_name(Some(&TOKEN_KEY.ns), &TOKEN_KEY.name)?,
        };
        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(ExecutionNodeDesc::empty_with_fragment(
                common_args.node_nr,
                common_args.query_fragment.clone(),
                est_output,
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (
                    Some(TOKEN_KEY.ns.clone().into()),
                    Some(TOKEN_KEY.name.clone().into()),
                ),
                cond: filters,
                const_output: Some(NODE_TYPE_KEY.clone()),
            }),
            is_sorted: false,
            timeout: common_args.timeout,
        })
    }

    fn new_anytoken_search(
        db: &'a AnnotationGraph,
        common_args: CommonArguments,
    ) -> Result<NodeSearch<'a>> {
        let it: Box<dyn Iterator<Item = Result<MatchGroup>>> = Box::from(AnyTokenSearch::new(db)?);
        // create filter functions
        let mut filters: Vec<MatchValueFilterFunc> = Vec::new();

        let cov_gs: Vec<Arc<dyn GraphStorage>> = db
            .get_all_components(Some(AnnotationComponentType::Coverage), None)
            .into_iter()
            .filter_map(|c| db.get_graphstorage(&c))
            .filter(|gs| {
                if let Some(stats) = gs.get_statistics() {
                    stats.nodes > 0
                } else {
                    true
                }
            })
            .collect();

        let filter_func: MatchValueFilterFunc = Box::new(move |m, _| {
            for cov in cov_gs.iter() {
                if let Some(edge) = cov.get_outgoing_edges(m.node).next() {
                    edge?;
                    return Ok(false);
                }
            }
            Ok(true)
        });
        filters.push(filter_func);

        let est_output = db
            .get_node_annos()
            .number_of_annotations_by_name(Some(&TOKEN_KEY.ns), &TOKEN_KEY.name)?;
        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(ExecutionNodeDesc::empty_with_fragment(
                common_args.node_nr,
                common_args.query_fragment.clone(),
                est_output,
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (
                    Some(TOKEN_KEY.ns.clone().into()),
                    Some(TOKEN_KEY.name.clone().into()),
                ),
                cond: filters,
                const_output: Some(NODE_TYPE_KEY.clone()),
            }),
            is_sorted: true,
            timeout: common_args.timeout,
        })
    }

    pub fn new_partofcomponentsearch(
        db: &'a AnnotationGraph,
        node_search_desc: Arc<NodeSearchDesc>,
        desc: Option<&ExecutionNodeDesc>,
        components: HashSet<Component<AnnotationComponentType>>,
        edge_anno_spec: Option<EdgeAnnoSearchSpec>,
        timeout: TimeoutCheck,
    ) -> Result<NodeSearch<'a>> {
        let node_search_desc_1 = node_search_desc.clone();
        let node_search_desc_2 = node_search_desc.clone();

        let it = components
            .into_iter()
            .flat_map(
                move |c: Component<AnnotationComponentType>| -> Box<
                    dyn Iterator<Item = std::result::Result<NodeID, GraphAnnisCoreError>>,
                > {
                    if let Some(gs) = db.get_graphstorage_as_ref(&c) {
                        if let Some(EdgeAnnoSearchSpec::ExactValue {
                            ref ns,
                            ref name,
                            ref val,
                        }) = edge_anno_spec
                        {
                            // for each component get the source nodes with this edge annotation
                            let anno_storage: &dyn EdgeAnnotationStorage = gs.get_anno_storage();

                            let it = anno_storage
                                .exact_anno_search(
                                    ns.as_ref().map(String::as_str),
                                    name,
                                    val.as_ref().map(String::as_str).into(),
                                )
                                .map_ok(|m| m.node);
                            Box::new(it)
                        } else {
                            // for each component get the all its source nodes
                            gs.source_nodes()
                        }
                    } else {
                        Box::new(std::iter::empty())
                    }
                },
            )
            .map(move |node| {
                match node {
                    Ok(node) => {
                        // fetch annotation candidates for the node based on the original description
                        let node_search_desc = node_search_desc_1.clone();
                        let all_keys = db.get_node_annos().get_all_keys_for_item(
                            &node,
                            node_search_desc.qname.0.as_deref(),
                            node_search_desc.qname.1.as_deref(),
                        )?;
                        let it = all_keys
                            .into_iter()
                            .map(move |anno_key| Match { node, anno_key });
                        Ok(it)
                    }
                    Err(e) => Err(e),
                }
            })
            .flatten_ok()
            .map(move |m| match m {
                Ok(m) => {
                    // only include the nodes that fullfill all original node search predicates
                    for cond in &node_search_desc_2.cond {
                        if !cond(&m, db.get_node_annos())? {
                            return Ok(None);
                        }
                    }
                    Ok(Some(smallvec![m]))
                }
                Err(e) => Err(GraphAnnisError::from(e)),
            })
            .filter_map_ok(|m| m);
        let mut new_desc = desc.cloned();
        if let Some(ref mut new_desc) = new_desc {
            new_desc.impl_description = String::from("part-of-component-search");
        }
        Ok(NodeSearch {
            it: Box::new(it),
            desc: new_desc,
            node_search_desc,
            is_sorted: false,
            timeout,
        })
    }

    pub fn set_desc(&mut self, desc: Option<ExecutionNodeDesc>) {
        self.desc = desc;
    }

    pub fn get_node_search_desc(&self) -> Arc<NodeSearchDesc> {
        self.node_search_desc.clone()
    }
}

impl ExecutionNode for NodeSearch<'_> {
    fn get_desc(&self) -> Option<&ExecutionNodeDesc> {
        self.desc.as_ref()
    }

    fn as_nodesearch(&self) -> Option<&NodeSearch> {
        Some(self)
    }

    fn is_sorted_by_text(&self) -> bool {
        self.is_sorted
    }
}

impl Iterator for NodeSearch<'_> {
    type Item = Result<MatchGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(e) = self.timeout.check() {
            return Some(Err(e));
        }
        self.it.next()
    }
}
