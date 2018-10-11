use super::cerror;
use super::cerror::ErrorList;
use capi::data::IterPtr;
use graph::{Annotation, AnnotationStorage, Component, ComponentType, Edge, NodeID};
use graph::{GraphStorage, Match};
use libc;
use std;
use std::ffi::CString;
use std::sync::Arc;
use update::GraphUpdate;
use Graph;

#[no_mangle]
pub extern "C" fn annis_component_type(c: *const Component) -> ComponentType {
    let c: &Component = cast_const!(c);
    return c.ctype.clone();
}

#[no_mangle]
pub extern "C" fn annis_component_layer(c: *const Component) -> *mut libc::c_char {
    let c: &Component = cast_const!(c);
    let as_string: &str = &c.layer;
    return CString::new(as_string).unwrap_or_default().into_raw();
}

#[no_mangle]
pub extern "C" fn annis_component_name(c: *const Component) -> *mut libc::c_char {
    let c: &Component = cast_const!(c);
    let as_string: &str = &c.name;
    return CString::new(as_string).unwrap_or_default().into_raw();
}

#[no_mangle]
pub extern "C" fn annis_graph_nodes_by_type(
    g: *const Graph,
    node_type: *const libc::c_char,
) -> *mut IterPtr<NodeID> {
    let db: &Graph = cast_const!(g);
    let node_type = cstr!(node_type);

    let type_key = db.get_node_type_key();
    let it = db
        .exact_anno_search(
            Some(type_key.ns),
            type_key.name,
            Some(String::from(node_type)),
        ).map(|m: Match| m.get_node());
    return Box::into_raw(Box::new(Box::new(it)));
}

#[no_mangle]
pub extern "C" fn annis_graph_annotations_for_node(g: *const Graph, node: NodeID) -> *mut Vec<Annotation> {
    let db: &Graph = cast_const!(g);

    Box::into_raw(Box::new(db.get_annotations_for_item(&node)))
}

#[no_mangle]
pub extern "C" fn annis_graph_all_components(g: *const Graph) -> *mut Vec<Component> {
    let db: &Graph = cast_const!(g);

    Box::into_raw(Box::new(db.get_all_components(None, None)))
}

#[no_mangle]
pub extern "C" fn annis_graph_all_components_by_type(
    g: *const Graph,
    ctype: ComponentType,
) -> *mut Vec<Component> {
    let db: &Graph = cast_const!(g);

    Box::into_raw(Box::new(db.get_all_components(Some(ctype), None)))
}

#[no_mangle]
pub extern "C" fn annis_graph_outgoing_edges(
    g: *const Graph,
    source: NodeID,
    component: *const Component,
) -> *mut Vec<Edge> {
    let db: &Graph = cast_const!(g);
    let component: &Component = cast_const!(component);

    let mut result: Vec<Edge> = Vec::new();

    if let Some(gs) = db.get_graphstorage(component) {
        let gs: Arc<GraphStorage> = gs;
        result.extend(gs.get_outgoing_edges(&source).map(|target| Edge {
            source: source.clone(),
            target,
        }));
    }

    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn annis_graph_annotations_for_edge(
    g: *const Graph,
    edge: Edge,
    component: *const Component,
) -> *mut Vec<Annotation> {
    let db: &Graph = cast_const!(g);
    let component: &Component = cast_const!(component);

    let annos: Vec<Annotation> = if let Some(gs) = db.get_graphstorage(component) {
        gs.get_anno_storage().get_annotations_for_item(&edge)
    } else {
        vec![]
    };

    Box::into_raw(Box::new(annos))
}

#[no_mangle]
pub extern "C" fn annis_graph_apply_update(
    g: *mut Graph,
    update: *mut GraphUpdate,
    err: *mut *mut ErrorList,
) {
    let db: &mut Graph = cast_mut!(g);
    let update: &mut GraphUpdate = cast_mut!(update);
    try_cerr!(db.apply_update(update), err, ());
}
