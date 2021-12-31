use super::cerror::ErrorList;
use super::{cast_mut, cstr, map_cerr};
use graphannis::update::{GraphUpdate, UpdateEvent};

/// Create a new graph (empty) update instance
#[no_mangle]
pub extern "C" fn annis_graphupdate_new() -> *mut GraphUpdate {
    let gu = GraphUpdate::new();
    Box::into_raw(Box::new(gu))
}

/// Add "add node" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `node_name` - Name of the new node.
/// - `node_type` - Type of the new node, e.g. "node" or "corpus".
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_add_node(
    ptr: *mut GraphUpdate,
    node_name: *const libc::c_char,
    node_type: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let u: &mut GraphUpdate = cast_mut(ptr);
    map_cerr(
        u.add_event(UpdateEvent::AddNode {
            node_name: String::from(cstr(node_name)),
            node_type: String::from(cstr(node_type)),
        }),
        err,
    );
}

/// Add "delete node" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `node_name` - Name of node to delete.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_delete_node(
    ptr: *mut GraphUpdate,
    node_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);
    map_cerr(
        cs.add_event(UpdateEvent::DeleteNode {
            node_name: String::from(cstr(node_name)),
        }),
        err,
    );
}

/// Add "add node label" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `node_name` - Name of the node the label is attached to.
/// - `annos_ns` - Namespace of the new annotation.
/// - `annos_name` - Name of the new annotation.
/// - `annos_value` - Value of the new annotation.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_add_node_label(
    ptr: *mut GraphUpdate,
    node_name: *const libc::c_char,
    anno_ns: *const libc::c_char,
    anno_name: *const libc::c_char,
    anno_value: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);
    map_cerr(
        cs.add_event(UpdateEvent::AddNodeLabel {
            node_name: String::from(cstr(node_name)),
            anno_ns: String::from(cstr(anno_ns)),
            anno_name: String::from(cstr(anno_name)),
            anno_value: String::from(cstr(anno_value)),
        }),
        err,
    );
}

/// Add "delete node label" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `node_name` - Name of the node the label is attached to.
/// - `annos_ns` - Namespace of deleted new annotation.
/// - `annos_name` - Name of the deleted annotation.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_delete_node_label(
    ptr: *mut GraphUpdate,
    node_name: *const libc::c_char,
    anno_ns: *const libc::c_char,
    anno_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);
    map_cerr(
        cs.add_event(UpdateEvent::DeleteNodeLabel {
            node_name: String::from(cstr(node_name)),
            anno_ns: String::from(cstr(anno_ns)),
            anno_name: String::from(cstr(anno_name)),
        }),
        err,
    );
}

/// Add "add edge" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `source_node` - Name of source node of the new edge.
/// - `target_node` - Name of target node of the new edge.
/// - `layer` - Layer of the new edge.
/// - `component_type` - Type of the component of the new edge.
/// - `component_name` - Name of the component of the new edge.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_add_edge(
    ptr: *mut GraphUpdate,
    source_node: *const libc::c_char,
    target_node: *const libc::c_char,
    layer: *const libc::c_char,
    component_type: *const libc::c_char,
    component_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);

    map_cerr(
        cs.add_event(UpdateEvent::AddEdge {
            source_node: String::from(cstr(source_node)),
            target_node: String::from(cstr(target_node)),
            layer: String::from(cstr(layer)),
            component_type: String::from(cstr(component_type)),
            component_name: String::from(cstr(component_name)),
        }),
        err,
    );
}

/// Add "delete edge" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `source_node` - Name of source node of the edge to delete.
/// - `target_node` - Name of target node of the edge to delete.
/// - `layer` - Layer of the edge to delete.
/// - `component_type` - Type of the component of the edge to delete.
/// - `component_name` - Name of the component of the edge to delete.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_delete_edge(
    ptr: *mut GraphUpdate,
    source_node: *const libc::c_char,
    target_node: *const libc::c_char,
    layer: *const libc::c_char,
    component_type: *const libc::c_char,
    component_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);
    map_cerr(
        cs.add_event(UpdateEvent::DeleteEdge {
            source_node: String::from(cstr(source_node)),
            target_node: String::from(cstr(target_node)),
            layer: String::from(cstr(layer)),
            component_type: String::from(cstr(component_type)),
            component_name: String::from(cstr(component_name)),
        }),
        err,
    );
}

/// Add "add edge label" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `source_node` - Name of source node of the edge.
/// - `target_node` - Name of target node of the edge.
/// - `layer` - Layer of the edge.
/// - `component_type` - Type of the component of the edge.
/// - `component_name` - Name of the component of the edge.
/// - `annos_ns` - Namespace of the new annotation.
/// - `annos_name` - Name of the new annotation.
/// - `annos_value` - Value of the new annotation.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_add_edge_label(
    ptr: *mut GraphUpdate,
    source_node: *const libc::c_char,
    target_node: *const libc::c_char,
    layer: *const libc::c_char,
    component_type: *const libc::c_char,
    component_name: *const libc::c_char,
    anno_ns: *const libc::c_char,
    anno_name: *const libc::c_char,
    anno_value: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);

    map_cerr(
        cs.add_event(UpdateEvent::AddEdgeLabel {
            source_node: String::from(cstr(source_node)),
            target_node: String::from(cstr(target_node)),
            layer: String::from(cstr(layer)),
            component_type: String::from(cstr(component_type)),
            component_name: String::from(cstr(component_name)),
            anno_ns: String::from(cstr(anno_ns)),
            anno_name: String::from(cstr(anno_name)),
            anno_value: String::from(cstr(anno_value)),
        }),
        err,
    );
}

/// Add "delete edge label" action to the graph update object.
///
/// - `ptr` - The graph update object.
/// - `source_node` - Name of source node of the edge.
/// - `target_node` - Name of target node of the edge.
/// - `layer` - Layer of the edge.
/// - `component_type` - Type of the component of the edge.
/// - `component_name` - Name of the component of the edge.
/// - `annos_ns` - Namespace of the annotation to delete.
/// - `annos_name` - Name of the annotation to delete.
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_graphupdate_delete_edge_label(
    ptr: *mut GraphUpdate,
    source_node: *const libc::c_char,
    target_node: *const libc::c_char,
    layer: *const libc::c_char,
    component_type: *const libc::c_char,
    component_name: *const libc::c_char,
    anno_ns: *const libc::c_char,
    anno_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut GraphUpdate = cast_mut(ptr);

    map_cerr(
        cs.add_event(UpdateEvent::DeleteEdgeLabel {
            source_node: String::from(cstr(source_node)),
            target_node: String::from(cstr(target_node)),
            layer: String::from(cstr(layer)),
            component_type: String::from(cstr(component_type)),
            component_name: String::from(cstr(component_name)),
            anno_ns: String::from(cstr(anno_ns)),
            anno_name: String::from(cstr(anno_name)),
        }),
        err,
    );
}
