use super::{cast_const, cstr};
use crate::data::IterPtr;
use graphannis::{
    graph::{Annotation, Edge, GraphStorage, Match, NodeID},
    model::{AnnotationComponent, AnnotationComponentType},
    AnnotationGraph,
};
use std::ffi::CString;
use std::sync::Arc;

/// Get the type of the given component.
#[no_mangle]
pub extern "C" fn annis_component_type(c: *const AnnotationComponent) -> AnnotationComponentType {
    let c: &AnnotationComponent = cast_const(c);
    c.get_type()
}

/// Get the layer of the given component.
///
/// The returned string must be deallocated by the caller using annis_str_free()!
#[no_mangle]
pub extern "C" fn annis_component_layer(c: *const AnnotationComponent) -> *mut libc::c_char {
    let c: &AnnotationComponent = cast_const(c);
    let as_string: &str = &c.layer;
    CString::new(as_string).unwrap_or_default().into_raw()
}

/// Get the name of the given component.
///
/// The returned string must be deallocated by the caller using annis_str_free()!
#[no_mangle]
pub extern "C" fn annis_component_name(c: *const AnnotationComponent) -> *mut libc::c_char {
    let c: &AnnotationComponent = cast_const(c);
    let as_string: &str = &c.name;
    CString::new(as_string).unwrap_or_default().into_raw()
}

/// Return an iterator over all nodes of the graph `g` and the given `node_type` (e.g. "node" or "corpus").
#[no_mangle]
pub extern "C" fn annis_graph_nodes_by_type(
    g: *const AnnotationGraph,
    node_type: *const libc::c_char,
) -> *mut IterPtr<NodeID> {
    let db: &AnnotationGraph = cast_const(g);
    let node_type = cstr(node_type);
    let it = db
        .get_node_annos()
        .exact_anno_search(Some("annis"), "node_type", Some(node_type.as_ref()).into())
        .map(|m: Match| m.node);
    Box::into_raw(Box::new(Box::new(it)))
}

/// Return a vector of all annotations for the given `node` in the graph `g`.
#[no_mangle]
pub extern "C" fn annis_graph_annotations_for_node(
    g: *const AnnotationGraph,
    node: NodeID,
) -> *mut Vec<Annotation> {
    let db: &AnnotationGraph = cast_const(g);

    Box::into_raw(Box::new(
        db.get_node_annos().get_annotations_for_item(&node),
    ))
}

/// Return a vector of all components for the graph `g`.
#[no_mangle]
pub extern "C" fn annis_graph_all_components(
    g: *const AnnotationGraph,
) -> *mut Vec<AnnotationComponent> {
    let db: &AnnotationGraph = cast_const(g);

    Box::into_raw(Box::new(db.get_all_components(None, None)))
}

/// Return a vector of all components for the graph `g` and the given component type.
#[no_mangle]
pub extern "C" fn annis_graph_all_components_by_type(
    g: *const AnnotationGraph,
    ctype: AnnotationComponentType,
) -> *mut Vec<AnnotationComponent> {
    let db: &AnnotationGraph = cast_const(g);

    Box::into_raw(Box::new(db.get_all_components(Some(ctype), None)))
}

/// Return a vector of all outgoing edges for the graph `g`, the `source` node and the given `component`.
#[no_mangle]
pub extern "C" fn annis_graph_outgoing_edges(
    g: *const AnnotationGraph,
    source: NodeID,
    component: *const AnnotationComponent,
) -> *mut Vec<Edge> {
    let db: &AnnotationGraph = cast_const(g);
    let component: &AnnotationComponent = cast_const(component);

    let mut result: Vec<Edge> = Vec::new();

    if let Some(gs) = db.get_graphstorage(component) {
        let gs: Arc<dyn GraphStorage> = gs;
        result.extend(
            gs.get_outgoing_edges(source)
                .map(|target| Edge { source, target }),
        );
    }

    Box::into_raw(Box::new(result))
}

/// Return a vector of annnotations for the given `edge` in the `component` of graph `g.
#[no_mangle]
pub extern "C" fn annis_graph_annotations_for_edge(
    g: *const AnnotationGraph,
    edge: Edge,
    component: *const AnnotationComponent,
) -> *mut Vec<Annotation> {
    let db: &AnnotationGraph = cast_const(g);
    let component: &AnnotationComponent = cast_const(component);

    let annos: Vec<Annotation> = if let Some(gs) = db.get_graphstorage(component) {
        gs.get_anno_storage().get_annotations_for_item(&edge)
    } else {
        vec![]
    };

    Box::into_raw(Box::new(annos))
}
