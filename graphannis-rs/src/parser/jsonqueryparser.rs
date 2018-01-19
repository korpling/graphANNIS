use serde_json;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use exec::nodesearch::NodeSearchSpec;

use graphdb::{ANNIS_NS, TOK};

use std::collections::BTreeMap;

pub fn parse(query_as_string: &str) -> Option<Disjunction> {
    let root : serde_json::Value = serde_json::from_str(query_as_string).ok()?;

    let mut conjunctions: Vec<Conjunction> = Vec::new();
    // iterate over all alternatives
    let alternatives = root["alternatives"].as_array()?;
    
    for alt in alternatives.iter() {
        let mut q = Conjunction::new();

        // add all nodes
        let mut node_id_to_pos: BTreeMap<usize, usize> = BTreeMap::new();
        if let serde_json::Value::Object(ref nodes) = alt["nodes"] {
            for (node_name, node) in nodes.iter() {
                if let Some(node_obj) = node.as_object() {
                    if let Ok(ref node_id) = node_name.parse::<u64>() {
                        let node_id = node_id.clone() as usize;

                        let pos = parse_node(node_obj, &mut q);
                        node_id_to_pos.insert(node_id, pos);
                    }
                }
            }
        }

        // TODO: add all joins
        if let serde_json::Value::Array(ref joins) = alt["joins"] {
            for j in joins.iter() {
                if let &serde_json::Value::Object(ref j_obj) = j {
                    parse_join(j_obj, &mut q, &node_id_to_pos);
                }
            }
        }

        // TODO: add all meta-data

        conjunctions.push(q);
        unimplemented!();
    }



    if !conjunctions.is_empty() {
        return Some(Disjunction::new(conjunctions));
    }
    
    return None;
}

fn parse_node(node: &serde_json::Map<String, serde_json::Value>, q: &mut Conjunction) -> usize {
    // annotation search?
    if let serde_json::Value::Array(ref a) = node["nodeAnnotations"] {
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

fn parse_join(join: &serde_json::Map<String, serde_json::Value>, q: &mut Conjunction, node_id_to_pos: &BTreeMap<usize, usize>) -> usize { 
    // get left and right index
    if let (Some(left_id), Some(right_id)) = (join["left"].as_u64(), join["right"].as_u64()) {
        let left_id = left_id as usize;
        let right_id = right_id as usize;
        if let (Some(pos_left),Some(pos_right)) = (node_id_to_pos.get(&left_id),node_id_to_pos.get(&right_id)) {
            if let Some(op) = join["op"].as_str() {

            }
        }
    }
    
    unimplemented!()
}

fn is_regex(json_node : &serde_json::Value) -> bool {
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
