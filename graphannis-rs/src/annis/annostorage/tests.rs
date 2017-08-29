use super::*;
use annis::NodeID;

#[test]
fn insert_same_anno() {
    let test_anno = Annotation {
        key: AnnoKey { name: 1, ns: 1 },
        val: 123,
    };
    let mut a: AnnoStorage<NodeID> = AnnoStorage::new();
    a.insert(1, test_anno.clone());
    a.insert(1, test_anno.clone());
    a.insert(2, test_anno.clone());
    a.insert(3, test_anno);

    assert_eq!(3, a.len());
    assert_eq!(3, a.by_container.len());
    assert_eq!(1, a.by_anno.len());
    assert_eq!(1, a.anno_keys.len());

    assert_eq!(123, a.get(&3, &AnnoKey { name: 1, ns: 1 }).unwrap().clone());
}

#[test]
fn get_all_for_node() {
    let test_anno1 = Annotation {
        key: AnnoKey { name: 1, ns: 1 },
        val: 123,
    };
    let test_anno2 = Annotation {
        key: AnnoKey { name: 2, ns: 2 },
        val: 123,
    };
    let test_anno3 = Annotation {
        key: AnnoKey { name: 3, ns: 1 },
        val: 123,
    };

    let mut a: AnnoStorage<NodeID> = AnnoStorage::new();
    a.insert(1, test_anno1.clone());
    a.insert(1, test_anno2.clone());
    a.insert(1, test_anno3.clone());

    assert_eq!(3, a.len());

    let all = a.get_all(&1);
    assert_eq!(3, all.len());

    assert_eq!(test_anno1, all[0]);
    assert_eq!(test_anno2, all[1]);
    assert_eq!(test_anno3, all[2]);
}

#[test]
fn remove() {
    let test_anno = Annotation {
        key: AnnoKey { name: 1, ns: 1 },
        val: 123,
    };
    let mut a: AnnoStorage<NodeID> = AnnoStorage::new();
    a.insert(1, test_anno.clone());

    assert_eq!(1, a.len());
    assert_eq!(1, a.by_container.len());
    assert_eq!(1, a.by_anno.len());
    assert_eq!(1, a.anno_keys.len());
    assert_eq!(&1, a.anno_keys.get(&test_anno.key).unwrap());

    a.remove(&1, &test_anno.key);

    assert_eq!(0, a.len());
    assert_eq!(0, a.by_container.len());
    assert_eq!(0, a.by_anno.len());
    assert_eq!(&0, a.anno_keys.get(&test_anno.key).unwrap_or(&0));

}