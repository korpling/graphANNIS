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
            assert!(!$x.is_null());
            std::ffi::CStr::from_ptr($x)
        }
    }
}

#[repr(C)]
pub struct OptError {
    is_error: bool,
    error_msg: *const c_char,
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
                error_msg: std::ptr::null(),
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