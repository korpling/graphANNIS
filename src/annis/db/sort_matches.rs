use annis::db::graphstorage::GraphStorage;
use annis::db::token_helper::TokenHelper;
use annis::db::Match;
use annis::types::NodeID;
use rustc_hash::FxHashMap;
use std;
use std::cmp::Ordering;

pub fn compare_matchgroup_by_text_pos(
    m1: &[Match],
    m2: &[Match],
    node_to_path: &FxHashMap<NodeID, (Vec<&str>, &str)>,
    token_helper: Option<&TokenHelper>,
    gs_order: Option<&GraphStorage>,
) -> Ordering {
    for i in 0..std::cmp::min(m1.len(), m2.len()) {
        let element_cmp =
            compare_match_by_text_pos(&m1[i], &m2[i], node_to_path, token_helper, gs_order);
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
    node_to_path: &FxHashMap<NodeID, (Vec<&str>, &str)>,
    token_helper: Option<&TokenHelper>,
    gs_order: Option<&GraphStorage>,
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
            if let (Some(token_helper), Some(gs_order)) = (token_helper, gs_order) {
                if let (Some(m1_lefttok), Some(m2_lefttok)) = (
                    token_helper.left_token_for(m1.node),
                    token_helper.left_token_for(m2.node),
                ) {
                    if gs_order.is_connected(
                        &m1_lefttok,
                        &m2_lefttok,
                        1,
                        std::ops::Bound::Unbounded,
                    ) {
                        return Ordering::Less;
                    } else if gs_order.is_connected(
                        &m2_lefttok,
                        &m1_lefttok,
                        1,
                        std::ops::Bound::Unbounded,
                    ) {
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

