use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let mut g = GraphUpdate::new();
    // create the corpus and document node
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial".to_string(),
        node_type: "corpus".to_string(),
    })?;
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1".to_string(),
        node_type: "corpus".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1".to_string(),
        target_node: "tutorial".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    // add the corpus structure to the existing nodes
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t1".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t2".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t3".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t4".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t5".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t6".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t7".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    })?;
    // apply the changes
    cs.apply_update("tutorial", &mut g).unwrap();

    // get the whole document as graph
    let subgraph = cs
        .subcorpus_graph("tutorial", vec!["tutorial/doc1".to_string()])
        .unwrap();
    let node_search = subgraph.get_node_annos().exact_anno_search(
        Some("annis"),
        "node_type",
        Some("node").into(),
    );
    for m in node_search {
        let m = m?;
        // get the numeric node ID from the match
        let id = m.node;
        // get the node name from the ID by searching for the label with the name "annis::node_name"
        let matched_node_name = subgraph
            .get_node_annos()
            .get_annotations_for_item(&id)
            .into_iter()
            .filter(|anno| anno.key.ns == "annis" && anno.key.name == "node_name")
            .map(|anno| anno.val)
            .next()
            .unwrap();
        println!("{}", matched_node_name);
    }

    Ok(())
}
