use super::cerror;
use super::cerror::ErrorList;
use super::Matrix;
use corpusstorage::{CountExtra, FrequencyTable, QueryAttributeDescription};
use corpusstorage::{FrequencyDefEntry, QueryLanguage, ResultOrder};
use graph::{AnnotationStorage, Component, ComponentType};
use libc;
use relannis;
use std;
use std::ffi::CString;
use std::path::PathBuf;
use update::GraphUpdate;
use {CorpusStorage, Graph};

/// Create a new corpus storage with an automatically determined maximum cache size.
#[no_mangle]
pub extern "C" fn annis_cs_with_auto_cache_size(
    db_dir: *const libc::c_char,
    use_parallel: bool,
) -> *mut CorpusStorage {
    let db_dir = cstr!(db_dir);

    let db_dir_path = PathBuf::from(String::from(db_dir));

    let s = CorpusStorage::with_auto_cache_size(&db_dir_path, use_parallel);

    match s {
        Ok(result) => {
            return Box::into_raw(Box::new(result));
        }
        Err(err) => error!("Could create corpus storage, error message was:\n{:?}", err),
    };
    return std::ptr::null_mut();
}

/// Create a new corpus storage with an manually defined maximum cache size.
#[no_mangle]
pub extern "C" fn annis_cs_with_max_cache_size(
    db_dir: *const libc::c_char,
    max_cache_size: usize,
    use_parallel: bool,
) -> *mut CorpusStorage {
    let db_dir = cstr!(db_dir);

    let db_dir_path = PathBuf::from(String::from(db_dir));

    let s = CorpusStorage::with_max_cache_size(&db_dir_path, Some(max_cache_size), use_parallel);

    match s {
        Ok(result) => {
            return Box::into_raw(Box::new(result));
        }
        Err(err) => error!("Could create corpus storage, error message was:\n{:?}", err),
    };
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_cs_free(ptr: *mut CorpusStorage) {
    if ptr.is_null() {
        return;
    }
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_cs_count(
    ptr: *const CorpusStorage,
    corpus: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> libc::uint64_t {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);
    let corpus = cstr!(corpus);

    return try_cerr!(cs.count(&corpus, &query, query_language), err, 0);
}

#[no_mangle]
pub extern "C" fn annis_cs_count_extra(
    ptr: *const CorpusStorage,
    corpus: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> CountExtra {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);
    let corpus = cstr!(corpus);

    return try_cerr!(
        cs.count_extra(&corpus, &query, query_language),
        err,
        CountExtra::default()
    );
}

#[no_mangle]
pub extern "C" fn annis_cs_find(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    offset: libc::size_t,
    limit: libc::size_t,
    order: ResultOrder,
    err: *mut *mut ErrorList,
) -> *mut Vec<CString> {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);
    let corpus = cstr!(corpus_name);

    let result = try_cerr!(
        cs.find(&corpus, &query, query_language, offset, limit, order),
        err,
        std::ptr::null_mut()
    );

    let vec_result: Vec<CString> = result
        .into_iter()
        .map(|x| CString::new(x).unwrap_or_default())
        .collect();

    return Box::into_raw(Box::new(vec_result));
}

#[no_mangle]
pub extern "C" fn annis_cs_subgraph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    node_ids: *const Vec<CString>,
    ctx_left: libc::size_t,
    ctx_right: libc::size_t,
    err: *mut *mut ErrorList,
) -> *mut Graph {
    let cs: &CorpusStorage = cast_const!(ptr);
    let node_ids: Vec<String> = cast_const!(node_ids)
        .iter()
        .map(|id| String::from(id.to_string_lossy()))
        .collect();
    let corpus = cstr!(corpus_name);

    let result = try_cerr!(
        cs.subgraph(&corpus, node_ids, ctx_left, ctx_right),
        err,
        std::ptr::null_mut()
    );
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_subcorpus_graph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    corpus_ids: *const Vec<CString>,
    err: *mut *mut ErrorList,
) -> *mut Graph {
    let cs: &CorpusStorage = cast_const!(ptr);
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

    let result = try_cerr!(
        cs.subcorpus_graph(&corpus, corpus_ids),
        err,
        std::ptr::null_mut()
    );
    trace!(
        "annis_cs_subcorpus_graph(...) returns subgraph with {} labels",
        result.number_of_annotations()
    );
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_corpus_graph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> *mut Graph {
    let cs: &CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);

    let result = try_cerr!(cs.corpus_graph(&corpus), err, std::ptr::null_mut());
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_subgraph_for_query(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> *mut Graph {
    let cs: &CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);
    let query = cstr!(query);

    let result = try_cerr!(
        cs.subgraph_for_query(&corpus, &query, query_language),
        err,
        std::ptr::null_mut()
    );
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_frequency(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    frequency_query_definition: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> *mut FrequencyTable<CString> {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);
    let corpus = cstr!(corpus_name);
    let frequency_query_definition = cstr!(frequency_query_definition);
    let table_def: Vec<FrequencyDefEntry> = frequency_query_definition
        .split(',')
        .filter_map(|d| -> Option<FrequencyDefEntry> { d.parse().ok() })
        .collect();

    let orig_ft = try_cerr!(
        cs.frequency(&corpus, &query, query_language, table_def),
        err,
        std::ptr::null_mut()
    );

    let mut result: FrequencyTable<CString> = FrequencyTable::new();

    for (tuple, count) in orig_ft.into_iter() {
        let mut new_tuple: Vec<CString> = Vec::with_capacity(tuple.len());
        for att in tuple.into_iter() {
            if let Ok(att) = CString::new(att) {
                new_tuple.push(att);
            } else {
                new_tuple.push(CString::default())
            }
        }

        result.push((new_tuple, count));
    }
    return Box::into_raw(Box::new(result));
}

/// List all known corpora.
#[no_mangle]
pub extern "C" fn annis_cs_list(
    ptr: *const CorpusStorage,
    err: *mut *mut ErrorList,
) -> *mut Vec<CString> {
    let cs: &CorpusStorage = cast_const!(ptr);

    let mut corpora: Vec<CString> = vec![];

    let info = try_cerr!(cs.list(), err, std::ptr::null_mut());

    for c in info {
        if let Ok(name) = CString::new(c.name) {
            corpora.push(name);
        }
    }
    return Box::into_raw(Box::new(corpora));
}

#[no_mangle]
pub extern "C" fn annis_cs_list_node_annotations(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    list_values: bool,
    only_most_frequent_values: bool,
) -> *mut Matrix<CString> {
    let cs: &CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);

    let orig_vec = cs.list_node_annotations(&corpus, list_values, only_most_frequent_values);
    let mut result: Matrix<CString> = Matrix::new();
    for anno in orig_vec.into_iter() {
        if let (Ok(ns), Ok(name), Ok(val)) = (
            CString::new(anno.key.ns),
            CString::new(anno.key.name),
            CString::new(anno.val),
        ) {
            result.push(vec![ns, name, val]);
        }
    }
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_list_edge_annotations(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    component_type: ComponentType,
    component_name: *const libc::c_char,
    component_layer: *const libc::c_char,
    list_values: bool,
    only_most_frequent_values: bool,
) -> *mut Matrix<CString> {
    let cs: &CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);
    let component = Component {
        ctype: component_type,
        name: String::from(cstr!(component_name)),
        layer: String::from(cstr!(component_layer)),
    };

    let orig_vec =
        cs.list_edge_annotations(&corpus, component, list_values, only_most_frequent_values);
    let mut result: Matrix<CString> = Matrix::new();
    for anno in orig_vec.into_iter() {
        if let (Ok(ns), Ok(name), Ok(val)) = (
            CString::new(anno.key.ns),
            CString::new(anno.key.name),
            CString::new(anno.val),
        ) {
            result.push(vec![ns, name, val]);
        }
    }
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_validate_query(
    ptr: *const CorpusStorage,
    corpus: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> bool {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);
    let corpus = cstr!(corpus);

    return try_cerr!(
        cs.validate_query(&corpus, &query, query_language),
        err,
        false
    );
}

#[no_mangle]
pub extern "C" fn annis_cs_node_descriptions(
    ptr: *const CorpusStorage,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> *mut Vec<QueryAttributeDescription> {
    let cs: &CorpusStorage = cast_const!(ptr);

    let query = cstr!(query);

    let result = try_cerr!(
        cs.node_descriptions(&query, query_language),
        err,
        std::ptr::null_mut()
    );
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_cs_import_relannis(
    ptr: *mut CorpusStorage,
    corpus: *const libc::c_char,
    path: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut CorpusStorage = cast_mut!(ptr);

    let override_corpus_name: Option<String> = if corpus.is_null() {
        None
    } else {
        Some(String::from(cstr!(corpus)))
    };
    let path: &str = &cstr!(path);

    let (corpus, db) = try_cerr!(relannis::load(&PathBuf::from(path)), err, ());
    let corpus: String = if let Some(o) = override_corpus_name {
        o
    } else {
        corpus
    };
    cs.import(&corpus, db);
}

#[no_mangle]
pub extern "C" fn annis_cs_list_components_by_type(
    ptr: *mut CorpusStorage,
    corpus_name: *const libc::c_char,
    ctype: ComponentType,
) -> *mut Vec<Component> {
    let cs: &CorpusStorage = cast_const!(ptr);
    let corpus = cstr!(corpus_name);

    Box::into_raw(Box::new(cs.list_components(&corpus, Some(ctype), None)))
}

/// Deletes a corpus from the corpus storage.
#[no_mangle]
pub extern "C" fn annis_cs_delete(
    ptr: *mut CorpusStorage,
    corpus: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> bool {
    let cs: &mut CorpusStorage = cast_mut!(ptr);
    let corpus = cstr!(corpus);

    try_cerr!(cs.delete(&corpus), err, false)
}

#[no_mangle]
pub extern "C" fn annis_cs_apply_update(
    ptr: *mut CorpusStorage,
    corpus: *const libc::c_char,
    update: *mut GraphUpdate,
    err: *mut *mut ErrorList,
) {
    let cs: &mut CorpusStorage = cast_mut!(ptr);
    let update: &mut GraphUpdate = cast_mut!(update);
    let corpus = cstr!(corpus);
    try_cerr!(cs.apply_update(&corpus, update), err, ());
}
