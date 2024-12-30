use insta::assert_snapshot;

use super::*;

#[test]
fn serialize_deserialize_bincode() {
    let example_updates = vec![
        UpdateEvent::AddNode {
            node_name: "parent".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddNode {
            node_name: "child".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddEdge {
            source_node: "child".into(),
            target_node: "parent".into(),
            layer: "annis".into(),
            component_type: "PartOf".into(),
            component_name: "".into(),
        },
    ];

    let mut updates = GraphUpdate::new();
    for e in example_updates.iter() {
        updates.add_event(e.clone()).unwrap();
    }

    let seralized_bytes: Vec<u8> = bincode::serialize(&updates).unwrap();
    let deseralized_update: GraphUpdate = bincode::deserialize(&seralized_bytes).unwrap();

    assert_eq!(3, deseralized_update.len().unwrap());
    let deseralized_events: Vec<UpdateEvent> = deseralized_update
        .iter()
        .unwrap()
        .map(|e| e.unwrap().1)
        .collect();
    assert_eq!(example_updates, deseralized_events);
}

#[test]
fn serialize_deserialize_bincode_empty() {
    let example_updates: Vec<UpdateEvent> = Vec::new();

    let mut updates = GraphUpdate::new();
    for e in example_updates.iter() {
        updates.add_event(e.clone()).unwrap();
    }

    let seralized_bytes: Vec<u8> = bincode::serialize(&updates).unwrap();
    let deseralized_update: GraphUpdate = bincode::deserialize(&seralized_bytes).unwrap();

    assert_eq!(0, deseralized_update.len().unwrap());
    assert_eq!(true, deseralized_update.is_empty().unwrap());
}

#[test]
fn serialize_json() {
    let example_updates = vec![
        UpdateEvent::AddNode {
            node_name: "parent".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddNode {
            node_name: "child".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddEdge {
            source_node: "child".into(),
            target_node: "parent".into(),
            layer: "annis".into(),
            component_type: "PartOf".into(),
            component_name: "".into(),
        },
    ];

    let mut updates = GraphUpdate::new();
    for e in example_updates.iter() {
        updates.add_event(e.clone()).unwrap();
    }

    let seralized_string = serde_json::to_string_pretty(&updates).unwrap();
    assert_snapshot!(seralized_string);
}

#[test]
fn serialize_deserialize_json() {
    let example_updates = vec![
        UpdateEvent::AddNode {
            node_name: "parent".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddNode {
            node_name: "child".into(),
            node_type: "corpus".into(),
        },
        UpdateEvent::AddEdge {
            source_node: "child".into(),
            target_node: "parent".into(),
            layer: "annis".into(),
            component_type: "PartOf".into(),
            component_name: "".into(),
        },
    ];

    let mut updates = GraphUpdate::new();
    for e in example_updates.iter() {
        updates.add_event(e.clone()).unwrap();
    }

    let seralized_string = serde_json::to_string_pretty(&updates).unwrap();
    let deseralized_update: GraphUpdate = serde_json::from_str(&seralized_string).unwrap();

    assert_eq!(3, deseralized_update.len().unwrap());
    let deseralized_events: Vec<UpdateEvent> = deseralized_update
        .iter()
        .unwrap()
        .map(|e| e.unwrap().1)
        .collect();
    assert_eq!(example_updates, deseralized_events);
}
