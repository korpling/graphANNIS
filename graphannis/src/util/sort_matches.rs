use types::Match;
use types::Component;
use types::ComponentType;
use graphdb::GraphDB;
use std::cmp::Ordering;
use util;
use std;

pub fn compare_matchgroup_by_text_pos(m1 : &Vec<Match>, m2 : &Vec<Match>, db : &GraphDB) -> Ordering {

    for i in 0..std::cmp::min(m1.len(), m2.len()) {
        let element_cmp = compare_match_by_text_pos(&m1[i], &m2[i], db);
        if element_cmp != Ordering::Equal {
            return element_cmp;
        }
    }
    // sort shorter vectors before larger ones
    return m1.len().cmp(&m2.len());
}

pub fn compare_match_by_text_pos(m1 : &Match, m2 : &Match, db : &GraphDB) -> Ordering {
    
    if m1.node == m2.node {
        // same node, use annotation name and namespace to compare
        let m1_anno = (db.strings.str(m1.anno.key.name), db.strings.str(m1.anno.key.ns));
        let m2_anno = (db.strings.str(m2.anno.key.name), db.strings.str(m2.anno.key.ns));
        return m1_anno.cmp(&m2_anno);

    } else {
        // get the node paths and names
        let node_name_key = db.get_node_name_key();
        let m1_name_strid = db.node_annos.get(&m1.node, &node_name_key);
        let m2_name_strid = db.node_annos.get(&m1.node, &node_name_key);

        if let (Some(m1_name), Some(m2_name)) = (m1_name_strid, m2_name_strid) {

            let m1_name = db.strings.str(*m1_name);
            let m2_name = db.strings.str(*m2_name);

            if let (Some(m1_name), Some(m2_name)) = (m1_name, m2_name) {
                let (m1_path, m1_name) = util::extract_node_path(m1_name);
                let (m2_path, m2_name) = util::extract_node_path(m2_name);

                // 1. compare the path
                let path_cmp = m1_path.cmp(&m2_path);
                if path_cmp != Ordering::Equal {
                    return path_cmp;
                }
                // 2. compare the name
                let name_cmp = m1_name.cmp(&m2_name);
                if name_cmp != Ordering::Equal {
                    return name_cmp;
                }
                // 3. compare the token ordering
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
            }
        }

        // compare node IDs directly as last resort
        return m1.node.cmp(&m2.node);
    }
}