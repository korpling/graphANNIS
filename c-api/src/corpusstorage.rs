use libc;
use std;
use graphannis::api::corpusstorage::CorpusStorage;
use std::path::PathBuf;

#[repr(C)]
pub struct annis_CorpusStorage(CorpusStorage);

/// Create a new corpus storage
#[no_mangle]
pub extern "C" fn annis_cs_new(db_dir: *const libc::c_char) -> *mut annis_CorpusStorage {
    let db_dir = cstr!(db_dir);

    if let Ok(db_dir) = db_dir.to_str() {
        let db_dir_path = PathBuf::from(db_dir);

        let s = CorpusStorage::new(&db_dir_path);
        if let Ok(s) = s {
            return Box::into_raw(Box::new(annis_CorpusStorage(s)));
        }
    }

    return std::ptr::null_mut();
}

/// Delete a corpus storage
#[no_mangle]
pub extern "C" fn annis_cs_free(ptr: *mut annis_CorpusStorage) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_cs_count(
    ptr: *const annis_CorpusStorage,
    corpus: *const libc::c_char,
    query_as_json: *const libc::c_char,
) -> libc::uint64_t {
    let cs: &CorpusStorage = cast_const!(ptr);

    if let (Ok(query), Ok(corpus)) = (cstr!(query_as_json).to_str(), cstr!(corpus).to_str()) {

        return cs.count(corpus, query).unwrap_or(0) as u64;

    }

    return 0;
}
