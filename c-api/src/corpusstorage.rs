use libc;
use libc::c_char;
use std;
use std::ffi::CString;
use graphannis::api::corpusstorage as cs;
use graphannis::api::update::GraphUpdate;
use std::path::PathBuf;

/// Create a new corpus storage
#[no_mangle]
pub extern "C" fn annis_cs_new(db_dir: *const libc::c_char) -> *mut cs::CorpusStorage {
    let db_dir = cstr!(db_dir);

    let db_dir_path = PathBuf::from(String::from(db_dir));

    let s = cs::CorpusStorage::new_auto_cache_size(&db_dir_path);
    if let Ok(s) = s {
        return Box::into_raw(Box::new(s));
    }


    return std::ptr::null_mut();
}

/// Delete a corpus storage
#[no_mangle]
pub extern "C" fn annis_cs_free(ptr: *mut cs::CorpusStorage) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_cs_count(
    ptr: *const cs::CorpusStorage,
    corpus: *const libc::c_char,
    query_as_json: *const libc::c_char,
) -> libc::uint64_t {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let query = cstr!(query_as_json);
    let corpus =  cstr!(corpus);

    return cs.count(&corpus, &query).unwrap_or(0) as u64;
}

/// Return an NULL-terminated array of strings that contains the names of all known corpora.
#[no_mangle]
pub extern "C" fn annis_cs_list(
    ptr: *const cs::CorpusStorage,
) -> *mut *mut c_char {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let mut corpora : Vec<* mut c_char> = vec![];

    if let Ok(info) = cs.list() {
        for c in info {
            if let Ok(name) = CString::new(c.name) {
                corpora.push(name.into_raw());
            }
        }  
    }

    // add a null-pointer to the end
    corpora.push(std::ptr::null_mut());

    corpora.shrink_to_fit();
    let corpora_ref = corpora.as_mut_ptr();
    std::mem::forget(corpora);

    return corpora_ref;
}

#[no_mangle]
pub extern "C" fn annis_cs_apply_update(
    ptr: *mut cs::CorpusStorage,
    corpus: *const libc::c_char,
    update: *mut GraphUpdate,
) -> * mut c_char {
    let cs: &mut cs::CorpusStorage = cast_mut!(ptr);
    let update: &mut GraphUpdate = cast_mut!(update);
    let corpus = cstr!(corpus); 
    if let Err(e) = cs.apply_update(&corpus, update) {
        return super::error_string(e);
    }
    

    std::ptr::null_mut()
}

