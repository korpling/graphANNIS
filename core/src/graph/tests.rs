use super::*;
use crate::types::{AnnoKey, Annotation, DefaultComponentType, Edge};

#[test]
fn create_writeable_gs() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();

    let anno_key = AnnoKey {
        ns: "test".into(),
        name: "edge_anno".into(),
    };
    let anno_val = "testValue".into();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());
    let gs: &mut dyn WriteableGraphStorage = db.get_or_create_writable(&component).unwrap();

    gs.add_edge(Edge {
        source: 0,
        target: 1,
    })
    .unwrap();

    gs.add_edge_annotation(
        Edge {
            source: 0,
            target: 1,
        },
        Annotation {
            key: anno_key,
            val: anno_val,
        },
    )
    .unwrap();
}

#[test]
fn open_existing_graph_storage() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());
    db.get_or_create_writable(&component).unwrap();

    let tmp = tempfile::tempdir().unwrap();

    db.save_to(tmp.path()).unwrap();

    let mut db = Graph::new(false).unwrap();
    db.open(tmp.path()).unwrap();
    assert_eq!(1, db.components.len());
    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(false, gs.unwrap().is_some());

    db.ensure_loaded(&component).unwrap();
    assert_eq!(1, db.components.len());
    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(true, gs.unwrap().is_some());
}

#[test]
fn load_existing_graph_wit_preload() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());
    db.get_or_create_writable(&component).unwrap();

    let tmp = tempfile::tempdir().unwrap();

    db.save_to(tmp.path()).unwrap();

    let mut db = Graph::new(false).unwrap();
    #[allow(deprecated)]
    db.load_from(tmp.path(), true).unwrap();
    assert_eq!(1, db.components.len());
    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(true, gs.unwrap().is_some());
}

#[test]
fn load_existing_graph_storage_parallel() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());
    db.get_or_create_writable(&component).unwrap();

    let tmp = tempfile::tempdir().unwrap();

    db.save_to(tmp.path()).unwrap();

    let mut db = Graph::new(false).unwrap();
    db.open(tmp.path()).unwrap();
    assert_eq!(1, db.components.len());
    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(false, gs.unwrap().is_some());

    db.ensure_loaded_parallel(&[component.clone()]).unwrap();
    assert_eq!(1, db.components.len());
    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(true, gs.unwrap().is_some());
}

#[test]
fn load_non_existing_graph_storage_parallel() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());

    let tmp = tempfile::tempdir().unwrap();

    db.save_to(tmp.path()).unwrap();

    let mut db = Graph::new(false).unwrap();
    db.open(tmp.path()).unwrap();

    db.ensure_loaded_parallel(&[component]).unwrap();
    assert_eq!(0, db.components.len());
}

#[test]
fn load_with_wal_file() {
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();
    let example_node = 0;
    db.node_annos
        .insert(
            example_node,
            Annotation {
                key: NODE_TYPE_KEY.as_ref().clone(),
                val: "corpus".into(),
            },
        )
        .unwrap();
    db.node_annos
        .insert(
            example_node,
            Annotation {
                key: NODE_NAME_KEY.as_ref().clone(),
                val: "root".into(),
            },
        )
        .unwrap();

    let tmp = tempfile::tempdir().unwrap();
    // Save and remember the location, so that updates are recorded in a WAL
    // file
    db.persist_to(tmp.path()).unwrap();

    // Add an node annotation with apply_update
    let mut u = GraphUpdate::new();
    u.add_event(UpdateEvent::AddNodeLabel {
        node_name: "root".into(),
        anno_ns: "example".into(),
        anno_name: "anno-name".into(),
        anno_value: "anno-value".into(),
    })
    .unwrap();
    db.apply_update(&mut u, |_| {}).unwrap();

    std::mem::drop(db);

    // Check that loading the database again contains the changes
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();
    db.open(tmp.path()).unwrap();
    db.ensure_loaded_all().unwrap();
    let anno_value = db
        .node_annos
        .get_value_for_item(
            &example_node,
            &AnnoKey {
                name: "anno-name".into(),
                ns: "example".into(),
            },
        )
        .unwrap()
        .unwrap();
    assert_eq!("anno-value", anno_value);
}

#[test]
fn import_from_existing() {
    let mut other = Graph::<DefaultComponentType>::new(false).unwrap();

    let component = Component::new(DefaultComponentType::Edge, "test".into(), "dep".into());
    other.get_or_create_writable(&component).unwrap();

    let tmp1 = tempfile::tempdir().unwrap();
    let tmp2 = tempfile::tempdir().unwrap();

    other.save_to(tmp1.path()).unwrap();
    other.save_to(tmp2.path()).unwrap();

    // Open the first copy
    let mut db = Graph::new(false).unwrap();
    db.open(tmp1.path()).unwrap();
    assert_eq!(Some(tmp1.path().to_path_buf()), db.location);

    // Import the second location, the location should have been cleared and the
    // graph storages loaded from the second location.
    db.import(tmp2.path()).unwrap();

    assert_eq!(None, db.location);
    assert_eq!(1, db.components.len());

    let gs = db.components.get(&component);
    assert_eq!(true, gs.is_some());
    assert_eq!(true, gs.unwrap().is_some());
}

#[test]
fn update_without_updating_stats() {
    // Create a minimal graph
    let mut db = Graph::<DefaultComponentType>::new(false).unwrap();
    for example_node in 0..10 {
        db.node_annos
            .insert(
                example_node,
                Annotation {
                    key: NODE_TYPE_KEY.as_ref().clone(),
                    val: "corpus".into(),
                },
            )
            .unwrap();
        db.node_annos
            .insert(
                example_node,
                Annotation {
                    key: NODE_NAME_KEY.as_ref().clone(),
                    val: format!("n{example_node}").into(),
                },
            )
            .unwrap();
    }

    db.calculate_all_statistics().unwrap();
    let old_guess = db
        .node_annos
        .guess_max_count(Some("annis"), "node_name", "", &char::MAX.to_string())
        .unwrap();
    assert_eq!(10, old_guess);

    // Add a complete unknown annotation name to the graph
    let mut updates = GraphUpdate::new();
    for example_node in 0..10 {
        updates
            .add_event(UpdateEvent::AddNodeLabel {
                node_name: format!("n{example_node}").into(),
                anno_ns: "test".into(),
                anno_name: "test".into(),
                anno_value: "nothing".into(),
            })
            .unwrap();
    }

    // Apply an update and check that the new annotation layer is still unknown
    // to the statistics
    db.apply_update_keep_statistics(&mut updates, |_| {})
        .unwrap();
    let new_guess = db
        .node_annos
        .guess_max_count(Some("test"), "test", "", &char::MAX.to_string())
        .unwrap();
    assert_eq!(0, new_guess);
}
