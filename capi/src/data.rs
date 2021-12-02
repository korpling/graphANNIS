use super::Matrix;
use super::{cast_const, cast_mut, cstr};
use graphannis::{
    corpusstorage::{FrequencyTable, QueryAttributeDescription},
    graph::{Annotation, Edge, NodeID},
    model::AnnotationComponent,
};
use libc::{c_char, c_void, size_t};
use std::ffi::CString;

/// Frees the internal object given as `ptr` argument.
///
/// # Safety
///
/// This functions dereferences the `ptr` pointer and is therefore unsafe.
#[no_mangle]
pub unsafe extern "C" fn annis_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    // take ownership and destroy the pointer
    Box::from_raw(ptr);
}

/// Frees the string given as `s` argument.
///
/// # Safety
///
/// This functions dereferences the `s` pointer and is therefore unsafe.
#[no_mangle]
pub unsafe extern "C" fn annis_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // take ownership and destruct
    drop(CString::from_raw(s));
}

pub type IterPtr<T> = Box<dyn Iterator<Item = T>>;

pub fn iter_next<T>(ptr: *mut Box<dyn Iterator<Item = T>>) -> *mut T {
    let it: &mut Box<dyn Iterator<Item = T>> = cast_mut(ptr);
    if let Some(v) = it.next() {
        return Box::into_raw(Box::new(v));
    }
    std::ptr::null_mut()
}

/// Returns a pointer to the next node ID for the iterator given by the `ptr` argument
/// or `NULL` if iterator is empty.
#[no_mangle]
pub extern "C" fn annis_iter_nodeid_next(ptr: *mut IterPtr<NodeID>) -> *mut NodeID {
    iter_next(ptr)
}

pub fn vec_size<T>(ptr: *const Vec<T>) -> size_t {
    let v: &Vec<T> = cast_const(ptr);
    v.len()
}

pub fn vec_get<T>(ptr: *const Vec<T>, i: size_t) -> *const T {
    let v: &Vec<T> = cast_const(ptr);
    if i < v.len() {
        return &v[i] as *const T;
    }
    std::ptr::null()
}

/// Returns the number of elements of the string vector.
#[no_mangle]
pub extern "C" fn annis_vec_str_size(ptr: *const Vec<CString>) -> size_t {
    vec_size(ptr)
}

/// Get a read-only reference to the string at position `i` of the vector.
#[no_mangle]
pub extern "C" fn annis_vec_str_get(ptr: *const Vec<CString>, i: size_t) -> *const c_char {
    // custom implementation for string vectors, don't return a referance to CString but a char pointer
    let strvec: &Vec<CString> = cast_const(ptr);
    if i < strvec.len() {
        strvec[i].as_ptr()
    } else {
        std::ptr::null()
    }
}

/// Create a new string vector.
#[no_mangle]
pub extern "C" fn annis_vec_str_new() -> *mut Vec<CString> {
    let result: Vec<CString> = Vec::new();
    Box::into_raw(Box::new(result))
}

/// Add an element to the string vector.
#[no_mangle]
pub extern "C" fn annis_vec_str_push(ptr: *mut Vec<CString>, v: *const c_char) {
    let strvec: &mut Vec<CString> = cast_mut(ptr);
    let v: &str = &cstr(v);
    if let Ok(cval) = CString::new(v) {
        strvec.push(cval);
    }
}

/// Get the namespace of the given annotation object.
#[no_mangle]
pub extern "C" fn annis_annotation_ns(ptr: *const Annotation) -> *mut c_char {
    let anno: &Annotation = cast_const(ptr);
    CString::new(anno.key.ns.as_str())
        .unwrap_or_default()
        .into_raw()
}

/// Get the name of the given annotation object.
#[no_mangle]
pub extern "C" fn annis_annotation_name(ptr: *const Annotation) -> *mut c_char {
    let anno: &Annotation = cast_const(ptr);
    CString::new(anno.key.name.as_str())
        .unwrap_or_default()
        .into_raw()
}

/// Get the value of the given annotation object.
#[no_mangle]
pub extern "C" fn annis_annotation_val(ptr: *const Annotation) -> *mut c_char {
    let anno: &Annotation = cast_const(ptr);
    CString::new(anno.val.as_str())
        .unwrap_or_default()
        .into_raw()
}

/// Returns the number of elements of the annotation vector.
#[no_mangle]
pub extern "C" fn annis_vec_annotation_size(ptr: *const Vec<Annotation>) -> size_t {
    vec_size(ptr)
}

/// Get a read-only reference to the annotation at position `i` of the vector.
#[no_mangle]
pub extern "C" fn annis_vec_annotation_get(
    ptr: *const Vec<Annotation>,
    i: size_t,
) -> *const Annotation {
    vec_get(ptr, i)
}

/// Returns the number of elements of the edge vector.
#[no_mangle]
pub extern "C" fn annis_vec_edge_size(ptr: *const Vec<Edge>) -> size_t {
    vec_size(ptr)
}

/// Get a read-only reference to the edge at position `i` of the vector.
#[no_mangle]
pub extern "C" fn annis_vec_edge_get(ptr: *const Vec<Edge>, i: size_t) -> *const Edge {
    vec_get(ptr, i)
}

/// Returns the number of elements of the component vector.
#[no_mangle]
pub extern "C" fn annis_vec_component_size(ptr: *const Vec<AnnotationComponent>) -> size_t {
    vec_size(ptr)
}

/// Get a read-only reference to the component at position `i` of the vector.
#[no_mangle]
pub extern "C" fn annis_vec_component_get(
    ptr: *const Vec<AnnotationComponent>,
    i: size_t,
) -> *const AnnotationComponent {
    vec_get(ptr, i)
}

/// Returns the number of elements of the query attribute description vector.
#[no_mangle]
pub extern "C" fn annis_vec_qattdesc_size(ptr: *const Vec<QueryAttributeDescription>) -> size_t {
    vec_size(ptr)
}

/// Get a read-only reference to the query attribute description at position `i` of the vector.
#[no_mangle]
pub extern "C" fn annis_vec_qattdesc_get_component_nr(
    ptr: *const Vec<QueryAttributeDescription>,
    i: size_t,
) -> usize {
    let desc_ptr: *const QueryAttributeDescription = vec_get(ptr, i);
    let desc: &QueryAttributeDescription = cast_const(desc_ptr);
    desc.alternative
}

/// Create a string representing the AQL fragment part of the query attribute description.
///
/// The resulting char* must be freeed with annis_str_free!
#[no_mangle]
pub extern "C" fn annis_vec_qattdesc_get_aql_fragment(
    ptr: *const Vec<QueryAttributeDescription>,
    i: size_t,
) -> *mut c_char {
    let desc_ptr: *const QueryAttributeDescription = vec_get(ptr, i);
    let desc: &QueryAttributeDescription = cast_const(desc_ptr);
    let cstr: CString = CString::new(desc.query_fragment.as_str()).unwrap_or_default();
    cstr.into_raw()
}

/// Create a string representing the variable part of the query attribute description.
///
/// The resulting char* must be freeed with annis_str_free!
#[no_mangle]
pub extern "C" fn annis_vec_qattdesc_get_variable(
    ptr: *const Vec<QueryAttributeDescription>,
    i: size_t,
) -> *mut c_char {
    let desc_ptr: *const QueryAttributeDescription = vec_get(ptr, i);
    let desc: &QueryAttributeDescription = cast_const(desc_ptr);
    let cstr: CString = CString::new(desc.variable.as_str()).unwrap_or_default();
    cstr.into_raw()
}

/// Create a string representing the annotation name part of the query attribute description.
///
/// The resulting char* must be freeed with annis_str_free!
#[no_mangle]
pub extern "C" fn annis_vec_qattdesc_get_anno_name(
    ptr: *const Vec<QueryAttributeDescription>,
    i: size_t,
) -> *mut c_char {
    let desc_ptr: *const QueryAttributeDescription = vec_get(ptr, i);
    let desc: &QueryAttributeDescription = cast_const(desc_ptr);
    if let Some(ref anno_name) = desc.anno_name {
        let cstr: CString = CString::new(anno_name.as_str()).unwrap_or_default();
        cstr.into_raw()
    } else {
        std::ptr::null_mut()
    }
}

/// Returns the number of rows of the string matrix.
#[no_mangle]
pub extern "C" fn annis_matrix_str_nrows(ptr: *const Matrix<CString>) -> size_t {
    vec_size(ptr)
}

/// Returns the number of columns of the string matrix.
#[no_mangle]
pub extern "C" fn annis_matrix_str_ncols(ptr: *const Matrix<CString>) -> size_t {
    let v: &Vec<Vec<CString>> = cast_const(ptr);
    if !v.is_empty() {
        return v[0].len();
    }
    0
}

/// Get a read-only reference to the string at the at position (`row`, `col`) of the matrix.
#[no_mangle]
pub extern "C" fn annis_matrix_str_get(
    ptr: *const Matrix<CString>,
    row: size_t,
    col: size_t,
) -> *const c_char {
    // custom implementation for string matrix, don't return a referance to CString but a char pointer
    let strmatrix: &Vec<Vec<CString>> = cast_const(ptr);
    if row < strmatrix.len() && col < strmatrix[row].len() {
        return strmatrix[row][col].as_ptr();
    }
    std::ptr::null()
}

/// Returns the number of rows of the frequency table.
#[no_mangle]
pub extern "C" fn annis_freqtable_str_nrows(ptr: *const FrequencyTable<CString>) -> size_t {
    vec_size(ptr)
}

/// Returns the number of columns of the frequency table.
#[no_mangle]
pub extern "C" fn annis_freqtable_str_ncols(ptr: *const FrequencyTable<CString>) -> size_t {
    let v: &FrequencyTable<CString> = cast_const(ptr);
    if !v.is_empty() {
        return v[0].values.len();
    }
    0
}

/// Get a read-only reference to the string at the at position (`row`, `col`) of the frequency table.
#[no_mangle]
pub extern "C" fn annis_freqtable_str_get(
    ptr: *const FrequencyTable<CString>,
    row: size_t,
    col: size_t,
) -> *const c_char {
    // custom implementation for string matrix, don't return a referance to CString but a char pointer
    let ft: &FrequencyTable<CString> = cast_const(ptr);
    if row < ft.len() && col < ft[row].values.len() {
        return ft[row].values[col].as_ptr();
    }
    std::ptr::null()
}

/// Get the count of the `row` of the frequency table.
#[no_mangle]
pub extern "C" fn annis_freqtable_str_count(
    ptr: *const FrequencyTable<CString>,
    row: size_t,
) -> size_t {
    let ft: &FrequencyTable<CString> = cast_const(ptr);
    if row < ft.len() {
        return ft[row].count;
    }
    0
}
