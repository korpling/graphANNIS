use std::ffi::CString;
use libc::{size_t, c_char, c_void};
use std;

use {Matrix,FrequencyTable,Annotation, NodeID, Edge, Component, NodeDesc};

#[no_mangle]
pub extern "C" fn annis_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_str_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        // take ownership and destruct
        CString::from_raw(s)
    };
}

/// Allocates a new char* based on an existing internal string. 
/// 
/// Result must be manually freed with annis_str_free(char* )! 
#[no_mangle]
pub extern "C" fn annis_string_copy(ptr : * const String) -> * mut c_char {
    let strref : &String = cast_const!(ptr);
    let cstr = CString::new(strref.as_str()).unwrap_or(CString::default());
    return cstr.into_raw();
}

pub type IterPtr<T> = Box<Iterator<Item=T>>;

pub fn iter_next<T>(ptr : * mut Box<Iterator<Item=T>>) -> * mut T {
    let it : &mut Box<Iterator<Item=T>> = cast_mut!(ptr);
    if let Some(v) = it.next() {
        return Box::into_raw(Box::new(v));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_iter_nodeid_next(ptr : * mut IterPtr<NodeID>) -> * mut NodeID {return iter_next(ptr);}

pub fn vec_size<T>(ptr : * const Vec<T>) -> size_t {
    let v : &Vec<T> = cast_const!(ptr);
    return v.len();
}

pub fn vec_get<T>(ptr : * const Vec<T>, i : size_t) -> * const T {
    let v : &Vec<T> = cast_const!(ptr);
    if i < v.len() {
        return &v[i] as * const T;
    }
    return std::ptr::null();
}

#[no_mangle]
pub extern "C" fn annis_vec_str_size(ptr : * const Vec<CString>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_vec_str_get(ptr : * const Vec<CString>, i : size_t) -> * const c_char {
    // custom implementation for string vectors, don't return a referance to CString but a char pointer
    let strvec : &Vec<CString> = cast_const!(ptr);
    if i < strvec.len() {
        return strvec[i].as_ptr();
    } else {
        return std::ptr::null();
    }
}

#[no_mangle]
pub extern "C" fn annis_vec_str_new() -> * mut Vec<CString> {
    let result : Vec<CString> = Vec::new();
    return Box::into_raw(Box::new(result));
}

#[no_mangle]
pub extern "C" fn annis_vec_str_push(ptr : * mut Vec<CString>, v : * const c_char) {
    let strvec : &mut Vec<CString> = cast_mut!(ptr);
    let v : &str = &cstr!(v);
    if let Ok(cval) = CString::new(v) {
        strvec.push(cval);
    }
}

#[no_mangle]
pub extern "C" fn annis_vec_annotation_size(ptr : * const Vec<Annotation>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_vec_annotation_get(ptr : * const Vec<Annotation>, i : size_t) -> * const Annotation {vec_get(ptr, i)}

#[no_mangle]
pub extern "C" fn annis_vec_edge_size(ptr : * const Vec<Edge>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_vec_edge_get(ptr : * const Vec<Edge>, i : size_t) -> * const Edge { vec_get(ptr, i)}

#[no_mangle]
pub extern "C" fn annis_vec_component_size(ptr : * const Vec<Component>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_vec_component_get(ptr : * const Vec<Component>, i : size_t) -> * const Component { vec_get(ptr, i)}

#[no_mangle]
pub extern "C" fn annis_vec_nodedesc_size(ptr : * const Vec<NodeDesc>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_vec_nodedesc_get(ptr : * const Vec<NodeDesc>, i : size_t) -> * const NodeDesc { vec_get(ptr, i)}

#[no_mangle]
pub extern "C" fn annis_matrix_str_nrows(ptr : * const Matrix<CString>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_matrix_str_ncols(ptr : * const Matrix<CString>) -> size_t {
    let v : &Vec<Vec<CString>> = cast_const!(ptr);
    if !v.is_empty() {
        return v[0].len();
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn annis_matrix_str_get(ptr : * const Matrix<CString>, row : size_t, col : size_t) -> * const c_char {
    // custom implementation for string matrix, don't return a referance to CString but a char pointer
    let strmatrix : &Vec<Vec<CString>> = cast_const!(ptr);
    if row < strmatrix.len() {
        if col < strmatrix[row].len() {
            return strmatrix[row][col].as_ptr();
        }
    }
    return std::ptr::null();
}

#[no_mangle]
pub extern "C" fn annis_freqtable_str_nrows(ptr : * const FrequencyTable<CString>) -> size_t {vec_size(ptr)}

#[no_mangle]
pub extern "C" fn annis_freqtable_str_ncols(ptr : * const FrequencyTable<CString>) -> size_t {
    let v : &FrequencyTable<CString> = cast_const!(ptr);
    if !v.is_empty() {
        return v[0].0.len();
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn annis_freqtable_str_get(ptr : * const FrequencyTable<CString>, row : size_t, col : size_t) -> * const c_char {
    // custom implementation for string matrix, don't return a referance to CString but a char pointer
    let ft : &FrequencyTable<CString> = cast_const!(ptr);
    if row < ft.len() {
        if col < ft[row].0.len() {
            return ft[row].0[col].as_ptr();
        }
    }
    return std::ptr::null();
}

#[no_mangle]
pub extern "C" fn annis_freqtable_str_count(ptr : * const FrequencyTable<CString>, row : size_t) -> size_t {
    let ft : &FrequencyTable<CString> = cast_const!(ptr);
    if row < ft.len() {
        return ft[row].1;
    }
    return 0;
}