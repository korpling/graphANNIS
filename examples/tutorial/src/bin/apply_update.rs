use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();

    let mut g = GraphUpdate::new()?;

    // First add the node (with the default type "node"),
    // then all node labels for the node.
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t1".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t1".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "That".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t2".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t2".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "is".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t3".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t3".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "a".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t4".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t4".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "Category".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t5".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t5".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "3".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t6".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t6".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "storm".to_string(),
    })?;

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t7".to_string(),
        node_type: "node".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t7".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: ".".to_string(),
    })?;

    // Add the ordering edges to specify token order.
    // The names of the source and target nodes are given as in the enum as fields,
    // followed by the component layer, type and name.
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t1".to_string(),
        target_node: "tutorial/doc1#t2".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t2".to_string(),
        target_node: "tutorial/doc1#t3".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t3".to_string(),
        target_node: "tutorial/doc1#t4".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t4".to_string(),
        target_node: "tutorial/doc1#t5".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t5".to_string(),
        target_node: "tutorial/doc1#t6".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t6".to_string(),
        target_node: "tutorial/doc1#t7".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    })?;

    // Insert the changes in the corpus with the name "tutorial"
    cs.apply_update("tutorial", &mut g).unwrap();

    // List newly created corpus
    let corpora = cs.list().unwrap();
    let corpus_names: Vec<String> = corpora
        .into_iter()
        .map(|corpus_info| corpus_info.name)
        .collect();
    println!("{:?}", corpus_names);

    Ok(())
}
