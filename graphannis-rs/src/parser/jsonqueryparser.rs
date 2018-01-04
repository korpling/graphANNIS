use json;
use json::JsonValue;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;

use std::collections::BTreeMap;

pub fn parse(query_as_string : &str) -> Option<Disjunction> {
    let parsed = json::parse(query_as_string);

    if let Ok(root) = parsed {

        let mut conjunctions : Vec<Conjunction> = Vec::new();
        // iterate over all alternatives
        match root["alternatives"] {
            JsonValue::Array (ref alternatices) => {
                for alt in alternatices.iter() {

                    let mut q = Conjunction::new();

                    // add all nodes
                    let mut node_id_to_pos : BTreeMap<usize, usize> = BTreeMap::new();
                    if let JsonValue::Object(ref nodes) = alt["nodes"] {
                        for (node_name, node) in nodes.iter() {
                            if let JsonValue::Object(ref node_object) = *node {
                                if let Ok(ref node_id) = node_name.parse::<usize>(){
                                    let pos = parse_node(node_object, &mut q);
                                }
                            }

                        
                        }
                    }

                    conjunctions.push(q);
                    unimplemented!();
                }
            },
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

fn parse_node(node : &json::object::Object, q : &mut Conjunction) -> usize {
    unimplemented!()
}