use libc;
use std;
use graphannis::api::corpusstorage::CorpusStorage;
use std::path::PathBuf;


#[repr(C)]
pub struct annis_CorpusStorage(CorpusStorage);

/// Create a new corpus storage
#[no_mangle]
pub extern "C" fn annis_csm_new(db_dir: *const libc::c_char) -> *mut annis_CorpusStorage {
    let db_dir: &str = unsafe {
        assert!(!db_dir.is_null());

        std::ffi::CStr::from_ptr(db_dir).to_str().unwrap()
    };

    let db_dir_path = PathBuf::from(db_dir);

    let s = CorpusStorage::new(&db_dir_path);
    if let Ok(s) = s {
        Box::into_raw(Box::new(annis_CorpusStorage(s)))
    } else {
        std::ptr::null_mut()
    }

}

/// Delete a corpus storage 
#[no_mangle]
pub extern "C" fn annis_csm_free(ptr: *mut annis_CorpusStorage) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_csm_count(ptr: *const annis_CorpusStorage) -> libc::uint64_t {

    let cs : &CorpusStorage = cast_const!(ptr);
    

    return 0;

}