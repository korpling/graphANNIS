use super::cerror::ErrorList;
use super::Matrix;
use super::{cast_const, cast_mut, cstr, map_cerr};
use graphannis::corpusstorage::ExportFormat;
use graphannis::{
    corpusstorage::{
        CacheStrategy, CountExtra, FrequencyDefEntry, FrequencyTable, FrequencyTableRow,
        ImportFormat, QueryAttributeDescription, QueryLanguage, ResultOrder, SearchQuery,
    },
    model::{AnnotationComponent, AnnotationComponentType},
    update::GraphUpdate,
    AnnotationGraph, CorpusStorage,
};
use std::ffi::CString;
use std::path::PathBuf;

/// Create a new instance with a an automatic determined size of the internal corpus cache.
///
/// Currently, set the maximum cache size to 25% of the available/free memory at construction time.
/// This behavior can change in the future.
///
/// - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
/// - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_with_auto_cache_size(
    db_dir: *const libc::c_char,
    use_parallel_joins: bool,
    err: *mut *mut ErrorList,
) -> *mut CorpusStorage {
    let db_dir = cstr(db_dir);

    let db_dir_path = PathBuf::from(String::from(db_dir));

    let s = CorpusStorage::with_auto_cache_size(&db_dir_path, use_parallel_joins);

    map_cerr(s, err)
        .map(|cs| Box::into_raw(Box::new(cs)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Create a new corpus storage with an manually defined maximum cache size.
///
/// - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
/// - `max_cache_size` - Fixed maximum size of the cache in bytes.
/// - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_with_max_cache_size(
    db_dir: *const libc::c_char,
    max_cache_size: usize,
    use_parallel_joins: bool,
    err: *mut *mut ErrorList,
) -> *mut CorpusStorage {
    let db_dir = cstr(db_dir);

    let db_dir_path = PathBuf::from(String::from(db_dir));

    let s = CorpusStorage::with_cache_strategy(
        &db_dir_path,
        CacheStrategy::FixedMaxMemory(max_cache_size),
        use_parallel_joins,
    );

    map_cerr(s, err)
        .map(|cs| Box::into_raw(Box::new(cs)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Frees the reference to the corpus storage object.
/// - `ptr` - The corpus storage object.
///
/// # Safety
///
/// This functions dereferences the pointer given as argument and is therefore unsafe.
#[no_mangle]
pub unsafe extern "C" fn annis_cs_free(ptr: *mut CorpusStorage) {
    if ptr.is_null() {
        return;
    }
    // take ownership and destroy the pointer
    let ptr = Box::from_raw(ptr);
    std::mem::drop(ptr);
}

/// Count the number of results for a `query`.
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The name of the corpora to execute the query on.
/// - `query` - The query as string.
/// - `query_language` The query language of the query (e.g. AQL).
/// - `err` - Pointer to a list of errors. If any error occurred, this list will be non-empty.
///
/// Returns the count as number.
#[no_mangle]
pub extern "C" fn annis_cs_count(
    ptr: *const CorpusStorage,
    corpus_names: *const Vec<CString>,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> u64 {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();

    let search_query = SearchQuery {
        query: &query,
        corpus_names: &corpus_names,
        query_language,
        timeout: None,
    };

    map_cerr(cs.count(search_query), err).unwrap_or(0)
}

/// Count the number of results for a `query` and return both the total number of matches and also the number of documents in the result set.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The name of the corpora to execute the query on.
/// - `query` - The query as string.
/// - `query_language` The query language of the query (e.g. AQL).
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_count_extra(
    ptr: *const CorpusStorage,
    corpus_names: *const Vec<CString>,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> CountExtra {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();

    let search_query = SearchQuery {
        query: &query,
        corpus_names: &corpus_names,
        query_language,
        timeout: None,
    };
    map_cerr(cs.count_extra(search_query), err).unwrap_or_default()
}

/// Find all results for a `query` and return the match ID for each result.
///
/// The query is paginated and an offset and limit can be specified.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The name of the corpora to execute the query on.
/// - `query` - The query as string.
/// - `query_language` The query language of the query (e.g. AQL).
/// - `offset` - Skip the `n` first results, where `n` is the offset.
/// - `limit` - Return at most `n` matches, where `n` is the limit.  Use `None` to allow unlimited result sizes.
/// - `order` - Specify the order of the matches.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// Returns a vector of match IDs, where each match ID consists of the matched node annotation identifiers separated by spaces.
/// You can use the `annis_cs_subgraph(...)` method to get the subgraph for a single match described by the node annnotation identifiers.
///
/// # Safety
///
/// This functions dereferences the `err` pointer and is therefore unsafe.
#[no_mangle]
pub unsafe extern "C" fn annis_cs_find(
    ptr: *const CorpusStorage,
    corpus_names: *const Vec<CString>,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    offset: libc::size_t,
    limit: *const libc::size_t,
    order: ResultOrder,
    err: *mut *mut ErrorList,
) -> *mut Vec<CString> {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();

    let search_query = SearchQuery {
        query: &query,
        corpus_names: &corpus_names,
        query_language,
        timeout: None,
    };

    let limit = if limit.is_null() { None } else { Some(*limit) };

    map_cerr(cs.find(search_query, offset, limit, order), err)
        .map(|result| {
            let vec_result = result
                .into_iter()
                .map(|x| CString::new(x.as_str()).unwrap_or_default())
                .collect();
            Box::into_raw(Box::new(vec_result))
        })
        .unwrap_or_else(std::ptr::null_mut)
}

/// Return the copy of a subgraph which includes the given list of node annotation identifiers,
/// the nodes that cover the same token as the given nodes and
/// all nodes that cover the token which are part of the defined context.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
/// - `node_ids` - A set of node annotation identifiers describing the subgraph.
/// - `ctx_left` and `ctx_right` - Left and right context in token distance to be included in the subgraph.
/// - `segmentation` - The name of the segmentation which should be used to as base for the context. Use `None` to define the context in the default token layer.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// # Safety
///
/// This functions dereferences the `err` pointer and is therefore unsafe.
#[no_mangle]
pub extern "C" fn annis_cs_subgraph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    node_ids: *const Vec<CString>,
    ctx_left: libc::size_t,
    ctx_right: libc::size_t,
    segmentation: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> *mut AnnotationGraph {
    let cs: &CorpusStorage = cast_const(ptr);
    let node_ids: Vec<String> = cast_const(node_ids)
        .iter()
        .map(|id| String::from(id.to_string_lossy()))
        .collect();
    let corpus = cstr(corpus_name);

    let segmentation = if segmentation.is_null() {
        None
    } else {
        Some(cstr(segmentation).to_string())
    };

    map_cerr(
        cs.subgraph(&corpus, node_ids, ctx_left, ctx_right, segmentation),
        err,
    )
    .map(|result| Box::into_raw(Box::new(result)))
    .unwrap_or_else(std::ptr::null_mut)
}

/// Return the copy of a subgraph which includes all nodes that belong to any of the given list of sub-corpus/document identifiers.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
/// - `corpus_ids` - A set of sub-corpus/document identifiers describing the subgraph.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// # Safety
///
/// This functions dereferences the `err` pointer and is therefore unsafe.
#[no_mangle]
pub extern "C" fn annis_cs_subcorpus_graph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    corpus_ids: *const Vec<CString>,
    err: *mut *mut ErrorList,
) -> *mut AnnotationGraph {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus_ids: Vec<String> = cast_const(corpus_ids)
        .iter()
        .map(|id| String::from(id.to_string_lossy()))
        .collect();
    let corpus = cstr(corpus_name);

    map_cerr(cs.subcorpus_graph(&corpus, corpus_ids), err)
        .map(|result| Box::into_raw(Box::new(result)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Return the copy of the graph of the corpus structure given by `corpus_name`.
///
/// - `ptr` - The corpus storage object.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_corpus_graph(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> *mut AnnotationGraph {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);

    map_cerr(cs.corpus_graph(&corpus), err)
        .map(|result| Box::into_raw(Box::new(result)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Return the copy of a subgraph which includes all nodes matched by the given `query`.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
/// - `query` - The query which defines included nodes.
/// - `query_language` - The query language of the query (e.g. AQL).
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_subgraph_for_query(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> *mut AnnotationGraph {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);
    let query = cstr(query);

    map_cerr(
        cs.subgraph_for_query(&corpus, &query, query_language, None),
        err,
    )
    .map(|result| Box::into_raw(Box::new(result)))
    .unwrap_or_else(std::ptr::null_mut)
}

/// Return the copy of a subgraph which includes all nodes matched by the given `query` and an additional filter.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
/// - `query` - The query which defines included nodes.
/// - `query_language` - The query language of the query (e.g. AQL).
/// - `component_type_filter` - Only include edges of that belong to a component of the given type.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_subgraph_for_query_with_ctype(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    component_type_filter: AnnotationComponentType,
    err: *mut *mut ErrorList,
) -> *mut AnnotationGraph {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);
    let query = cstr(query);

    map_cerr(
        cs.subgraph_for_query(&corpus, &query, query_language, Some(component_type_filter)),
        err,
    )
    .map(|result| Box::into_raw(Box::new(result)))
    .unwrap_or_else(std::ptr::null_mut)
}

/// Execute a frequency query.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The name of the corpora to execute the query on.
/// - `query` - The query as string.
/// - `query_language` The query language of the query (e.g. AQL).
/// - `frequency_query_definition` - A string representation of the list of frequency query definitions.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// Returns a frequency table of strings.
#[no_mangle]
pub extern "C" fn annis_cs_frequency(
    ptr: *const CorpusStorage,
    corpus_names: *const Vec<CString>,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    frequency_query_definition: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> *mut FrequencyTable<CString> {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();

    let search_query = SearchQuery {
        query: &query,
        corpus_names: &corpus_names,
        query_language,
        timeout: None,
    };

    let frequency_query_definition = cstr(frequency_query_definition);
    let table_def: Vec<FrequencyDefEntry> = frequency_query_definition
        .split(',')
        .filter_map(|d| -> Option<FrequencyDefEntry> { d.parse().ok() })
        .collect();

    match map_cerr(cs.frequency(search_query, table_def), err) {
        Some(orig_ft) => {
            let mut result: FrequencyTable<CString> = FrequencyTable::new();

            for row in orig_ft.into_iter() {
                let mut new_tuple: Vec<CString> = Vec::with_capacity(row.values.len());
                for att in row.values.into_iter() {
                    if let Ok(att) = CString::new(att) {
                        new_tuple.push(att);
                    } else {
                        new_tuple.push(CString::default())
                    }
                }

                result.push(FrequencyTableRow {
                    values: new_tuple,
                    count: row.count,
                });
            }
            Box::into_raw(Box::new(result))
        }
        None => std::ptr::null_mut(),
    }
}

/// List all available corpora in the corpus storage.
///
/// - `ptr` - The corpus storage object.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_list(
    ptr: *const CorpusStorage,
    err: *mut *mut ErrorList,
) -> *mut Vec<CString> {
    let cs: &CorpusStorage = cast_const(ptr);

    let mut corpora: Vec<CString> = vec![];

    map_cerr(cs.list(), err)
        .map(|info| {
            for c in info {
                if let Ok(name) = CString::new(c.name) {
                    corpora.push(name);
                }
            }
            Box::into_raw(Box::new(corpora))
        })
        .unwrap_or_else(std::ptr::null_mut)
}

/// Returns a list of all node annotations of a corpus given by `corpus_name`.
///
/// - `ptr` - The corpus storage object.
/// - `list_values` - If true include the possible values in the result.
/// - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_list_node_annotations(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    list_values: bool,
    only_most_frequent_values: bool,
    err: *mut *mut ErrorList,
) -> *mut Matrix<CString> {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);

    map_cerr(
        cs.list_node_annotations(&corpus, list_values, only_most_frequent_values),
        err,
    )
    .map(|orig_vec| {
        let mut result: Matrix<CString> = Matrix::new();
        for anno in orig_vec.into_iter() {
            if let (Ok(ns), Ok(name), Ok(val)) = (
                CString::new(anno.key.ns.as_str()),
                CString::new(anno.key.name.as_str()),
                CString::new(anno.val.as_str()),
            ) {
                result.push(vec![ns, name, val]);
            }
        }
        Box::into_raw(Box::new(result))
    })
    .unwrap_or_else(std::ptr::null_mut)
}

/// Returns a list of all edge annotations of a corpus given by `corpus_name` and the component.
///
/// - `ptr` - The corpus storage object.
/// - `list_values` - If true include the possible values in the result.
/// - `component_type` - The type of the edge component.
/// - `component_name` - The name of the edge component.
/// - `component_layer` - The layer of the edge component.
/// - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_list_edge_annotations(
    ptr: *const CorpusStorage,
    corpus_name: *const libc::c_char,
    component_type: AnnotationComponentType,
    component_name: *const libc::c_char,
    component_layer: *const libc::c_char,
    list_values: bool,
    only_most_frequent_values: bool,
    err: *mut *mut ErrorList,
) -> *mut Matrix<CString> {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);
    let component = AnnotationComponent::new(
        component_type,
        cstr(component_layer).into(),
        cstr(component_name).into(),
    );

    map_cerr(
        cs.list_edge_annotations(&corpus, &component, list_values, only_most_frequent_values),
        err,
    )
    .map(|orig_vec| {
        let mut result: Matrix<CString> = Matrix::new();
        for anno in orig_vec.into_iter() {
            if let (Ok(ns), Ok(name), Ok(val)) = (
                CString::new(anno.key.ns.as_str()),
                CString::new(anno.key.name.as_str()),
                CString::new(anno.val.as_str()),
            ) {
                result.push(vec![ns, name, val]);
            }
        }
        Box::into_raw(Box::new(result))
    })
    .unwrap_or_else(std::ptr::null_mut)
}

/// Parses a `query` and checks if it is valid.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The name of the corpora the query would be executed on (needed to catch certain corpus-specific semantic errors).
/// - `query` - The query as string.
/// - `query_language` The query language of the query (e.g. AQL).
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// Returns `true` if valid and an error with the parser message if invalid.
#[no_mangle]
pub extern "C" fn annis_cs_validate_query(
    ptr: *const CorpusStorage,
    corpus_names: *const Vec<CString>,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> bool {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();

    map_cerr(
        cs.validate_query(&corpus_names, &query, query_language),
        err,
    )
    .unwrap_or(false)
}

/// Parses a `query`and return a list of descriptions for its nodes.
///
/// - `ptr` - The corpus storage object.
/// - `query` - The query to be analyzed.
/// - `query_language` - The query language of the query (e.g. AQL).
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_node_descriptions(
    ptr: *const CorpusStorage,
    query: *const libc::c_char,
    query_language: QueryLanguage,
    err: *mut *mut ErrorList,
) -> *mut Vec<QueryAttributeDescription> {
    let cs: &CorpusStorage = cast_const(ptr);

    let query = cstr(query);

    map_cerr(cs.node_descriptions(&query, query_language), err)
        .map(|result| Box::into_raw(Box::new(result)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Import a corpus from an external location on the file system into this corpus storage.
///
/// - `ptr` - The corpus storage object.
/// - `path` - The location on the file system where the corpus data is located.
/// - `format` - The format in which this corpus data is stored.
/// - `corpus_name` - Optionally override the name of the new corpus for file formats that already provide a corpus name.
/// - `disk_based` - If `true`, prefer disk-based annotation and graph storages instead of memory-only ones.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// Returns the name of the imported corpus.
/// The returned string must be deallocated by the caller using annis_str_free()!
#[no_mangle]
pub extern "C" fn annis_cs_import_from_fs(
    ptr: *mut CorpusStorage,
    path: *const libc::c_char,
    format: ImportFormat,
    corpus_name: *const libc::c_char,
    disk_based: bool,
    overwrite_existing: bool,
    err: *mut *mut ErrorList,
) -> *mut libc::c_char {
    let cs: &mut CorpusStorage = cast_mut(ptr);

    let override_corpus_name: Option<String> = if corpus_name.is_null() {
        None
    } else {
        Some(String::from(cstr(corpus_name)))
    };
    let path: &str = &cstr(path);
    map_cerr(
        cs.import_from_fs(
            &PathBuf::from(path),
            format,
            override_corpus_name,
            disk_based,
            overwrite_existing,
            |status| info!("{}", status),
        ),
        err,
    )
    .map(|corpus_name| {
        CString::new(corpus_name.as_str())
            .unwrap_or_default()
            .into_raw()
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Export a corpus to an external location on the file system using the given format.
///
/// - `ptr` - The corpus storage object.
/// - `corpus_names` - The corpora to include in the exported file(s).
/// - `path` - The location on the file system where the corpus data should be written to.
/// - `format` - The format in which this corpus data will be stored stored.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_export_to_fs(
    ptr: *mut CorpusStorage,
    corpus_names: *const Vec<CString>,
    path: *const libc::c_char,
    format: ExportFormat,
    err: *mut *mut ErrorList,
) {
    let cs: &mut CorpusStorage = cast_mut(ptr);
    let corpus_names: Vec<String> = cast_const(corpus_names)
        .iter()
        .map(|cn| String::from(cn.to_string_lossy()))
        .collect();
    let path: &str = &cstr(path);
    map_cerr(
        cs.export_to_fs(&corpus_names, &PathBuf::from(path), format),
        err,
    );
}

/// Returns a list of all components of a corpus given by `corpus_name` and the component type.
///
/// - `ptr` - The corpus storage object.
/// - `ctype` -Filter by the component type.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_list_components_by_type(
    ptr: *mut CorpusStorage,
    corpus_name: *const libc::c_char,
    ctype: AnnotationComponentType,
    err: *mut *mut ErrorList,
) -> *mut Vec<AnnotationComponent> {
    let cs: &CorpusStorage = cast_const(ptr);
    let corpus = cstr(corpus_name);

    map_cerr(cs.list_components(&corpus, Some(ctype), None), err)
        .map(|c| Box::into_raw(Box::new(c)))
        .unwrap_or_else(std::ptr::null_mut)
}

/// Delete a corpus from this corpus storage.
/// Returns `true` if the corpus was successfully deleted and `false` if no such corpus existed.
///
/// - `ptr` - The corpus storage object.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_delete(
    ptr: *mut CorpusStorage,
    corpus: *const libc::c_char,
    err: *mut *mut ErrorList,
) -> bool {
    let cs: &mut CorpusStorage = cast_mut(ptr);
    let corpus = cstr(corpus);

    map_cerr(cs.delete(&corpus), err).unwrap_or(false)
}

/// Unloads a corpus from the cache.
///
/// - `corpus` The name of the corpus to unload.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_cs_unload(
    ptr: *mut CorpusStorage,
    corpus: *const libc::c_char,
    err: *mut *mut ErrorList,
) {
    let cs: &mut CorpusStorage = cast_mut(ptr);
    let corpus = cstr(corpus);

    map_cerr(cs.unload(&corpus), err);
}

/// Apply a sequence of updates (`update` parameter) to this graph for a corpus given by the `corpus_name` parameter.
///
/// - `ptr` - The corpus storage object.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
///
/// It is ensured that the update process is atomic and that the changes are persisted to disk if the error list is empty.
#[no_mangle]
pub extern "C" fn annis_cs_apply_update(
    ptr: *mut CorpusStorage,
    corpus_name: *const libc::c_char,
    update: *mut GraphUpdate,
    err: *mut *mut ErrorList,
) {
    let cs: &mut CorpusStorage = cast_mut(ptr);
    let update: &mut GraphUpdate = cast_mut(update);
    let corpus_name = cstr(corpus_name);

    map_cerr(cs.apply_update(&corpus_name, update), err);
}
