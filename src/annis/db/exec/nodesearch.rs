use super::{Desc, ExecutionNode, NodeSearchDesc};
use annis::db::AnnotationStorage;
use annis::db::{Graph, Match, ANNIS_NS};
use annis::errors::*;
use annis::operator::EdgeAnnoSearchSpec;
use annis::types::{Component, ComponentType, Edge, LineColumnRange, NodeID};
use annis::util;
use itertools::Itertools;
use regex;
use std;
use std::fmt;
use std::sync::Arc;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<Iterator<Item = Vec<Match>> + 'a>,

    desc: Option<Desc>,
    node_search_desc: Arc<NodeSearchDesc>,
}
#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum NodeSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
        is_meta: bool,
    },
    RegexValue {
        ns: Option<String>,
        name: String,
        val: String,
        is_meta: bool,
    },
    ExactTokenValue {
        val: String,
        leafs_only: bool,
    },
    RegexTokenValue {
        val: String,
        leafs_only: bool,
    },
    AnyToken,
    AnyNode,
}

impl NodeSearchSpec {
    pub fn new_exact(
        ns: Option<&str>,
        name: &str,
        val: Option<&str>,
        is_meta: bool,
    ) -> NodeSearchSpec {
        NodeSearchSpec::ExactValue {
            ns: ns.map(String::from),
            name: String::from(name),
            val: val.map(String::from),
            is_meta,
        }
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
            } => if ns.is_some() && val.is_some() {
                write!(
                    f,
                    "{}:{}=\"{}\"",
                    ns.as_ref().unwrap(),
                    name,
                    val.as_ref().unwrap()
                )
            } else if ns.is_some() {
                write!(f, "{}:{}", ns.as_ref().unwrap(), name)
            } else if val.is_some() {
                write!(f, "{}=\"{}\"", name, val.as_ref().unwrap())
            } else {
                write!(f, "{}", name)
            },
            NodeSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => if ns.is_some() {
                write!(f, "{}:{}=/{}/", ns.as_ref().unwrap(), name, &val)
            } else {
                write!(f, "{}=/{}/", name, &val)
            },
            NodeSearchSpec::ExactTokenValue {
                ref val,
                ref leafs_only,
            } => if *leafs_only {
                write!(f, "tok=\"{}\"", val)
            } else {
                write!(f, "\"{}\"", val)
            },
            NodeSearchSpec::RegexTokenValue {
                ref val,
                ref leafs_only,
            } => if *leafs_only {
                write!(f, "tok=/{}/", val)
            } else {
                write!(f, "/{}/", val)
            },
            NodeSearchSpec::AnyToken => write!(f, "tok"),
            NodeSearchSpec::AnyNode => write!(f, "node"),
        }
    }
}

impl<'a> NodeSearch<'a> {
    pub fn from_spec(
        spec: NodeSearchSpec,
        node_nr: usize,
        db: &'a Graph,
        location_in_query: Option<LineColumnRange>,
    ) -> Result<NodeSearch<'a>> {
        let query_fragment = format!("{}", spec);

        match spec {
            NodeSearchSpec::ExactValue {
                ns,
                name,
                val,
                is_meta,
            } => NodeSearch::new_annosearch_exact(
                db,
                (ns, name),
                val,
                is_meta,
                &query_fragment,
                node_nr,
            ),
            NodeSearchSpec::RegexValue {
                ns,
                name,
                val,
                is_meta,
            } => {
                // check if the regex can be replaced with an exact value search
                let is_regex = util::contains_regex_metacharacters(&val);
                if is_regex {
                    NodeSearch::new_annosearch_regex(
                        db,
                        (ns,name),
                        &val,
                        is_meta,
                        &query_fragment,
                        node_nr,
                        location_in_query,
                    )
                } else {
                    NodeSearch::new_annosearch_exact(
                        db,
                        (ns,name),
                        Some(val),
                        is_meta,
                        &query_fragment,
                        node_nr,
                    )
                }
            }
            NodeSearchSpec::ExactTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                Some(val),
                leafs_only,
                false,
                &query_fragment,
                node_nr,
                location_in_query,
            ),
            NodeSearchSpec::RegexTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                Some(val),
                leafs_only,
                true,
                &query_fragment,
                node_nr,
                location_in_query,
            ),
            NodeSearchSpec::AnyToken => NodeSearch::new_tokensearch(
                db,
                None,
                true,
                false,
                &query_fragment,
                node_nr,
                location_in_query,
            ),
            NodeSearchSpec::AnyNode => {
                let type_key = db.get_node_type_key();

                let it = db
                    .node_annos
                    .exact_anno_search(Some(type_key.ns), type_key.name, Some("node".to_owned()))
                    .map(move |n| vec![n]);

                let node_annos = db.node_annos.clone();
                let filter_func: Box<Fn(&Match) -> bool + Send + Sync> = Box::new(move |m| {
                    if let Some(val) = node_annos.get_value_for_item_by_id(&m.node, m.anno_key) {
                        return val == "node";
                    } else {
                        return false;
                    }
                });

                let type_key = db.get_node_type_key();

                let est_output = db.node_annos.guess_max_count(
                    Some(type_key.ns.clone()),
                    type_key.name.clone(),
                    "node",
                    "node",
                );
                let est_output = std::cmp::max(1, est_output);

                let const_output = db.node_annos.get_key_id(&db.get_node_type_key());

                Ok(NodeSearch {
                    it: Box::new(it),
                    desc: Some(Desc::empty_with_fragment(
                        &query_fragment,
                        node_nr,
                        Some(est_output),
                    )),
                    node_search_desc: Arc::new(NodeSearchDesc {
                        qname: (Some(type_key.ns), Some(type_key.name)),
                        cond: vec![filter_func],
                        const_output,
                    }),
                })
            }
        }
    }

    fn new_annosearch_exact(
        db: &'a Graph,
        qname:(Option<String>, String),
        val: Option<String>,
        is_meta: bool,
        query_fragment: &str,
        node_nr: usize,
    ) -> Result<NodeSearch<'a>> {
        let base_it = db.node_annos
                .exact_anno_search(qname.0.clone(), qname.1.clone(), val.clone());

        let const_output = if is_meta {
            Some(
                db.node_annos
                    .get_key_id(&db.get_node_type_key())
                    .ok_or("Node type annotation does not exist in database")?,
            )
        } else {
            None
        };

        let base_it: Box<Iterator<Item = Match>> = if let Some(const_output) = const_output {
            let is_unique = db.node_annos.get_qnames(&qname.1).len() <= 1;
            // Replace the result annotation with a constant value.
            // If a node matches two different annotations (because there is no namespace), this can result in duplicates which needs to be filtered out.
            if is_unique {
                Box::new(base_it.map(move |m| Match {
                    node: m.node,
                    anno_key: const_output,
                }))
            } else {
                Box::new(
                    base_it
                        .map(move |m| Match {
                            node: m.node,
                            anno_key: const_output,
                        }).unique(),
                )
            }
        } else {
            base_it
        };

        let est_output = if let Some(ref val) = val {
            db.node_annos
                .guess_max_count(qname.0.clone(), qname.1.clone(), &val, &val)
        } else {
            db.node_annos
                .number_of_annotations_by_name(qname.0.clone(), qname.1.clone())
        };

        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let it = base_it.map(|n| vec![n]);

        let mut filters: Vec<Box<Fn(&Match) -> bool + Send + Sync>> = Vec::new();

        if val.is_some() {
            let val = val.unwrap();
            let node_annos = db.node_annos.clone();
            filters.push(Box::new(move |m| {
                if let Some(anno_val) = node_annos.get_value_for_item_by_id(&m.node, m.anno_key) {
                    return anno_val == val.as_str();
                } else {
                    return false;
                }
            }));
        };
        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(
                &query_fragment,
                node_nr,
                Some(est_output),
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (qname.0, Some(qname.1)),
                cond: filters,
                const_output,
            }),
        })
    }

    fn new_annosearch_regex(
        db: &'a Graph,
        qname:(Option<String>, String),
        pattern: &str,
        is_meta: bool,
        query_fragment: &str,
        node_nr: usize,
        location_in_query: Option<LineColumnRange>,
    ) -> Result<NodeSearch<'a>> {
        let base_it = 
            // match_regex works only with values
            db.node_annos.regex_anno_search(
                qname.0.clone(),
                qname.1.clone(),
                pattern,
            );
        

        let const_output = if is_meta {
            Some(
                db.node_annos
                    .get_key_id(&db.get_node_type_key())
                    .ok_or("Node type annotation does not exist in database")?,
            )
        } else {
            None
        };

        let base_it: Box<Iterator<Item = Match>> = if let Some(const_output) = const_output {
            let is_unique = db.node_annos.get_qnames(&qname.1).len() <= 1;
            // Replace the result annotation with a constant value.
            // If a node matches two different annotations (because there is no namespace), this can result in duplicates which needs to be filtered out.
            if is_unique {
                Box::new(base_it.map(move |m| Match {
                    node: m.node,
                    anno_key: const_output,
                }))
            } else {
                Box::new(
                    base_it
                        .map(move |m| Match {
                            node: m.node,
                            anno_key: const_output,
                        }).unique(),
                )
            }
        } else {
            base_it
        };

        let est_output = 
            db.node_annos.guess_max_count_regex(
                qname.0.clone(),
                qname.1.clone(),
                pattern,
            );

        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let it = base_it.map(|n| vec![n]);

        let mut filters: Vec<Box<Fn(&Match) -> bool + Send + Sync>> = Vec::new();

        let full_match_pattern = util::regex_full_match(&pattern);
        let re = regex::Regex::new(&full_match_pattern);
        match re {
            Ok(re) => {
                let node_annos = db.node_annos.clone();
                filters.push(Box::new(move |m| {
                    if let Some(val) = node_annos.get_value_for_item_by_id(&m.node, m.anno_key)
                    {
                        return re.is_match(val);
                    } else {
                        return false;
                    }
                }));
            }
            Err(e) => bail!(ErrorKind::AQLSemanticError(
                format!("/{}/ -> {}", pattern, e),
                location_in_query
            )),
        }
    
        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(
                &query_fragment,
                node_nr,
                Some(est_output),
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (qname.0, Some(qname.1)),
                cond: filters,
                const_output,
            }),
        })
    }

    fn new_tokensearch(
        db: &'a Graph,
        val: Option<String>,
        leafs_only: bool,
        match_regex: bool,
        query_fragment: &str,
        node_nr: usize,
        location_in_query: Option<LineColumnRange>,
    ) -> Result<NodeSearch<'a>> {
        let tok_key = db.get_token_key();
        let any_anno_key = db
            .node_annos
            .get_key_id(&db.get_node_type_key())
            .ok_or("Node type annotation does not exist in database")?;

        let it_base: Box<Iterator<Item = Match>> = if let Some(v) = val.clone() {
            let it = if match_regex {
                db.node_annos
                    .regex_anno_search(Some(tok_key.ns.clone()), tok_key.name.clone(), &v)
            } else {
                db.node_annos.exact_anno_search(
                    Some(tok_key.ns.clone()),
                    tok_key.name.clone(),
                    Some(v),
                )
            };
            Box::new(it)
        } else {
            let it = db.node_annos.exact_anno_search(
                Some(tok_key.ns.clone()),
                tok_key.name.clone(),
                None,
            );
            Box::new(it)
        };
        let it_base = if leafs_only {
            let cov_gs = db.get_graphstorage(&Component {
                ctype: ComponentType::Coverage,
                layer: String::from(ANNIS_NS),
                name: String::from(""),
            });
            let it = it_base.filter(move |n| {
                if let Some(ref cov) = cov_gs {
                    cov.get_outgoing_edges(n.node).next().is_none()
                } else {
                    true
                }
            });
            Box::new(it)
        } else {
            it_base
        };
        // map to vector
        let it = it_base.map(move |n| {
            vec![Match {
                node: n.node,
                anno_key: any_anno_key,
            }]
        });
        // create filter functions
        let mut filters: Vec<Box<Fn(&Match) -> bool + Send + Sync>> = Vec::new();

        if let Some(v) = val.clone() {
            if match_regex {
                let full_match_pattern = util::regex_full_match(&v);
                let re = regex::Regex::new(&full_match_pattern);
                let node_annos = db.node_annos.clone();
                match re {
                    Ok(re) => filters.push(Box::new(move |m| {
                        if let Some(val) = node_annos.get_value_for_item_by_id(&m.node, m.anno_key)
                        {
                            return re.is_match(val);
                        } else {
                            return false;
                        }
                    })),
                    Err(e) => bail!(ErrorKind::AQLSemanticError(
                        format!("/{}/ -> {}", v, e),
                        location_in_query
                    )),
                };
            } else {
                let node_annos = db.node_annos.clone();
                filters.push(Box::new(move |m| {
                    if let Some(anno_val) = node_annos.get_value_for_item_by_id(&m.node, m.anno_key)
                    {
                        return anno_val == v.as_str();
                    } else {
                        return false;
                    }
                }));
            };
        };

        if leafs_only {
            let cov_gs = db.get_graphstorage(&Component {
                ctype: ComponentType::Coverage,
                layer: String::from(ANNIS_NS),
                name: String::from(""),
            });
            let filter_func: Box<Fn(&Match) -> bool + Send + Sync> = Box::new(move |m| {
                if let Some(ref cov) = cov_gs {
                    cov.get_outgoing_edges(m.node).next().is_none()
                } else {
                    true
                }
            });
            filters.push(filter_func);
        };

        // TODO: is_leaf should be part of the estimation
        let est_output = if let Some(val) = val {
            if match_regex {
                db.node_annos.guess_max_count_regex(
                    Some(tok_key.ns.clone()),
                    tok_key.name.clone(),
                    &val,
                )
            } else {
                db.node_annos.guess_max_count(
                    Some(tok_key.ns.clone()),
                    tok_key.name.clone(),
                    &val,
                    &val,
                )
            }
        } else {
            db.node_annos
                .number_of_annotations_by_name(Some(tok_key.ns.clone()), tok_key.name.clone())
        };
        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let const_output = db
            .node_annos
            .get_key_id(&db.get_node_type_key())
            .ok_or("Node type annotation does not exist in database")?;

        Ok(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(
                &query_fragment,
                node_nr,
                Some(est_output),
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (Some(tok_key.ns), Some(tok_key.name)),
                cond: filters,
                const_output: Some(const_output),
            }),
        })
    }

    pub fn new_partofcomponentsearch(
        db: &'a Graph,
        node_search_desc: Arc<NodeSearchDesc>,
        desc: Option<&Desc>,
        components: Vec<Component>,
        edge_anno_spec: Option<EdgeAnnoSearchSpec>,
    ) -> Result<NodeSearch<'a>> {
        let node_search_desc_1 = node_search_desc.clone();
        let node_search_desc_2 = node_search_desc.clone();

        let it = components
            .into_iter()
            .flat_map(move |c: Component| -> Box<Iterator<Item = NodeID>> {
                if let Some(gs) = db.get_graphstorage_as_ref(&c) {
                    if let Some(EdgeAnnoSearchSpec::ExactValue {
                        ref ns,
                        ref name,
                        ref val,
                    }) = edge_anno_spec
                    {
                        // for each component get the source nodes with this edge annotation
                        let anno_storage: &AnnotationStorage<Edge> = gs.get_anno_storage();

                        let it = anno_storage
                            .exact_anno_search(ns.clone(), name.clone(), val.clone())
                            .map(|m: Match| m.node);
                        return Box::new(it);
                    } else {
                        // for each component get the all its source nodes
                        return gs.source_nodes();
                    }
                } else {
                    return Box::new(std::iter::empty());
                }
            }).flat_map(move |node: NodeID| {
                // fetch annotation candidates for the node based on the original description
                let node_search_desc = node_search_desc_1.clone();
                db.node_annos
                    .find_annotations_for_item(
                        &node,
                        node_search_desc.qname.0.clone(),
                        node_search_desc.qname.1.clone(),
                    ).into_iter()
                    .map(move |anno_key| Match { node, anno_key })
            }).filter_map(move |m: Match| -> Option<Vec<Match>> {
                // only include the nodes that fullfill all original node search predicates
                for cond in &node_search_desc_2.cond {
                    if !cond(&m) {
                        return None;
                    }
                }
                Some(vec![m])
            });
        let mut new_desc = desc.cloned();
        if let Some(ref mut new_desc) = new_desc {
            new_desc.impl_description = String::from("part-of-component-search");
        }
        Ok(NodeSearch {
            it: Box::new(it),
            desc: new_desc,
            node_search_desc,
        })
    }

    pub fn set_desc(&mut self, desc: Option<Desc>) {
        self.desc = desc;
    }

    pub fn get_node_search_desc(&self) -> Arc<NodeSearchDesc> {
        self.node_search_desc.clone()
    }
}

impl<'a> ExecutionNode for NodeSearch<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }

    fn as_nodesearch(&self) -> Option<&NodeSearch> {
        Some(self)
    }
}

impl<'a> Iterator for NodeSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}
