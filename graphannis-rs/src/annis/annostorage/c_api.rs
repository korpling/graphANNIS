use libc;
use annis::NodeID;
use annis::util::c_api::*;
use super::*;

#[repr(C)]
pub struct annis_NodeAnnoStoragePtr(AnnoStorage<NodeID>);

#[no_mangle]
pub extern "C" fn annis_nodeannostorage_new() -> *mut annis_NodeAnnoStoragePtr {
    let s = AnnoStorage::<NodeID>::new();
    Box::into_raw(Box::new(annis_NodeAnnoStoragePtr(s)))
}

#[no_mangle]
pub extern "C" fn annis_nodeannostorage_free(ptr: *mut annis_NodeAnnoStoragePtr) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}