use super::{Desc, ExecutionNode, NodeSearchDesc};
use graphdb::{GraphDB, ANNIS_NS};
use {Annotation, Component, ComponentType, Match, StringID};
use stringstorage::StringStorage;

use util;
use regex;

use std::rc::Rc;
use std::fmt;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<Iterator<Item = Vec<Match>> + 'a>,

    desc: Option<Desc>,
    node_search_desc: Rc<NodeSearchDesc>,
}

#[derive(Clone)]
pub enum NodeSearchSpec {
    ExactValue {
        ns: Option<String>,
        name: String,
        val: Option<String>,
    },
    RegexValue {
        ns: Option<String>,
        name: String,
        val: String,
    },
    ExactTokenValue { val: String, leafs_only: bool },
    RegexTokenValue { val: String, leafs_only: bool },
    AnyToken,
    AnyNode,
}

impl NodeSearchSpec {
    pub fn new_exact(ns: Option<&str>, name: &str, val: Option<&str>) -> NodeSearchSpec {
        NodeSearchSpec::ExactValue {
            ns: ns.map(|v| String::from(v)),
            name: String::from(name),
            val: val.map(|v| String::from(v)),
        }
    }

    pub fn new_regex(ns: Option<&str>, name: &str, val: &str) -> NodeSearchSpec {
        NodeSearchSpec::RegexValue {
            ns: ns.map(|v| String::from(v)),
            name: String::from(name),
            val: String::from(val),
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
                write!(
                    f,
                    "{}:{}=/{}/",
                    ns.as_ref().unwrap(),
                    name,
                    &val
                )
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
    pub fn from_spec(spec: NodeSearchSpec, node_nr : usize, db: &'a GraphDB) -> Option<NodeSearch<'a>> {
        let query_fragment = format!("{}", spec);

        match spec {
            NodeSearchSpec::ExactValue { ns, name, val } => {
                NodeSearch::new_annosearch(db, ns, name, val, false, &query_fragment, node_nr)
            },
            NodeSearchSpec::RegexValue { ns, name, val } => {
                NodeSearch::new_annosearch(db, ns, name, Some(val), true, &query_fragment, node_nr)
            },
            NodeSearchSpec::ExactTokenValue { val, leafs_only } => {
                NodeSearch::new_tokensearch(db, Some(val), leafs_only, false, &query_fragment, node_nr)
            },
            NodeSearchSpec::RegexTokenValue { val, leafs_only } => {
                NodeSearch::new_tokensearch(db, Some(val), leafs_only, true, &query_fragment, node_nr)
            },
            NodeSearchSpec::AnyToken => {
                NodeSearch::new_tokensearch(db, None, false, false, &query_fragment, node_nr)
            },
            NodeSearchSpec::AnyNode => {
                let type_key = db.get_node_type_key();
                let node_str_id = db.strings.find_id("node")?.clone();
                let it = db.node_annos
                    .exact_anno_search(Some(type_key.ns), type_key.name, None)
                    .map(move |n| vec![n]);

                let filter_func: Box<Fn(Annotation, &StringStorage) -> bool> =
                    Box::new(move |anno, _| {
                        return anno.val == node_str_id;
                    });

                let type_key = db.get_node_type_key();

                Some(NodeSearch {
                    it: Box::new(it),
                    desc: Some(Desc::empty_with_fragment(&query_fragment,  node_nr)),
                    node_search_desc: Rc::new(NodeSearchDesc {
                        qname: (Some(type_key.ns), Some(type_key.name)),
                        cond: filter_func,
                    }),
                })
            },
        }
    }


    fn new_annosearch(
        db: &'a GraphDB,
        ns: Option<String>,
        name: String,
        val: Option<String>,
        match_regex: bool,
        query_fragment: &str,
        node_nr : usize,
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

        let it = base_it.map(|n| vec![n]);

        let filter_func: Box<Fn(Annotation, &StringStorage) -> bool> = if match_regex {
            // match_regex works only with values
            let val = val?.clone();
            let full_match_pattern = util::regex_full_match(&val);
            let re = regex::Regex::new(&full_match_pattern).ok()?;
            Box::new(move |anno, strings| {
                let val_str_opt = strings.str(anno.val);
                if let Some(val_str) = val_str_opt {
                    return re.is_match(val_str);
                } else {
                    return false;
                };
            })
        } else if val_id.is_some() {
            Box::new(move |anno, _| {
                return anno.val == val_id.unwrap();
            })
        } else {
            Box::new(move |_, _| {
                return true;
            })
        };

        return Some(NodeSearch {
            it: Box::new(it),
            desc: Some(Desc::empty_with_fragment(&query_fragment, node_nr)),
            node_search_desc: Rc::new(NodeSearchDesc {
                qname: (ns_id, Some(name_id)),
                cond: filter_func,
            }),
        });
    }

    fn new_tokensearch(
        db: &'a GraphDB,
        val: Option<String>,
        leafs_only: bool,
        match_regex: bool,
        query_fragment: &str,
        node_nr : usize,
    ) -> Option<NodeSearch<'a>> {
        let tok_key = db.get_token_key();
        let any_anno = Annotation {
            key: db.get_node_type_key(),
            val: 0,
        };


        if let Some(v) = val {
            let cov_gs = db.get_graphstorage(&Component {
                ctype: ComponentType::Coverage,
                layer: String::from(ANNIS_NS),
                name: String::from(""),
            });

            let base_it = if match_regex {
                db.node_annos
                    .regex_anno_search(&db.strings, Some(tok_key.ns), tok_key.name, &v)
            } else {
                let val_id = db.strings.find_id(&v)?.clone();
                db.node_annos
                    .exact_anno_search(Some(tok_key.ns), tok_key.name, Some(val_id))
            };

            let it: Box<Iterator<Item = Vec<Match>> + 'a> = if leafs_only {
                Box::new(
                    base_it
                        .filter(move |n| if let Some(ref cov) = cov_gs {
                            cov.get_outgoing_edges(&n.node).next().is_none()
                        } else {
                            true
                        })
                        .map(move |n| {
                            vec![
                                Match {
                                    node: n.node,
                                    anno: any_anno.clone(),
                                },
                            ]
                        }),
                )
            } else {
                Box::new(base_it.map(move |n| {
                    vec![
                        Match {
                            node: n.node,
                            anno: any_anno.clone(),
                        },
                    ]
                }))
            };

            let filter_func: Box<Fn(Annotation, &StringStorage) -> bool> = if match_regex {
                let full_match_pattern = util::regex_full_match(&v);
                let re = regex::Regex::new(&full_match_pattern).ok()?;
                Box::new(move |anno, strings| {
                    let val_str_opt = strings.str(anno.val);
                    if let Some(val_str) = val_str_opt {
                        return re.is_match(val_str);
                    } else {
                        return false;
                    };
                })
            
            } else {
                let val_id = db.strings.find_id(&v)?.clone();
                Box::new(move |anno, _| {
                    return anno.val == val_id;
                })
            };

            let tok_key = db.get_token_key();


            return Some(NodeSearch {
                it,
                desc: Some(Desc::empty_with_fragment(&query_fragment, node_nr)),
                node_search_desc: Rc::new(NodeSearchDesc {
                    qname: (Some(tok_key.ns), Some(tok_key.name)),
                    cond: filter_func,
                }),
            });
        } else {
            let it = db.node_annos
                .exact_anno_search(Some(tok_key.ns), tok_key.name, None)
                .map(move |n| {
                    vec![
                        Match {
                            node: n.node,
                            anno: any_anno.clone(),
                        },
                    ]
                });

            let filter_func: Box<Fn(Annotation, &StringStorage) -> bool> =
                Box::new(move |_anno, _| {
                    return true;
                });

            let tok_key = db.get_token_key();

            Some(NodeSearch {
                it: Box::new(it),
                desc: Some(Desc::empty_with_fragment("tok", node_nr)),
                node_search_desc: Rc::new(NodeSearchDesc {
                    qname: (Some(tok_key.ns), Some(tok_key.name)),
                    cond: filter_func,
                }),
            })
        }
    }


    pub fn set_desc(&mut self, desc: Option<Desc>) {
        self.desc = desc;
    }


    pub fn get_node_search_desc(&'a self) -> Rc<NodeSearchDesc> {
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
