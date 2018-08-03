use serde_json;

use exec::nodesearch::NodeSearchSpec;
use graphdb::GraphDB;
use operator::edge_op::EdgeAnnoSearchSpec;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;

use operator::{
    DominanceSpec, IdenticalCoverageSpec, IdenticalNodeSpec, InclusionSpec, OperatorSpec,
    OverlapSpec, PartOfSubCorpusSpec, PointingSpec, PrecedenceSpec,
};

use std::collections::BTreeMap;

pub fn parse<'a>(query_as_string: &str, db: &GraphDB) -> Option<Disjunction<'a>> {
    let root: serde_json::Value = serde_json::from_str(query_as_string).ok()?;

    let mut conjunctions: Vec<Conjunction> = Vec::new();
    // iterate over all alternatives
    let alternatives = root.get("alternatives")?.as_array()?;

    for alt in alternatives.iter() {
        let mut q = Conjunction::new();

        let mut first_node_pos: Option<usize> = None;

        // add all nodes
        let mut node_id_to_pos: BTreeMap<usize, usize> = BTreeMap::new();
        if let &serde_json::Value::Object(ref nodes) = alt.get("nodes")? {
            for (node_name, node) in nodes.iter() {
                if let Some(node_obj) = node.as_object() {
                    if let Ok(ref node_id) = node_name.parse::<u64>() {
                        let node_id = node_id.clone() as usize;

                        let pos = parse_node(node_obj, &mut q)?;
                        if first_node_pos.is_none() {
                            first_node_pos = Some(pos.clone());
                        }
                        node_id_to_pos.insert(node_id, pos);
                    }
                }
            }
        }

        // add all joins
        if let &serde_json::Value::Array(ref joins) = alt.get("joins")? {
            for j in joins.iter() {
                if let &serde_json::Value::Object(ref j_obj) = j {
                    parse_join(j_obj, &mut q, &node_id_to_pos, db);
                }
            }
        }

        // add all meta-data
        if let Some(meta_obj) = alt.get("meta") {
            let mut first_meta_idx: Option<usize> = None;

            if let Some(meta_array) = meta_obj.as_array() {
                for m in meta_array.iter() {
                    // add an artificial node that describes the document/corpus node
                    if let Some(meta_node_idx) = add_node_annotation(
                        &mut q,
                        m.get("namespace").and_then(|n| n.as_str()),
                        m.get("name").and_then(|n| n.as_str()),
                        m.get("value").and_then(|n| n.as_str()),
                        is_regex(m),
                        true,
                        None,
                    ) {
                        if let Some(first_meta_idx) = first_meta_idx {
                            // avoid nested loops by joining additional meta nodes with a "identical node"
                            q.add_operator(
                                Box::new(IdenticalNodeSpec {}),
                                first_meta_idx,
                                meta_node_idx,
                            );
                        } else if let Some(first_node_pos) = first_node_pos {
                            first_meta_idx = Some(meta_node_idx);
                            // add a special join to the first node of the query
                            q.add_operator(
                                Box::new(PartOfSubCorpusSpec::new(1, usize::max_value())),
                                first_node_pos,
                                meta_node_idx,
                            );
                            // Also make sure the matched node is actually a document
                            // (the @* could match anything in the hierarchy, including the toplevel corpus)
                            if let Some(doc_anno_idx) = add_node_annotation(
                                &mut q,
                                Some("annis"),
                                Some("doc"),
                                None,
                                false,
                                true,
                                None,
                            ) {
                                q.add_operator(
                                    Box::new(IdenticalNodeSpec {}),
                                    meta_node_idx,
                                    doc_anno_idx,
                                );
                            }
                        }
                    }
                }
            }
        }

        conjunctions.push(q);
    }

    if !conjunctions.is_empty() {
        return Some(Disjunction::new(conjunctions));
    }

    return None;
}

fn parse_node(
    node: &serde_json::Map<String, serde_json::Value>,
    q: &mut Conjunction,
) -> Option<usize> {
    let variable = node.get("variable").and_then(|s| s.as_str());
    // annotation search?
    if node.contains_key("nodeAnnotations") {
        if let serde_json::Value::Array(ref a) = node["nodeAnnotations"] {
            if !a.is_empty() {
                // get the first one
                let a = &a[0];
                return add_node_annotation(
                    q,
                    a.get("namespace").and_then(|n| n.as_str()),
                    a.get("name").and_then(|n| n.as_str()),
                    a.get("value").and_then(|n| n.as_str()),
                    is_regex(a),
                    false,
                    variable,
                );
            }
        }
    }

    // check for special non-annotation search constructs
    // token search?
    if (node.contains_key("spannedText") && node["spannedText"].is_string())
        || (node.contains_key("token")
            && node["token"].is_boolean()
            && node["token"].as_bool() == Some(true))
    {
        let spanned = node.get("spannedText").and_then(|s| s.as_str());

        if let Some(tok_val) = spanned {
            let mut leafs_only = false;
            if let Some(is_token) = node["token"].as_bool() {
                if is_token {
                    // special treatment for explicit searches for token (tok="...)
                    leafs_only = true;
                }
            }
            if node.contains_key("spanTextMatching")
                && node["spanTextMatching"].as_str() == Some("REGEXP_EQUAL")
            {
                return Some(q.add_node(
                    NodeSearchSpec::RegexTokenValue {
                        val: String::from(tok_val),
                        leafs_only,
                    },
                    variable,
                ));
            } else {
                return Some(q.add_node(
                    NodeSearchSpec::ExactTokenValue {
                        val: String::from(tok_val),
                        leafs_only,
                    },
                    variable,
                ));
            }
        } else {
            return Some(q.add_node(NodeSearchSpec::AnyToken, variable));
        }
    } else {
        // just search for any node
        return Some(q.add_node(NodeSearchSpec::AnyNode, variable));
    }
}

fn parse_join(
    join: &serde_json::Map<String, serde_json::Value>,
    q: &mut Conjunction,
    node_id_to_pos: &BTreeMap<usize, usize>,
    db: &GraphDB,
) {
    // get left and right index
    if let (Some(left_id), Some(right_id)) = (
        join.get("left").and_then(|n| n.as_u64()),
        join.get("right").and_then(|n| n.as_u64()),
    ) {
        let left_id = left_id as usize;
        let right_id = right_id as usize;
        if let (Some(pos_left), Some(pos_right)) =
            (node_id_to_pos.get(&left_id), node_id_to_pos.get(&right_id))
        {
            let spec_opt: Option<Box<OperatorSpec>> = match join.get("op").and_then(|s| s.as_str())
            {
                Some("Precedence") => {
                    let min_dist = join.get("minDistance").and_then(|n| n.as_u64());
                    let max_dist = join.get("maxDistance").and_then(|n| n.as_u64());
                    let seg_name = join.get("segmentation-name").and_then(|s| s.as_str());

                    let spec = PrecedenceSpec {
                        segmentation: seg_name.map(|s| String::from(s)),
                        min_dist: min_dist.unwrap_or(1) as usize,
                        max_dist: max_dist.unwrap_or(1) as usize,
                    };
                    Some(Box::new(spec))
                }
                Some("IdenticalCoverage") => {
                    let spec = IdenticalCoverageSpec {};
                    Some(Box::new(spec))
                }
                Some("Inclusion") => {
                    let spec = InclusionSpec {};
                    Some(Box::new(spec))
                }
                Some("Overlap") => {
                    let spec = OverlapSpec {};
                    Some(Box::new(spec))
                }
                Some("Dominance") => {
                    let min_dist = join
                        .get("minDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;
                    let max_dist = join
                        .get("maxDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;

                    let (min_dist, max_dist) = if min_dist == 0 && max_dist == 0 {
                        // unlimited range
                        (1, usize::max_value())
                    } else {
                        (min_dist, max_dist)
                    };

                    let name = join.get("name").and_then(|n| n.as_str());
                    let edge_anno = join
                        .get("edgeAnnotations")
                        .and_then(|a| a.as_array())
                        .and_then(|a| get_edge_anno(&a[0]));
                    let spec =
                        DominanceSpec::new(db, name.unwrap_or(""), min_dist, max_dist, edge_anno);
                    Some(Box::new(spec))
                }
                Some("Pointing") => {
                    let min_dist = join
                        .get("minDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;
                    let max_dist = join
                        .get("maxDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;

                    let name = join.get("name").and_then(|n| n.as_str());
                    let edge_anno = join
                        .get("edgeAnnotations")
                        .and_then(|a| a.as_array())
                        .and_then(|a| get_edge_anno(&a[0]));

                    let (min_dist, max_dist) = if min_dist == 0 && max_dist == 0 {
                        // unlimited range
                        (1, usize::max_value())
                    } else {
                        (min_dist, max_dist)
                    };

                    let spec =
                        PointingSpec::new(db, name.unwrap_or(""), min_dist, max_dist, edge_anno);
                    Some(Box::new(spec))
                }
                Some("PartOfSubcorpus") => {
                    let min_dist = join
                        .get("minDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;
                    let max_dist = join
                        .get("maxDistance")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(1) as usize;

                    let (min_dist, max_dist) = if min_dist == 0 && max_dist == 0 {
                        // unlimited range
                        (1, usize::max_value())
                    } else {
                        (min_dist, max_dist)
                    };

                    let spec = PartOfSubCorpusSpec::new(min_dist, max_dist);
                    Some(Box::new(spec))
                }
                Some("IdenticalNode") => Some(Box::new(IdenticalNodeSpec)),
                // TODO: add more operators
                _ => None,
            };
            if let Some(spec) = spec_opt {
                q.add_operator(spec, pos_left.clone() as usize, pos_right.clone() as usize);
            }
        }
    }
}

fn get_edge_anno(json_node: &serde_json::Value) -> Option<EdgeAnnoSearchSpec> {
    if let Some(tm) = json_node.get("textMatching").and_then(|n| n.as_str()) {
        if tm == "EXACT_EQUAL" {
            let name = json_node.get("name")?.as_str()?;

            return Some(EdgeAnnoSearchSpec::ExactValue {
                ns: json_node
                    .get("namespace")
                    .and_then(|n| n.as_str())
                    .map(|s| String::from(s)),
                val: json_node
                    .get("value")
                    .and_then(|n| n.as_str())
                    .map(|s| String::from(s)),
                name: String::from(name),
            });
        }
        // TODO: what about regex?
    }
    None
}

fn is_regex(json_node: &serde_json::Value) -> bool {
    if let Some(tm) = json_node.get("textMatching").and_then(|n| n.as_str()) {
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
    regex: bool,
    is_meta: bool,
    variable: Option<&str>,
) -> Option<usize> {
    if let Some(name_val) = name {
        // TODO: replace regex with normal text matching if this is not an actual regular expression

        // search for the value
        if regex {
            if let Some(val) = value {
                let mut n: NodeSearchSpec = NodeSearchSpec::new_regex(ns, name_val, val, is_meta);
                return Some(q.add_node(n, variable));
            }
        } else {
            // has namespace?
            let mut n: NodeSearchSpec = NodeSearchSpec::new_exact(ns, name_val, value, is_meta);
            return Some(q.add_node(n, variable));
        }
    }
    return None;
}
