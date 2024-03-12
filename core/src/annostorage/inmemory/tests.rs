use super::*;

use crate::{
    graph::{ANNIS_NS, NODE_NAME},
    types::NodeID,
};

#[test]
fn insert_same_anno() {
    let test_anno = Annotation {
        key: AnnoKey {
            name: "anno1".into(),
            ns: "annis".into(),
        },
        val: "test".into(),
    };
    let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
    a.insert(1, test_anno.clone()).unwrap();
    a.insert(1, test_anno.clone()).unwrap();
    a.insert(2, test_anno.clone()).unwrap();
    a.insert(3, test_anno).unwrap();

    assert_eq!(3, a.number_of_annotations().unwrap());
    assert_eq!(3, a.by_container.len());
    assert_eq!(1, a.by_anno.len());
    assert_eq!(1, a.anno_keys.len());

    assert_eq!(
        "test",
        a.get_value_for_item(
            &3,
            &AnnoKey {
                name: "anno1".into(),
                ns: "annis".into()
            }
        )
        .unwrap()
        .unwrap()
    );
}

#[test]
fn get_all_for_node() {
    let test_anno1 = Annotation {
        key: AnnoKey {
            name: "anno1".into(),
            ns: "annis1".into(),
        },
        val: "test".into(),
    };
    let test_anno2 = Annotation {
        key: AnnoKey {
            name: "anno2".into(),
            ns: "annis2".into(),
        },
        val: "test".into(),
    };
    let test_anno3 = Annotation {
        key: AnnoKey {
            name: "anno3".into(),
            ns: "annis1".into(),
        },
        val: "test".into(),
    };

    let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
    a.insert(1, test_anno1.clone()).unwrap();
    a.insert(1, test_anno2.clone()).unwrap();
    a.insert(1, test_anno3.clone()).unwrap();

    assert_eq!(3, a.number_of_annotations().unwrap());

    let all = a.get_annotations_for_item(&1).unwrap();
    assert_eq!(3, all.len());

    assert_eq!(test_anno1, all[0]);
    assert_eq!(test_anno2, all[1]);
    assert_eq!(test_anno3, all[2]);
}

#[test]
fn remove() {
    let test_anno = Annotation {
        key: AnnoKey {
            name: "anno1".into(),
            ns: "annis1".into(),
        },
        val: "test".into(),
    };
    let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
    a.insert(1, test_anno.clone()).unwrap();

    assert_eq!(1, a.number_of_annotations().unwrap());
    assert_eq!(1, a.by_container.len());
    assert_eq!(1, a.by_anno.len());
    assert_eq!(1, a.anno_key_sizes.len());
    assert_eq!(&1, a.anno_key_sizes.get(&test_anno.key).unwrap());

    a.remove_annotation_for_item(&1, &test_anno.key).unwrap();

    assert_eq!(0, a.number_of_annotations().unwrap());
    assert_eq!(0, a.by_container.len());
    assert_eq!(0, a.by_anno.len());
    assert_eq!(&0, a.anno_key_sizes.get(&test_anno.key).unwrap_or(&0));
}

#[test]
fn get_node_id_from_name() {
    let key = NODE_NAME_KEY.as_ref().clone();
    let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
    a.insert(
        1,
        Annotation {
            key: key.clone(),
            val: "node1".into(),
        },
    )
    .unwrap();
    a.insert(
        2,
        Annotation {
            key: key.clone(),
            val: "node2".into(),
        },
    )
    .unwrap();
    a.insert(
        3,
        Annotation {
            key: key.clone(),
            val: "node3".into(),
        },
    )
    .unwrap();

    assert_eq!(Some(1), a.get_node_id_from_name("node1").unwrap());
    assert_eq!(Some(2), a.get_node_id_from_name("node2").unwrap());
    assert_eq!(Some(3), a.get_node_id_from_name("node3").unwrap());
    assert_eq!(true, a.has_node_name("node1").unwrap());
    assert_eq!(true, a.has_node_name("node2").unwrap());
    assert_eq!(true, a.has_node_name("node3").unwrap());

    assert_eq!(None, a.get_node_id_from_name("node0").unwrap());
    assert_eq!(None, a.get_node_id_from_name("").unwrap());
    assert_eq!(None, a.get_node_id_from_name("somenode").unwrap());
    assert_eq!(false, a.has_node_name("node0").unwrap());
    assert_eq!(false, a.has_node_name("").unwrap());
    assert_eq!(false, a.has_node_name("somenode").unwrap());
}

#[test]
fn regex_search() {
    let key = NODE_NAME_KEY.as_ref().clone();
    let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
    a.insert(
        0,
        Annotation {
            key: key.clone(),
            val: "_ABC".into(),
        },
    )
    .unwrap();
    a.insert(
        1,
        Annotation {
            key: key.clone(),
            val: "AAA".into(),
        },
    )
    .unwrap();
    a.insert(
        2,
        Annotation {
            key: key.clone(),
            val: "AAB".into(),
        },
    )
    .unwrap();
    a.insert(
        3,
        Annotation {
            key: key.clone(),
            val: "AAC".into(),
        },
    )
    .unwrap();

    a.insert(
        4,
        Annotation {
            key: key.clone(),
            val: "B".into(),
        },
    )
    .unwrap();

    let result: Result<Vec<_>> = a
        .regex_anno_search(Some(ANNIS_NS), NODE_NAME, "A.*", false)
        .map_ok(|m| m.node)
        .collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![1, 2, 3], result);

    let result: Result<Vec<_>> = a
        .regex_anno_search(Some(ANNIS_NS), NODE_NAME, ".A.", false)
        .map_ok(|m| m.node)
        .collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![1, 2, 3], result);

    let result: Result<Vec<_>> = a
        .regex_anno_search(Some(ANNIS_NS), NODE_NAME, "_.*", false)
        .map_ok(|m| m.node)
        .collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![0], result);

    let result: Result<Vec<_>> = a
        .regex_anno_search(Some(ANNIS_NS), NODE_NAME, "(A|B).*", false)
        .map_ok(|m| m.node)
        .collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![1, 2, 3, 4], result);

    let result: Result<Vec<_>> = a
        .regex_anno_search(Some(ANNIS_NS), NODE_NAME, "C.*", false)
        .map_ok(|m| m.node)
        .collect();
    let result = result.unwrap();

    assert_eq!(0, result.len());
}
