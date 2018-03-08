use libc;
use std;
use graphannis::api::update::*;

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

#[no_mangle]
pub extern "C" fn annis_graphupdate_add_node(ptr: *mut GraphUpdate, 
    node_name: *const libc::c_char, 
    node_type: *const libc::c_char) {

    if let (Ok(node_name), Ok(node_type)) = (cstr!(node_name).to_str(), cstr!(node_type).to_str()) {
        let cs: &mut GraphUpdate = cast_mut!(ptr);
        cs.add_event(UpdateEvent::AddNode {
            node_name: String::from(node_name), node_type: String::from(node_type)
        });

    }

}