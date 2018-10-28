use annis::db::token_helper::TokenHelper;
use annis::db::Graph;
use annis::db::Match;
use annis::types::Component;
use annis::types::ComponentType;
use annis::types::NodeID;
use rustc_hash::FxHashMap;
use std;
use std::cmp::Ordering;

pub fn compare_matchgroup_by_text_pos(
    m1: &[Match],
    m2: &[Match],
    db: &Graph,
    node_to_path: &FxHashMap<NodeID, (Vec<String>, String)>,
) -> Ordering {
    for i in 0..std::cmp::min(m1.len(), m2.len()) {
        let element_cmp = compare_match_by_text_pos(&m1[i], &m2[i], db, node_to_path);
        if element_cmp != Ordering::Equal {
            return element_cmp;
        }
    }
    // sort shorter vectors before larger ones
    m1.len().cmp(&m2.len())
}

pub fn compare_match_by_text_pos(
    m1: &Match,
    m2: &Match,
    db: &Graph,
    node_to_path: &FxHashMap<NodeID, (Vec<String>, String)>,
) -> Ordering {
    if m1.node == m2.node {
        // same node, use annotation name and namespace to compare
        m1.anno_key.cmp(&m2.anno_key)
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

            if let (Some(token_helper), Some(gs_order)) =
                (TokenHelper::new(db), db.get_graphstorage(&component_order))
            {
                if let (Some(m1_lefttok), Some(m2_lefttok)) = (
                    token_helper.left_token_for(m1.node),
                    token_helper.left_token_for(m2.node),
                ) {
                    if gs_order.is_connected(&m1_lefttok, &m2_lefttok, 1, std::ops::Bound::Unbounded) {
                        return Ordering::Less;
                    } else if gs_order.is_connected(&m2_lefttok, &m1_lefttok, 1, std::ops::Bound::Unbounded)
                    {
                        return Ordering::Greater;
                    }
                }
            }

            // 3. compare the name
            let name_cmp = m1_name.cmp(&m2_name);
            if name_cmp != Ordering::Equal {
                return name_cmp;
            }
        }

        // compare node IDs directly as last resort
        m1.node.cmp(&m2.node)
    }
}
