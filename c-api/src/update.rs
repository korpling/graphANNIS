use libc;
use libc::{c_char};
use std;
use graphannis::api::update::*;
use graphannis;

/// Create a new graph update instance
#[no_mangle]
pub extern "C" fn annis_graphupdate_new() -> *mut GraphUpdate {

    let gu = GraphUpdate::new();
    return Box::into_raw(Box::new(gu));
}

/// Delete a graph update instance
#[no_mangle]
pub extern "C" fn annis_graphupdate_free(ptr: *mut GraphUpdate) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[repr(C)]
pub enum UpdateEvent {
    AddNode {
        node_name: * const c_char,
        node_type: * const c_char,
    },
    DeleteNode {
        node_name: * const c_char,
    },
    AddNodeLabel {
        node_name: * const c_char,
        anno_ns: * const c_char,
        anno_name: * const c_char,
        anno_value: * const c_char,
    },
    DeleteNodeLabel {
        node_name: * const c_char,
        anno_ns: * const c_char,
        anno_name: * const c_char,
    },
    AddEdge {
        source_node: * const c_char,
        target_node: * const c_char,
        layer: * const c_char,
        component_type: * const c_char,
        component_name: * const c_char,
    },
    DeleteEdge {
        source_node: * const c_char,
        target_node: * const c_char,
        layer: * const c_char,
        component_type: * const c_char,
        component_name: * const c_char,
    },
    AddEdgeLabel {
        source_node: * const c_char,
        target_node: * const c_char,
        layer: * const c_char,
        component_type: * const c_char,
        component_name: * const c_char,
        anno_ns: * const c_char,
        anno_name: * const c_char,
        anno_value: * const c_char,
    },
    DeleteEdgeLabel {
        source_node: * const c_char,
        target_node: * const c_char,
        layer: * const c_char,
        component_type: * const c_char,
        component_name: * const c_char,
        anno_ns: * const c_char,
        anno_name: * const c_char,
    },
}

#[no_mangle]
pub extern "C" fn annis_graphupdate_add_event(ptr: *mut GraphUpdate, event : UpdateEvent) {

}

#[no_mangle]
pub extern "C" fn annis_graphupdate_add_node(ptr: *mut GraphUpdate, 
    node_name: *const libc::c_char, 
    node_type: *const libc::c_char) {

    if let (Ok(node_name), Ok(node_type)) = (cstr!(node_name).to_str(), cstr!(node_type).to_str()) {
        let cs: &mut GraphUpdate = cast_mut!(ptr);
        cs.add_event(graphannis::api::update::UpdateEvent::AddNode {
            node_name: String::from(node_name), node_type: String::from(node_type)
        });

    }

}