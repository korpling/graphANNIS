use crate::annis::db::annostorage::AnnoStorage;
use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::Match;
use crate::annis::db::{ANNIS_NS, NODE_NAME};
use crate::annis::types::{AnnoKey, NodeID};
use std;
use std::cmp::Ordering;
use std::ffi::CString;

pub fn compare_matchgroup_by_text_pos(
    m1: &[Match],
    m2: &[Match],
    node_annos: &AnnoStorage<NodeID>,
    token_helper: Option<&TokenHelper>,
    gs_order: Option<&GraphStorage>,
    use_local_collation: bool,
) -> Ordering {
    for i in 0..std::cmp::min(m1.len(), m2.len()) {
        let element_cmp = compare_match_by_text_pos(
            &m1[i],
            &m2[i],
            node_annos,
            token_helper,
            gs_order,
            use_local_collation,
        );
        if element_cmp != Ordering::Equal {
            return element_cmp;
        }
    }
    // sort shorter vectors before larger ones
    m1.len().cmp(&m2.len())
}

fn split_path_and_nodename(full_node_name: &str) -> (&str, &str) {
    let hash_pos = full_node_name.rfind('#');
    let path: &str = &full_node_name[0..hash_pos.unwrap_or_else(|| full_node_name.len())];

    if let Some(hash_pos) = hash_pos {
        (path, &full_node_name[hash_pos + 1..])
    } else {
        (path, "")
    }
}

fn compare_document_path(p1: &str, p2: &str, use_local_collation: bool) -> std::cmp::Ordering {
    let it1 = p1.split('/').filter(|s| !s.is_empty());
    let it2 = p2.split('/').filter(|s| !s.is_empty());

    for (part1, part2) in it1.zip(it2) {
        let string_cmp = compare_string(part1, part2, use_local_collation);
        if string_cmp != std::cmp::Ordering::Equal {
            return string_cmp;
        }
    }

    // Both paths have the same prefix, check if one of them has more elements.
    // TODO: Since both iterators have been moved, they have to be recreated, there
    // should be a more efficient way of doing this.
    let length1 = p1.split('/').filter(|s| !s.is_empty()).count();
    let length2 = p2.split('/').filter(|s| !s.is_empty()).count();
    length1.cmp(&length2)
}

fn compare_string(s1: &str, s2: &str, use_local_collation: bool) -> std::cmp::Ordering {
    if use_local_collation {
        let cmp = unsafe {
            let c_s1 = CString::new(s1).unwrap_or_default();
            let c_s2 = CString::new(s2).unwrap_or_default();
            libc::strcoll(c_s1.as_ptr(), c_s2.as_ptr())
        };
        if cmp < 0 {
            return std::cmp::Ordering::Less;
        } else if cmp > 0 {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Equal;
        }
    } else {
        if s1 < s2 {
            return std::cmp::Ordering::Less;
        } else if s1 > s2 {
            return std::cmp::Ordering::Greater;
        }
        return std::cmp::Ordering::Equal;
    }
}

lazy_static! {
    static ref NODE_NAME_KEY: AnnoKey = AnnoKey {
        ns: ANNIS_NS.to_string(),
        name: NODE_NAME.to_string(),
    };
}

pub fn compare_match_by_text_pos(
    m1: &Match,
    m2: &Match,
    node_annos: &AnnoStorage<NodeID>,
    token_helper: Option<&TokenHelper>,
    gs_order: Option<&GraphStorage>,
    use_local_collation: bool,
) -> Ordering {
    if m1.node == m2.node {
        // same node, use annotation name and namespace to compare
        m1.anno_key.cmp(&m2.anno_key)
    } else {
        // get the node paths and names
        let m1_anno_val = node_annos.get_value_for_item(&m1.node, &NODE_NAME_KEY);
        let m2_anno_val = node_annos.get_value_for_item(&m2.node, &NODE_NAME_KEY);

        if let (Some(m1_anno_val), Some(m2_anno_val)) = (m1_anno_val, m2_anno_val) {
            let (m1_path, m1_name) = split_path_and_nodename(m1_anno_val);
            let (m2_path, m2_name) = split_path_and_nodename(m2_anno_val);

            // 1. compare the path
            let path_cmp = compare_document_path(m1_path, m2_path, use_local_collation);
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn tiger_doc_name_sort() {
        let p1 = "tiger2/tiger2/tiger_release_dec05_110";
        let p2 = "tiger2/tiger2/tiger_release_dec05_1_1";
        assert_eq!(
            std::cmp::Ordering::Less,
            compare_document_path(p1, p2, false)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn tiger_doc_name_sort_strcoll() {
        unsafe {
            let locale = CString::new("de_DE.UTF-8").unwrap_or_default();
            libc::setlocale(libc::LC_COLLATE, locale.as_ptr());
        }

        let p1 = "tiger2/tiger2/tiger_release_dec05_110";
        let p2 = "tiger2/tiger2/tiger_release_dec05_1_1";

        assert_eq!(
            std::cmp::Ordering::Greater,
            compare_document_path(p1, p2, true)
        );
    }
}
