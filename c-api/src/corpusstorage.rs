use libc;
use std;
use std::ffi::CString;
use graphannis::api::corpusstorage as cs;
use graphannis::api::update::GraphUpdate;
use graphannis::graphdb::GraphDB;
use graphannis::relannis;
use graphannis::{ComponentType, Component, CountExtra};
use std::path::PathBuf;
use super::error::Error;
use super::Matrix;

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

#[no_mangle]
pub extern "C" fn annis_cs_count(
    ptr: *const cs::CorpusStorage,
    corpus: *const libc::c_char,
    query_as_json: *const libc::c_char,
) -> libc::uint64_t {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let query = cstr!(query_as_json);
    let corpus = cstr!(corpus);

    return cs.count(&corpus, &query).unwrap_or(0);
}

#[no_mangle]
pub extern "C" fn annis_cs_count_extra(
    ptr: *const cs::CorpusStorage,
    corpus: *const libc::c_char,
    query_as_json: *const libc::c_char,
) -> CountExtra {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let query = cstr!(query_as_json);
    let corpus = cstr!(corpus);

    return cs.count_extra(&corpus, &query)
        .unwrap_or(CountExtra::default());
}

#[no_mangle]
pub extern "C" fn annis_cs_find(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    query_as_json: *const libc::c_char,
    offset: libc::size_t,
    limit: libc::size_t,
) -> *mut Vec<CString> {
    let cs: &cs::CorpusStorage = cast_const!(ptr);

    let query = cstr!(query_as_json);
    let corpus = cstr!(corpus_name);

    let result = cs.find(&corpus, &query, offset, limit);

    let vec_result: Vec<CString> = if let Ok(result) = result {
        result
            .into_iter()
            .map(|x| CString::new(x).unwrap_or_default())
            .collect()
    } else {
        vec![]
    };

    return Box::into_raw(Box::new(vec_result));
}

#[no_mangle]
pub extern "C" fn annis_cs_subgraph(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    node_ids: *const Vec<CString>,
    ctx_left: libc::size_t,
    ctx_right: libc::size_t,
) -> *mut GraphDB {
    let cs: &cs::CorpusStorage = cast_const!(ptr);
    let node_ids: Vec<String> = cast_const!(node_ids)
        .iter()
        .map(|id| String::from(id.to_string_lossy()))
        .collect();
    let corpus = cstr!(corpus_name);

    if let Ok(result) = cs.subgraph(&corpus, node_ids, ctx_left, ctx_right) {
        return Box::into_raw(Box::new(result));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_cs_subcorpus_graph(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    corpus_ids: *const Vec<CString>,
) -> *mut GraphDB {
    let cs: &cs::CorpusStorage = cast_const!(ptr);
    let corpus_ids: Vec<String> = cast_const!(corpus_ids)
        .iter()
        .map(|id| String::from(id.to_string_lossy()))
        .collect();
    let corpus = cstr!(corpus_name);

    trace!(
        "annis_cs_subcorpus_graph(..., {}, {:?}) called",
        corpus,
        corpus_ids
    );

    let res = cs.subcorpus_graph(&corpus, corpus_ids);
    match res {
        Ok(result) => {
            trace!(
                "annis_cs_subcorpus_graph(...) returns subgraph with {} labels",
                result.node_annos.len()
            );
            return Box::into_raw(Box::new(result));
        }
        Err(err) => warn!("Could not get subgraph, error message was:\n{:?}", err),
    };
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_cs_corpus_graph(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
) -> *mut GraphDB {
    let cs: &cs::CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);

    let res = cs.corpus_graph(&corpus);
    match res {
        Ok(result) => {
            return Box::into_raw(Box::new(result));
        }
        Err(err) => warn!("Could not get corpus graph, error message was:\n{:?}", err),
    };
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_cs_subgraph_for_query(
    ptr: *const cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    query_as_json: *const libc::c_char,
) -> *mut GraphDB {
    let cs: &cs::CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);
    let query_as_json = cstr!(query_as_json);

    let res = cs.subgraph_for_query(&corpus, &query_as_json);
    match res {
        Ok(result) => {
            return Box::into_raw(Box::new(result));
        }
        Err(err) => warn!(
            "Could not get subcorpus graph for query, error message was:\n{:?}",
            err
        ),
    };
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
pub extern "C" fn annis_cs_list_node_annotations(
        ptr: *const cs::CorpusStorage,
        corpus_name: *const libc::c_char,
        list_values: bool,
        only_most_frequent_values: bool,
    ) -> *mut Matrix<CString> {

        let cs: &cs::CorpusStorage = cast_const!(ptr);
        let corpus = cstr!(corpus_name);

        let orig_vec = cs.list_node_annotations(&corpus, list_values, only_most_frequent_values);
        let mut result : Matrix<CString> = Matrix::new();
        for (ns, name, val) in orig_vec.into_iter() {
            if let (Ok(ns), Ok(name), Ok(val)) = (CString::new(ns), CString::new(name), CString::new(val)) {
                result.push(vec![ns, name, val]);
            }
        }
        return Box::into_raw(Box::new(result));
    }

#[no_mangle]
pub extern "C" fn annis_cs_import_relannis(
    ptr: *mut cs::CorpusStorage,
    corpus: *const libc::c_char,
    path: *const libc::c_char,
) -> *mut Error {
    let cs: &mut cs::CorpusStorage = cast_mut!(ptr);

    let override_corpus_name: Option<String> = if corpus.is_null() {
        None
    } else {
        Some(String::from(cstr!(corpus)))
    };
    let path: &str = &cstr!(path);

    let res = relannis::load(&PathBuf::from(path));

    match res {
        Ok((corpus, db)) => {
            let corpus: String = if let Some(o) = override_corpus_name {
                o
            } else {
                corpus
            };
            cs.import(&corpus, db); 
        }
        Err(err) => {
            return Box::into_raw(Box::new(Error::from(err)));
        }
    };

    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn annis_cs_all_components_by_type(
    ptr: *mut cs::CorpusStorage,
    corpus_name: *const libc::c_char,
    ctype: ComponentType,
) -> *mut Vec<Component> {

    let cs: &cs::CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);

    Box::into_raw(Box::new(cs.get_all_components(&corpus, Some(ctype), None)))
}


#[no_mangle]
pub extern "C" fn annis_cs_delete(ptr: *mut cs::CorpusStorage, corpus: *const libc::c_char) {
    let cs: &mut cs::CorpusStorage = cast_mut!(ptr);
    let corpus = cstr!(corpus);

    cs.delete(&corpus);
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
