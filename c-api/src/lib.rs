extern crate graphannis;
extern crate libc;

use std::ffi::CString;
use libc::c_char;

#[allow(unused_macros)]
macro_rules! cast_mut {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&mut (*$x))
            }
        }
    };
}

macro_rules! cast_const {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&(*$x))
            }
        }
    };
}

macro_rules! cstr {
    ($x:expr) => {
        unsafe {
            use std::borrow::Cow;
            if $x.is_null() {
                Cow::from("")
            } else {
                std::ffi::CStr::from_ptr($x).to_string_lossy()
            }
        }
    }
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

pub fn error_string(e: graphannis::api::corpusstorage::Error) -> *mut c_char {
    if let Ok(error_msg) = CString::new(String::from(format!("{:?}", e))) {
        error_msg.into_raw()
    } else {
        // meta-error
        CString::new(String::from("Some error occured"))
            .unwrap()
            .into_raw()
    }
}

pub mod corpusstorage;
pub mod update;

// put all the C-APIs in parent scope
pub use corpusstorage::*;
pub use update::*;
pub use graphannis::api::update::UpdateEvent;
