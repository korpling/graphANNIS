use graphannis_core::graph::{
    update::{GraphUpdate, UpdateEvent},
    ANNIS_NS, DEFAULT_NS,
};

use crate::model::AnnotationComponentType;

/// Create update events for the following corpus structure:
///
/// ```
///            rootCorpus
///           /         \
/// 	subCorpus1    subCorpus2
/// 	/      \      /       \
///   doc1    doc2  doc3     doc4
/// ```
pub(crate) fn create_corpus_structure(update: &mut GraphUpdate) {
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
pub(crate) fn create_corpus_structure_simple(update: &mut GraphUpdate) {
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
pub(crate) fn create_tokens(
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
        create_token_node(
            update,
            &format!("{}tok{}", prefix, i),
            t,
            None,
            None,
            parent_node,
        );
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

/// Creates two segmentation layers that cover the same  timeline.
///
/// ```text
/// a: [Another   ] [ex] [ample] [text]
/// b: [An] [other] [example   ] [text]
/// ```
///
/// The timeline items have the name `tli1`, `tli2`, ..., `tli5`.
pub(crate) fn create_multiple_segmentations(update: &mut GraphUpdate, document_node: &str) {
    let prefix = format!("{}#", document_node);

    // Timeline items
    for i in 1..=5 {
        create_token_node(
            update,
            &format!("{prefix}tli{i}"),
            " ",
            None,
            None,
            Some(document_node),
        )
    }

    // Segmentation `a`
    make_segmentation_span(
        update,
        &format!("{prefix}a1"),
        document_node,
        "a",
        "Another",
        &[&format!("{prefix}tli1"), &format!("{prefix}tli2")],
    );
    make_segmentation_span(
        update,
        &format!("{prefix}a2"),
        document_node,
        "a",
        "ex",
        &[&format!("{prefix}tli3")],
    );
    make_segmentation_span(
        update,
        &format!("{prefix}a3"),
        document_node,
        "a",
        "ample",
        &[&format!("{prefix}tli4")],
    );

    make_segmentation_span(
        update,
        &format!("{prefix}a4"),
        document_node,
        "a",
        "text",
        &[&format!("{prefix}tli5")],
    );

    // Segmentation `b`
    make_segmentation_span(
        update,
        &format!("{prefix}b1"),
        document_node,
        "b",
        "An",
        &[&format!("{prefix}tli1")],
    );
    make_segmentation_span(
        update,
        &format!("{prefix}b2"),
        document_node,
        "b",
        "other",
        &[&format!("{prefix}tli2")],
    );
    make_segmentation_span(
        update,
        &format!("{prefix}b3"),
        document_node,
        "b",
        "example",
        &[&format!("{prefix}tli3"), &format!("{prefix}tli4")],
    );

    make_segmentation_span(
        update,
        &format!("{prefix}b4"),
        document_node,
        "b",
        "text",
        &[&format!("{prefix}tli5")],
    );

    // add the order relations
    for i in 1..5 {
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: format!("{prefix}tli{}", i),
                target_node: format!("{prefix}tli{}", i + 1),
                layer: ANNIS_NS.to_string(),
                component_type: "Ordering".to_string(),
                component_name: "".to_string(),
            })
            .unwrap();
    }
    for i in 1..4 {
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: format!("{prefix}a{}", i),
                target_node: format!("{prefix}a{}", i + 1),
                layer: DEFAULT_NS.to_string(),
                component_type: "Ordering".to_string(),
                component_name: "a".to_string(),
            })
            .unwrap();
        update
            .add_event(UpdateEvent::AddEdge {
                source_node: format!("{prefix}b{}", i),
                target_node: format!("{prefix}b{}", i + 1),
                layer: DEFAULT_NS.to_string(),
                component_type: "Ordering".to_string(),
                component_name: "b".to_string(),
            })
            .unwrap();
    }
}

pub(crate) fn create_token_node(
    update: &mut GraphUpdate,
    node_name: &str,
    token_value: &str,
    whitespace_before: Option<&str>,
    whitespace_after: Option<&str>,
    document_node: Option<&str>,
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

    if let Some(ws) = whitespace_before {
        update
            .add_event(UpdateEvent::AddNodeLabel {
                node_name: node_name.to_string(),
                anno_ns: ANNIS_NS.to_string(),
                anno_name: "tok-whitespace-before".to_string(),
                anno_value: ws.to_string(),
            })
            .unwrap();
    }
    if let Some(ws) = whitespace_after {
        update
            .add_event(UpdateEvent::AddNodeLabel {
                node_name: node_name.to_string(),
                anno_ns: ANNIS_NS.to_string(),
                anno_name: "tok-whitespace-after".to_string(),
                anno_value: ws.to_string(),
            })
            .unwrap();
    }

    if let Some(parent_node) = document_node {
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

pub(crate) fn make_span(
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

pub(crate) fn make_segmentation_span(
    update: &mut GraphUpdate,
    node_name: &str,
    parent_node_name: &str,
    segmentation_name: &str,
    segmentation_value: &str,
    covered_token_names: &[&str],
) {
    update
        .add_event(UpdateEvent::AddNode {
            node_name: node_name.to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();

    update
        .add_event(UpdateEvent::AddNodeLabel {
            node_name: node_name.into(),
            anno_ns: ANNIS_NS.into(),
            anno_name: "tok".into(),
            anno_value: segmentation_value.into(),
        })
        .unwrap();
    update
        .add_event(UpdateEvent::AddNodeLabel {
            node_name: node_name.into(),
            anno_ns: "".into(),
            anno_name: segmentation_name.into(),
            anno_value: segmentation_value.into(),
        })
        .unwrap();

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
    update
        .add_event(UpdateEvent::AddEdge {
            source_node: node_name.into(),
            target_node: parent_node_name.into(),
            layer: ANNIS_NS.into(),
            component_type: AnnotationComponentType::PartOf.to_string(),
            component_name: "".into(),
        })
        .unwrap();
}
