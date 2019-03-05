use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();

    let mut g = GraphUpdate::new();

    // First add the node (with the default type "node"),
    // then all node labels for the node.
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t1".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t1".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "That".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t2".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t2".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "is".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t3".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t3".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "a".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t4".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t4".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "Category".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t5".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t5".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "3".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t6".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t6".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: "storm".to_owned(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t7".to_owned(),
        node_type: "node".to_owned(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t7".to_owned(),
        anno_ns: "annis".to_owned(),
        anno_name: "tok".to_owned(),
        anno_value: ".".to_owned(),
    });

    // Add the ordering edges to specify token order.
    // The names of the source and target nodes are given as in the enum as fields,
    // followed by the component layer, type and name.
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t1".to_owned(),
        target_node: "tutorial/doc1#t2".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t2".to_owned(),
        target_node: "tutorial/doc1#t3".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t3".to_owned(),
        target_node: "tutorial/doc1#t4".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t4".to_owned(),
        target_node: "tutorial/doc1#t5".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t5".to_owned(),
        target_node: "tutorial/doc1#t6".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t6".to_owned(),
        target_node: "tutorial/doc1#t7".to_owned(),
        layer: "annis".to_owned(),
        component_type: "Ordering".to_owned(),
        component_name: "".to_owned(),
    });

    // Insert the changes in the corpus with the name "tutorial"
    cs.apply_update("tutorial", &mut g).unwrap();

    // List newly created corpus
    let corpora = cs.list().unwrap();
    let corpus_names: Vec<String> = corpora
        .into_iter()
        .map(|corpus_info| corpus_info.name)
        .collect();
    println!("{:?}", corpus_names);
}
