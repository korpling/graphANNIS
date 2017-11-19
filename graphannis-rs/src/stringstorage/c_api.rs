use libc;
use std::ffi::CStr;
use util::c_api::*;
use super::*;


#[repr(C)]
pub struct annis_StringStoragePtr(pub StringStorage);

#[no_mangle]
pub extern "C" fn annis_stringstorage_new() -> *mut annis_StringStoragePtr {
    let s = StringStorage::new();
    Box::into_raw(Box::new(annis_StringStoragePtr(s)))
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_free(ptr: *mut annis_StringStoragePtr) {
    if ptr.is_null() {
        return;
    };
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_str(
    ptr: *const annis_StringStoragePtr,
    id: libc::uint32_t,
) -> annis_Option_String {

    match cast_const!(ptr).str(id) {
        Some(v) => annis_Option_String {
            valid: true,
            value: annis_String {
                s: v.as_ptr() as *const libc::c_char,
                length: v.len(),
            },
        },
        None => annis_Option_String {
            valid: false,
            value: annis_String {
                s: std::ptr::null(),
                length: 0,
            },
        },
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_find_id(
    ptr: *const annis_StringStoragePtr,
    value: *const libc::c_char,
) -> annis_Option_u32 {
    let s = cast_const!(ptr);

    let c_value = unsafe {
        assert!(!value.is_null());
        CStr::from_ptr(value)
    };

    let result = match c_value.to_str() {
        Ok(v) => annis_Option_u32::from_ref(s.find_id(v)),
        Err(_) => annis_Option_u32::invalid(),
    };

    return result;
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_add(
    ptr: *mut annis_StringStoragePtr,
    value: *const libc::c_char,
) -> libc::uint32_t {
    let s = cast_mut!(ptr);

    let c_value = unsafe {
        assert!(!value.is_null());
        CStr::from_ptr(value)
    };

    match c_value.to_str() {
        Ok(v) => s.add(v),
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_clear(ptr: *mut annis_StringStoragePtr) {
    cast_mut!(ptr).clear();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_len(ptr: *const annis_StringStoragePtr) -> libc::size_t {
    return cast_const!(ptr).len();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_avg_length(
    ptr: *const annis_StringStoragePtr,
) -> libc::c_double {
    return cast_const!(ptr).avg_length();
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_save_to_file(
    ptr: *const annis_StringStoragePtr,
    path: *const libc::c_char,
) {
    let s = cast_const!(ptr);
    let c_path = unsafe {
        assert!(!path.is_null());
        CStr::from_ptr(path)
    };
    let safe_path = c_path.to_str();
    if safe_path.is_ok() {
        s.save_to_file(safe_path.unwrap());
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_load_from_file(
    ptr: *mut annis_StringStoragePtr,
    path: *const libc::c_char,
) {
    let s = cast_mut!(ptr);
    let c_path = unsafe {
        assert!(!path.is_null());
        CStr::from_ptr(path)
    };
    let safe_path = c_path.to_str();
    if safe_path.is_ok() {
        s.load_from_file(safe_path.unwrap());
    }
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_estimate_memory(
    ptr: *const annis_StringStoragePtr,
) -> libc::size_t {
    return cast_const!(ptr).estimate_memory_size();
}
