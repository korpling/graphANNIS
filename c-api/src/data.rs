use std::ffi::CString;
use libc::{size_t, c_char, c_void};
use std;


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

#[no_mangle]
pub extern "C" fn annis_stringvec_size(ptr : * const Vec<CString>) -> size_t {
    let strvec : &Vec<CString> = cast_const!(ptr);
    return strvec.len();
}

#[no_mangle]
pub extern "C" fn annis_stringvec_get(ptr : * const Vec<CString>, i : size_t) -> * const c_char {
    let strvec : &Vec<CString> = cast_const!(ptr);
    if i < strvec.len() {
        return strvec[i].as_ptr();
    } else {
        return std::ptr::null();
    }
}