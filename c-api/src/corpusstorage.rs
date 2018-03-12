use libc;
use std;
use std::ffi::CString;
use graphannis::api::corpusstorage as cs;
use graphannis::api::update::GraphUpdate;
use graphannis::api::graph::{Node};
use std::path::PathBuf;
use super::error::Error;

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
    let corpus = cstr!(corpus);

    return cs.count(&corpus, &query).unwrap_or(0) as u64;
}

#[no_mangle]
pub extern "C" fn annis_cs_find(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    query_as_json: *const libc::c_char,
    offset: libc::size_t,
    limit: libc::size_t,
) -> * mut Vec<CString> {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let query = cstr!(query_as_json);
    let corpus = cstr!(corpus_name);

    let result = cs.find(&corpus, &query, offset, limit);

    let vec_result : Vec<CString> = if let Ok(result) = result {
        result.into_iter().map(|x| CString::new(x).unwrap_or_default()).collect()
    } else {
        vec![]
    };

    return Box::into_raw(Box::new(vec_result));
}

#[no_mangle]
pub extern "C" fn annis_cs_subgraph(ptr: *const cs::CorpusStorage, 
        corpus_name: * const libc::c_char,
        node_ids: * const Vec<CString>,
        ctx_left: libc::size_t,
        ctx_right: libc::size_t) -> * mut Vec<Node> {

    let cs : &cs::CorpusStorage = cast_const!(ptr);
    let node_ids : Vec<String> = cast_const!(node_ids).iter().map(|id| String::from(id.to_string_lossy())).collect();
    let corpus = cstr!(corpus_name);

    if let Ok(result) = cs.subgraph(&corpus, node_ids, ctx_left, ctx_right) {
        return Box::into_raw(Box::new(result));
    }
    return std::ptr::null_mut();
}

/// List all known corpora.
#[no_mangle]
pub extern "C" fn annis_cs_list(ptr: *const cs::CorpusStorage) -> *mut Vec<CString> {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let mut corpora: Vec<CString> = vec![];

    if let Ok(info) = cs.list() {
        for c in info {
            if let Ok(name) = CString::new(c.name) {
                corpora.push(name);
            }
        }
    }

    return Box::into_raw(Box::new(corpora));
}

#[no_mangle]
pub extern "C" fn annis_cs_apply_update(
    ptr: *mut cs::CorpusStorage,
    corpus: *const libc::c_char,
    update: *mut GraphUpdate,
) -> *mut Error {
    let cs: &mut cs::CorpusStorage = cast_mut!(ptr);
    let update: &mut GraphUpdate = cast_mut!(update);
    let corpus = cstr!(corpus);
    if let Err(e) = cs.apply_update(&corpus, update) {
        return super::error::new(e);
    }

    std::ptr::null_mut()
}
