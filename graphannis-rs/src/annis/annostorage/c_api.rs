use annis::NodeID;
use super::*;


#[repr(C)]
pub struct annis_ASNodePtr(AnnoStorage<NodeID>);
#[repr(C)]
pub struct annis_ASEdgePtr(AnnoStorage<Edge>);

/*
AnnoStorage<Node>
*/

#[no_mangle]
pub extern "C" fn annis_asnode_new() -> *mut annis_ASNodePtr {
    let s = AnnoStorage::<NodeID>::new();
    Box::into_raw(Box::new(annis_ASNodePtr(s)))
}

#[no_mangle]
pub extern "C" fn annis_asnode_free(ptr: *mut annis_ASNodePtr) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_asnode_insert(ptr: *mut annis_ASNodePtr, 
    item : NodeID, anno : Annotation) {

     let delegate = unsafe {
        assert!(!ptr.is_null());
        &mut (*ptr).0
    }; 

    delegate.insert(item, anno);
}

/*
AnnoStorage<Edge>
*/

#[no_mangle]
pub extern "C" fn annis_asedge_new() -> *mut annis_ASEdgePtr {
    let s = AnnoStorage::<Edge>::new();
    Box::into_raw(Box::new(annis_ASEdgePtr(s)))
}

#[no_mangle]
pub extern "C" fn annis_asedge_free(ptr: *mut annis_ASEdgePtr) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_asedge_insert(ptr: *mut annis_ASEdgePtr, 
    item : Edge, anno : Annotation) {

     let delegate = unsafe {
        assert!(!ptr.is_null());
        &mut (*ptr).0
    }; 

    delegate.insert(item, anno);
}
