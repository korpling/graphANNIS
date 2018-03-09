extern crate libc;
extern crate graphannis;

use std::ffi::CString;
use libc::{c_char};

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

#[repr(C)]
pub struct OptError {
    is_error: bool,
    error_msg: *mut c_char,
}

#[no_mangle]
pub extern "C" fn annis_free_str(s : *mut c_char) {
    unsafe {
        if s.is_null() { return }
        // take ownership and destruct
        CString::from_raw(s)
    };
}

impl From<graphannis::api::corpusstorage::Error> for OptError {
    fn from(e: graphannis::api::corpusstorage::Error) -> OptError {
        if let Ok(error_msg) = CString::new(String::from(format!("{:?}", e))) {
            OptError {
                is_error: true,
                error_msg: error_msg.into_raw(),
            }
        } else {
            // meta-error
            OptError {
                is_error: true,
                error_msg: std::ptr::null_mut(),
            }
        }
        
    }
}


pub mod corpusstorage;
pub mod update;

// put all the C-APIs in parent scope
pub use corpusstorage::*;
pub use update::*;
pub use graphannis::api::update::UpdateEvent;