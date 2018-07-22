use types::Match;
use types::Component;
use types::ComponentType;
use graphdb::GraphDB;
use std::cmp::Ordering;
use std;
use fxhash::FxHashMap;
use NodeID;

pub fn compare_matchgroup_by_text_pos(m1 : &Vec<Match>, m2 : &Vec<Match>, db : &GraphDB, node_to_path : &FxHashMap<NodeID, (Vec<&str>, &str)>) -> Ordering {

    for i in 0..std::cmp::min(m1.len(), m2.len()) {
        let element_cmp = compare_match_by_text_pos(&m1[i], &m2[i], db, node_to_path);
        if element_cmp != Ordering::Equal {
            return element_cmp;
        }
    }
    // sort shorter vectors before larger ones
    return m1.len().cmp(&m2.len());
}

pub fn compare_match_by_text_pos(m1 : &Match, m2 : &Match, db : &GraphDB, node_to_path : &FxHashMap<NodeID, (Vec<&str>, &str)>) -> Ordering {
    
    if m1.node == m2.node {
        // same node, use annotation name and namespace to compare
        let m1_anno = (db.strings.str(m1.anno.key.name), db.strings.str(m1.anno.key.ns));
        let m2_anno = (db.strings.str(m2.anno.key.name), db.strings.str(m2.anno.key.ns));
        return m1_anno.cmp(&m2_anno);

    } else {
        // get the node paths and names
        let m1_entry = node_to_path.get(&m1.node);
        let m2_entry = node_to_path.get(&m2.node);
        if let (Some((m1_path, m1_name)), Some((m2_path, m2_name))) = (m1_entry, m2_entry) {
             // 1. compare the path
            let path_cmp = m1_path.cmp(&m2_path);
            if path_cmp != Ordering::Equal {
                return path_cmp;
            }

            // 2. compare the token ordering
            let component_order = Component {
                ctype: ComponentType::Ordering,
                layer: String::from("annis"),
                name: String::from(""),
            };
            if let Some(gs_order) = db.get_graphstorage(&component_order) {
                if gs_order.is_connected(&m1.node, &m2.node, 1, usize::max_value()) {
                    return Ordering::Less;
                } else if gs_order.is_connected(&m2.node, &m1.node, 1, usize::max_value()) {
                    return Ordering::Greater;
                }
            }

            // 3. compare the name
            let name_cmp = m1_name.cmp(&m2_name);
            if name_cmp != Ordering::Equal {
                return name_cmp;
            }
        }

        // compare node IDs directly as last resort
        return m1.node.cmp(&m2.node);
    }
}