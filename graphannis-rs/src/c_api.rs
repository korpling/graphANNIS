
use annis::stringstorage::*;
use libc;
use std;

#[repr(C)]
pub struct annis_StringStoragePtr(StringStorage);

#[repr(C)]
pub struct annis_OptionalString {
    pub valid: libc::c_int,
    pub value: *const libc::c_char,
    pub length: libc::size_t,
}

#[repr(C)]
pub struct annis_Option_u32 {
    pub valid: libc::c_int,
    pub value: libc::uint32_t,
}


#[no_mangle]
pub extern "C" fn annis_stringstorage_new() -> *mut annis_StringStoragePtr {
    let s = StringStorage::new();
    Box::into_raw(Box::new(annis_StringStoragePtr(s)))
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_free(target: *mut annis_StringStoragePtr) {
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(target) };
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_str(target: *const annis_StringStoragePtr,
                                          id: libc::uint32_t)
                                          -> annis_OptionalString {

    let s = unsafe { &(*target).0 };
    let result = match s.str(id) {
        Some(v) => {
            annis_OptionalString {
                valid: 1,
                value: v.as_ptr() as *const libc::c_char,
                length: v.len(),
            }
        }
        None => {
            annis_OptionalString {
                valid: 0,
                value: std::ptr::null(),
                length: 0,
            }
        }
    };

    return result;
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_find_id(target: *const annis_StringStoragePtr,
                                              value: *const libc::c_char)
                                              -> annis_Option_u32 {
    let s = unsafe { &(*target).0 };
    let wrapped_str = unsafe { std::ffi::CStr::from_ptr(value) };

    let result = match std::str::from_utf8(wrapped_str.to_bytes()) {
        Ok(v) => {
            match s.find_id(v) {
                Some(x) => {
                    annis_Option_u32 {
                        valid: 1,
                        value: *x,
                    }
                }
                None => {
                    annis_Option_u32 {
                        valid: 0,
                        value: 0,
                    }
                }
            }
        }
        Err(_) => {
            annis_Option_u32 {
                valid: 0,
                value: 0,
            }
        }
    };

    return result;
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_add(target: *mut annis_StringStoragePtr,
                                          value: *const libc::c_char)
                                          -> libc::uint32_t {
    let mut s = unsafe { &mut (*target).0 };
    let wrapped_str = unsafe { std::ffi::CStr::from_ptr(value) };

    match std::str::from_utf8(wrapped_str.to_bytes()) {
        Ok(v) => s.add(v),
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_clear(target: *mut annis_StringStoragePtr) {
    let mut s = unsafe { &mut (*target).0 };
    s.clear();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_len(target: *const annis_StringStoragePtr) -> libc::size_t {
    let s = unsafe { &(*target).0 };
    return s.len();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_avg_length(target: *const annis_StringStoragePtr)
                                                 -> libc::c_double {
    let s = unsafe { &(*target).0 };
    return s.avg_length();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_save_to_file(target: *const annis_StringStoragePtr,
                                                   path: *const libc::c_char) {
    let s = unsafe { &(*target).0 };
    let wrapped_str = unsafe { std::ffi::CStr::from_ptr(path) };
    let safe_path = std::str::from_utf8(wrapped_str.to_bytes());
    if safe_path.is_ok() {
        s.save_to_file(safe_path.unwrap());
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_load_from_file(target: *mut annis_StringStoragePtr,
                                                   path: *const libc::c_char) {
    let s = unsafe { &mut(*target).0 };
    let wrapped_str = unsafe { std::ffi::CStr::from_ptr(path) };
    let safe_path = std::str::from_utf8(wrapped_str.to_bytes());
    if safe_path.is_ok() {
        s.load_from_file(safe_path.unwrap());
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_estimate_memory(target: *const annis_StringStoragePtr) -> libc::size_t {
    let s = unsafe { &(*target).0 };

    return s.estimate_memory_size();
}
