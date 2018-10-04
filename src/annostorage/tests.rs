use super::*;
use NodeID;

#[test]
fn insert_same_anno() {
    let test_anno = Annotation {
        key: Arc::from(AnnoKey { name: "anno1".to_owned(), ns: "annis".to_owned()}),
        val: Arc::from("test".to_owned()),
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

    assert_eq!("test", a.get(&3, &AnnoKey { name: "anno1".to_owned(), ns: "annis".to_owned() }).unwrap().as_ref());
}

#[test]
fn get_all_for_node() {
    let test_anno1 = Annotation {
        key: Arc::from(AnnoKey { name: "anno1".to_owned(), ns: "annis1".to_owned() }),
        val: Arc::from("test".to_owned()),
    };
    let test_anno2 = Annotation {
        key: Arc::from(AnnoKey { name: "anno2".to_owned(), ns: "annis2".to_owned() }),
        val: Arc::from("test".to_owned()),
    };
    let test_anno3 = Annotation {
        key: Arc::from(AnnoKey { name: "anno3".to_owned(), ns: "annis1".to_owned() }),
        val: Arc::from("test".to_owned()),
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
        key: Arc::from(AnnoKey { name: "anno1".to_owned(), ns: "annis1".to_owned() }),
        val: Arc::from("test".to_owned()),
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