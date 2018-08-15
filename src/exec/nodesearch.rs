use types::Edge;
use annostorage::AnnoStorage;
use operator::EdgeAnnoSearchSpec;
use super::{Desc, ExecutionNode, NodeSearchDesc};
use graphdb::{GraphDB, ANNIS_NS};
use {Annotation, Component, ComponentType, Match, NodeID, StringID};
use stringstorage::StringStorage;

use util;
use regex;

use std::fmt;
use std::sync::Arc;
use std;

use itertools::Itertools;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<Iterator<Item = Vec<Match>> + 'a>,

    desc: Option<Desc>,
    node_search_desc: Arc<NodeSearchDesc>,
}
#[derive(Clone, Debug)]
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
            ns: ns.map(|v| String::from(v)),
            name: String::from(name),
            val: val.map(|v| String::from(v)),
            is_meta,
        }
    }

    pub fn new_regex(ns: Option<&str>, name: &str, val: &str, is_meta: bool) -> NodeSearchSpec {
        NodeSearchSpec::RegexValue {
            ns: ns.map(|v| String::from(v)),
            name: String::from(name),
            val: String::from(val),
            is_meta,
        }
    }
}

impl fmt::Display for NodeSearchSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &NodeSearchSpec::ExactValue {
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
            &NodeSearchSpec::RegexValue {
                ref ns,
                ref name,
                ref val,
                ..
            } => if ns.is_some() {
                write!(f, "{}:{}=/{}/", ns.as_ref().unwrap(), name, &val)
            } else {
                write!(f, "{}=/{}/", name, &val)
            },
            &NodeSearchSpec::ExactTokenValue {
                ref val,
                ref leafs_only,
            } => if *leafs_only {
                write!(f, "tok=\"{}\"", val)
            } else {
                write!(f, "\"{}\"", val)
            },
            &NodeSearchSpec::RegexTokenValue {
                ref val,
                ref leafs_only,
            } => if *leafs_only {
                write!(f, "tok=/{}/", val)
            } else {
                write!(f, "/{}/", val)
            },
            &NodeSearchSpec::AnyToken => write!(f, "tok"),
            &NodeSearchSpec::AnyNode => write!(f, "node"),
        }
    }
}

impl<'a> NodeSearch<'a> {
    pub fn from_spec(
        spec: NodeSearchSpec,
        node_nr: usize,
        db: &'a GraphDB,
    ) -> Option<NodeSearch<'a>> {
        let query_fragment = format!("{}", spec);

        match spec {
            NodeSearchSpec::ExactValue {
                ns,
                name,
                val,
                is_meta,
            } => NodeSearch::new_annosearch(
                db,
                ns,
                name,
                val,
                false,
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
                NodeSearch::new_annosearch(
                    db,
                    ns,
                    name,
                    Some(val),
                    is_regex,
                    is_meta,
                    &query_fragment,
                    node_nr,
                )
            }
            NodeSearchSpec::ExactTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                Some(val),
                leafs_only,
                false,
                &query_fragment,
                node_nr,
            ),
            NodeSearchSpec::RegexTokenValue { val, leafs_only } => NodeSearch::new_tokensearch(
                db,
                Some(val),
                leafs_only,
                true,
                &query_fragment,
                node_nr,
            ),
            NodeSearchSpec::AnyToken => {
                NodeSearch::new_tokensearch(db, None, true, false, &query_fragment, node_nr)
            }
            NodeSearchSpec::AnyNode => {
                let type_key = db.get_node_type_key();
                let node_str_id = db.strings.find_id("node")?.clone();
                let it = db.node_annos
                    .exact_anno_search(Some(type_key.ns), type_key.name, Some(node_str_id))
                    .map(move |n| vec![n]);

                let filter_func: Box<Fn(&Match, &StringStorage) -> bool + Send + Sync> = Box::new(move |m, _| {
                    return m.anno.val == node_str_id;
                });

                let type_key = db.get_node_type_key();

                let est_output =
                    db.node_annos
                        .guess_max_count(Some(type_key.ns), type_key.name, "node", "node");
                let est_output = std::cmp::max(1, est_output);

                let const_output = Some(Annotation {
                    key: db.get_node_type_key(),
                    val: db.strings.find_id("node").cloned().unwrap_or(0),
                });

                Some(NodeSearch {
                    it: Box::new(it),
                    desc: Some(Desc::empty_with_fragment(
                        &query_fragment,
                        node_nr,
                        Some(est_output),
                    )),
                    node_search_desc: Arc::new(NodeSearchDesc {
                        qname: (Some(type_key.ns), Some(type_key.name)),
                        cond: vec![filter_func],
                        const_output: const_output,
                    }),
                })
            }
        }
    }

    fn new_annosearch(
        db: &'a GraphDB,
        ns: Option<String>,
        name: String,
        val: Option<String>,
        match_regex: bool,
        is_meta: bool,
        query_fragment: &str,
        node_nr: usize,
    ) -> Option<NodeSearch<'a>> {
        let name_id: StringID = db.strings.find_id(&name)?.clone();
        // not finding the strings will result in an None result, not in an less specific search
        let ns_id: Option<StringID> = if let Some(ns) = ns.as_ref() {
            Some(db.strings.find_id(ns)?).cloned()
        } else {
            None
        };
        let val_id: Option<StringID> = val.clone().and_then(|v| db.strings.find_id(&v).cloned());

        if !match_regex && val.is_some() && val_id.is_none() {
            // searched string not found, impossible search
            return None;
        }

        let base_it = if match_regex {
            // match_regex works only with values
            db.node_annos
                .regex_anno_search(&db.strings, ns_id, name_id, &val.clone()?)
        } else {
            db.node_annos.exact_anno_search(ns_id, name_id, val_id)
        };

        let const_output = if is_meta {
            Some(Annotation {
                key: db.get_node_type_key(),
                val: db.strings.find_id("corpus").cloned().unwrap_or(0),
            })
        } else {
            None
        };

        let base_it :Box<Iterator<Item=Match>> = if let Some(const_output) = const_output.clone() {
            let is_unique = db.node_annos.get_qnames(name_id).len() <= 1;
            // Replace the result annotation with a constant value.
            // If a node matches two different annotations (because there is no namespace), this can result in duplicates which needs to be filtered out.
            if is_unique {
                Box::new(base_it.map(move |m| Match {
                    node: m.node,
                    anno: const_output.clone(),
                }))
            } else {
                Box::new(base_it.map(move |m| Match {
                    node: m.node,
                    anno: const_output.clone(),
                }).unique())
            }
        } else {
            base_it
        };

        let est_output = if match_regex {
            db.node_annos
                .guess_max_count_regex(ns_id, name_id, &val.clone()?)
        } else {
            if let Some(ref val) = val {
                db.node_annos.guess_max_count(ns_id, name_id, val, val)
            } else {
                db.node_annos.num_of_annotations(ns_id, name_id)
            }
        };

        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let it = base_it.map(|n| vec![n]);

        let mut filters: Vec<Box<Fn(&Match, &StringStorage) -> bool + Send + Sync>> = Vec::new();

        if match_regex {
            // match_regex works only with values
            let val = val?.clone();
            let full_match_pattern = util::regex_full_match(&val);
            let re = regex::Regex::new(&full_match_pattern).ok()?;
            filters.push(Box::new(move |m, strings| {
                let val_str_opt = strings.str(m.anno.val.clone());
                if let Some(val_str) = val_str_opt {
                    return re.is_match(val_str);
                } else {
                    return false;
                };
            }));
        } else if val_id.is_some() {
            filters.push(Box::new(move |m, _| {
                return m.anno.val == val_id.unwrap();
            }));
        };
        return Some(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(
                &query_fragment,
                node_nr,
                Some(est_output),
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (ns_id, Some(name_id)),
                cond: filters,
                const_output: const_output,
            }),
        });
    }

    fn new_tokensearch(
        db: &'a GraphDB,
        val: Option<String>,
        leafs_only: bool,
        match_regex: bool,
        query_fragment: &str,
        node_nr: usize,
    ) -> Option<NodeSearch<'a>> {
        let tok_key = db.get_token_key();
        let any_anno = Annotation {
            key: db.get_node_type_key(),
            val: 0,
        };

        let it_base: Box<Iterator<Item = Match>> = if let Some(v) = val.clone() {
            let it = if match_regex {
                db.node_annos
                    .regex_anno_search(&db.strings, Some(tok_key.ns), tok_key.name, &v)
            } else {
                let val_id = db.strings.find_id(&v)?.clone();
                db.node_annos
                    .exact_anno_search(Some(tok_key.ns), tok_key.name, Some(val_id))
            };
            Box::new(it)
        } else {
            let it = db.node_annos
                .exact_anno_search(Some(tok_key.ns), tok_key.name, None);
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
                    cov.get_outgoing_edges(&n.node).next().is_none()
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
            vec![
                Match {
                    node: n.node,
                    anno: any_anno.clone(),
                },
            ]
        });
        // create filter functions
        let mut filters: Vec<Box<Fn(&Match, &StringStorage) -> bool + Send + Sync>> = Vec::new();

        if let Some(ref v) = val {
            if match_regex {
                let full_match_pattern = util::regex_full_match(v);
                let re = regex::Regex::new(&full_match_pattern).ok()?;
                filters.push(Box::new(move |m, strings| {
                    let val_str_opt = strings.str(m.anno.val.clone());
                    if let Some(val_str) = val_str_opt {
                        return re.is_match(val_str);
                    } else {
                        return false;
                    };
                }));
            } else {
                let val_id = db.strings.find_id(v)?.clone();
                filters.push(Box::new(move |m, _| {
                    return m.anno.val == val_id;
                }));
            };
        };

        if leafs_only {
            let cov_gs = db.get_graphstorage(&Component {
                ctype: ComponentType::Coverage,
                layer: String::from(ANNIS_NS),
                name: String::from(""),
            });
            let filter_func: Box<Fn(&Match, &StringStorage) -> bool + Send + Sync> = Box::new(move |m, _| {
                if let Some(ref cov) = cov_gs {
                    cov.get_outgoing_edges(&m.node).next().is_none()
                } else {
                    true
                }
            });
            filters.push(filter_func);
        };

        // TODO: is_leaf should be part of the estimation
        let est_output = if let Some(ref val) = val {
            if match_regex {
                db.node_annos
                    .guess_max_count_regex(Some(tok_key.ns), tok_key.name, &val)
            } else {
                db.node_annos
                    .guess_max_count(Some(tok_key.ns), tok_key.name, &val, &val)
            }
        } else {
            db.node_annos
                .num_of_annotations(Some(tok_key.ns), tok_key.name)
        };
        // always assume at least one output item otherwise very small selectivity can fool the planner
        let est_output = std::cmp::max(1, est_output);

        let const_output = Some(Annotation {
            key: db.get_node_type_key(),
            val: db.strings.find_id("node").cloned().unwrap_or(0),
        });

        return Some(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(
                &query_fragment,
                node_nr,
                Some(est_output),
            )),
            node_search_desc: Arc::new(NodeSearchDesc {
                qname: (Some(tok_key.ns), Some(tok_key.name)),
                cond: filters,
                const_output: const_output,
            }),
        });
    }

    pub fn new_partofcomponentsearch(
        db: &'a GraphDB,
        node_search_desc: Arc<NodeSearchDesc>,
        desc: Option<&Desc>,
        components: Vec<Component>,
        edge_anno_spec: Option<EdgeAnnoSearchSpec>,
    ) -> NodeSearch<'a> {
        let node_search_desc_1 = node_search_desc.clone();
        let node_search_desc_2 = node_search_desc.clone();

        let edge_anno_template: Option<Annotation> =
            edge_anno_spec.and_then(|e| e.get_anno(&db.strings));

        let it = components
            .into_iter()
            .flat_map(move |c: Component| -> Box<Iterator<Item = NodeID>> {
                if let Some(gs) = db.get_graphstorage_as_ref(&c) {
                    if let Some(ref edge_anno_template) = edge_anno_template {
                        // for each component get the source nodes with this edge annotation
                        let anno_storage: &AnnoStorage<Edge> = gs.get_anno_storage();
                        let ns = if edge_anno_template.key.ns == 0 {
                            None
                        } else {
                            Some(edge_anno_template.key.ns)
                        };
                        let it = anno_storage
                            .exact_anno_search(
                                ns,
                                edge_anno_template.key.name,
                                Some(edge_anno_template.val),
                            )
                            .map(|m: Match| m.node);
                        return Box::new(it);
                    } else {
                        // for each component get the all its source nodes
                        return gs.source_nodes();
                    }
                } else {
                    return Box::new(std::iter::empty());
                }
            })
            .flat_map(move |node: NodeID| {
                // fetch annotation candidates for the node based on the original description
                let node_search_desc = node_search_desc_1.clone();
                db.node_annos
                    .find_by_name(&node, node_search_desc.qname.0, node_search_desc.qname.1)
                    .into_iter()
                    .map(move |anno| Match { node, anno })
            })
            .filter_map(move |m: Match| -> Option<Vec<Match>> {
                // only include the nodes that fullfill all original node search predicates
                for cond in node_search_desc_2.cond.iter() {
                    if !cond(&m, &db.strings) {
                        return None;
                    }
                }
                return Some(vec![m]);
            });
        let mut new_desc = desc.cloned();
        if let Some(ref mut new_desc) = new_desc {
            new_desc.impl_description = String::from("part-of-component-search");
        }
        return NodeSearch {
            it: Box::new(it),
            desc: new_desc,
            node_search_desc: node_search_desc,
        };
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
