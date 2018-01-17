use json;
use json::JsonValue;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use exec::nodesearch::NodeSearchSpec;

use graphdb::{ANNIS_NS, TOK};

use std::collections::BTreeMap;

pub fn parse(query_as_string: &str) -> Option<Disjunction> {
    let parsed = json::parse(query_as_string);

    if let Ok(root) = parsed {
        let mut conjunctions: Vec<Conjunction> = Vec::new();
        // iterate over all alternatives
        match root["alternatives"] {
            JsonValue::Array(ref alternatices) => {
                for alt in alternatices.iter() {
                    let mut q = Conjunction::new();

                    // add all nodes
                    let mut node_id_to_pos: BTreeMap<usize, usize> = BTreeMap::new();
                    if let JsonValue::Object(ref nodes) = alt["nodes"] {
                        for (node_name, node) in nodes.iter() {
                            if let JsonValue::Object(ref node_object) = *node {
                                if let Ok(ref node_id) = node_name.parse::<usize>() {
                                    let pos = parse_node(node_object, &mut q);
                                    node_id_to_pos.insert(node_id.clone(), pos);
                                }
                            }
                        }
                    }

                    // TODO: add all joins
                    if let JsonValue::Array(ref joins) = alt["joins"] {
                        for j in joins.iter() {
                            if let &JsonValue::Object(ref j_obj) = j {
                                parse_join(j_obj, &mut q);
                            }
                        }
                    }

                    // TODO: add all meta-data

                    conjunctions.push(q);
                    unimplemented!();
                }
            }
            _ => {
                return None;
            }
        };

        if !conjunctions.is_empty() {
            return Some(Disjunction::new(conjunctions));
        }
    }

    return None;
}

fn parse_node(node: &json::object::Object, q: &mut Conjunction) -> usize {
    // annotation search?
    if let JsonValue::Array(ref a) = node["nodeAnnotations"] {
        if !a.is_empty() {
            // get the first one
            let a = &a[0];
            return add_node_annotation(
                q,
                a["namespace"].as_str(),
                a["name"].as_str(),
                a["value"].as_str(),
                is_regex(a),
            );
        }
    }

    // check for special non-annotation search constructs
    // token search?
    if node["spannedText"].is_string()
        || (node["token"].is_boolean() && node["token"].is_boolean()) {
        let spanned = node["spannedText"].as_str();

        let mut leafs_only = false;
        if let Some(is_token) = node["token"].as_bool() {
            if is_token {
                // special treatment for explicit searches for token (tok="...)
                leafs_only = true;
            }
        }

        if let Some(tok_val) = spanned {
            if node["textMatching"].as_str() == Some("REGEXP_EQUAL") {
                return q.add_node(NodeSearchSpec::RegexTokenValue{val: String::from(tok_val), leafs_only,});
            } else {
                return q.add_node(NodeSearchSpec::ExactTokenValue{val: String::from(tok_val), leafs_only,});
            }
        } else {
            return q.add_node(NodeSearchSpec::AnyToken);
        }


    } else {
        // just search for any node
        return q.add_node((NodeSearchSpec::AnyNode));
    }
}

fn parse_join(join: &json::object::Object, q: &mut Conjunction) -> usize { 
    // get left and right index
    
    unimplemented!()
}

fn is_regex(json_node : &JsonValue) -> bool {
    if let Some(tm) = json_node["textMatching"].as_str() {
        if tm == "REGEXP_EQUAL" {
            return true;
        }
    }
    return false;
}

fn add_node_annotation(
    q: &mut Conjunction,
    ns: Option<&str>,
    name: Option<&str>,
    value: Option<&str>,
    regex : bool,
) -> usize {
    if let Some(name_val) = name {
        // TODO: replace regex with normal text matching if this is not an actual regular expression

        // search for the value
        if regex {
            if let Some(val) = value {
                let mut n: NodeSearchSpec =
                    NodeSearchSpec::new_regex(ns, name_val, val);
                return q.add_node(n);
            }
        } else  {
            // has namespace?
            let mut n: NodeSearchSpec =
                NodeSearchSpec::new_exact(ns, name_val, value);
            return q.add_node(n);
        }
    }
    unimplemented!()
}
