use std::ffi::CString;
use libc::{size_t, c_char, c_void};
use std;

use graphannis::{Annotation, NodeID};


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
pub extern "C" fn annis_vec_annotation_get(ptr : * const Vec<Annotation>, i : size_t) -> * const Annotation {
    vec_get(ptr, i)
}