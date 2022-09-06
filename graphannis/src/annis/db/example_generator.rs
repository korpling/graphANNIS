use graphannis_core::graph::{
    update::{GraphUpdate, UpdateEvent},
    ANNIS_NS,
};

/// Create update events for the following corpus structure:
///
/// ```
///            rootCorpus
///           /         \
/// 	subCorpus1    subCorpus2
/// 	/      \      /       \
///   doc1    doc2  doc3     doc4
/// ```
pub fn create_corpus_structure(update: &mut GraphUpdate) {
    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus1".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus1".to_string(),
            target_node: "root".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus2".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus2".to_string(),
            target_node: "root".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus1/doc1".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus1/doc1".to_string(),
            target_node: "root/subCorpus1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus1/doc2".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus1/doc2".to_string(),
            target_node: "root/subCorpus1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus2/doc3".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus2/doc3".to_string(),
            target_node: "root/subCorpus2".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/subCorpus2/doc4".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/subCorpus2/doc4".to_string(),
            target_node: "root/subCorpus2".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();
}

/// Create update events for the following corpus structure:
///
/// ```
///  rootCorpus
///       |
///      docc1
/// ```
pub fn create_corpus_structure_simple(update: &mut GraphUpdate) {
    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/doc1".to_string(),
            node_type: "corpus".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1".to_string(),
            target_node: "root".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNode {
            node_name: "root/doc1#text1".to_string(),
            node_type: "datasource".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddEdge {
            source_node: "root/doc1#text1".to_string(),
            target_node: "root/doc1".to_string(),
            layer: ANNIS_NS.to_string(),
            component_type: "PartOf".to_string(),
            component_name: "".to_string(),
        })
        .unwrap();
}

/// Creates éxample token objects. If a document name is given, the
/// token objects are attached to it.
///
/// The example tokens are
/// - Is
/// - this
/// - example
/// - more
/// - complicated
/// - than
/// - it
/// - appears
/// - to
/// - be
/// - ?
///  
pub fn create_tokens(
    update: &mut GraphUpdate,
    document_name: Option<&str>,
    parent_node: Option<&str>,
) {
    let prefix = if let Some(document_name) = document_name {
        format!("{}#", document_name)
    } else {
        "".to_string()
    };

    let token_strings = vec![
        "Is",
        "this",
        "example",
        "more",
        "complicated",
        "than",
        "it",
        "appears",
        "to",
        "be",
        "?",
    ];
    for (i, t) in token_strings.iter().enumerate() {
        create_token_node(update, &format!("{}tok{}", prefix, i), t, parent_node);
    }

    // add the order relations
    for i in 0..token_strings.len() {
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: format!("{}tok{}", prefix, i),
                target_node: format!("{}tok{}", prefix, i + 1),
                layer: ANNIS_NS.to_string(),
                component_type: "Ordering".to_string(),
                component_name: "".to_string(),
            })
            .unwrap();
    }
}

pub fn create_token_node(
    update: &mut GraphUpdate,
    node_name: &str,
    token_value: &str,
    parent_node: Option<&str>,
) {
    update
        .add_event(UpdateEvent::AddNode {
            node_name: node_name.to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddNodeLabel {
            node_name: node_name.to_string(),
            anno_ns: ANNIS_NS.to_string(),
            anno_name: "tok".to_string(),
            anno_value: token_value.to_string(),
        })
        .unwrap();

    if let Some(parent_node) = parent_node {
        // add the token node to the document
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: node_name.to_string(),
                target_node: parent_node.to_string(),
                layer: ANNIS_NS.to_string(),
                component_type: "PartOf".to_string(),
                component_name: "".to_string(),
            })
            .unwrap();
    }
}

pub fn make_span(
    update: &mut GraphUpdate,
    node_name: &str,
    covered_token_names: &[&str],
    create_source: bool,
) {
    if create_source {
        update
            .add_event(UpdateEvent::AddNode {
                node_name: node_name.to_string(),
                node_type: "node".to_string(),
            })
            .unwrap();
    }
    for c in covered_token_names {
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: node_name.to_string(),
                target_node: c.to_string(),
                layer: "".to_string(),
                component_type: "Coverage".to_string(),
                component_name: "".to_string(),
            })
            .unwrap();
    }
}
