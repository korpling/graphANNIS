extern crate graphannis;
extern crate libc;
#[macro_use]
extern crate log;
extern crate simplelog;

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
                assert!(!$x.is_null(), "Object argument was null");
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

pub type Matrix<T> = Vec<Vec<T>>;


pub mod corpusstorage;
pub mod update;
pub mod data;
pub mod error;
pub mod graph;
pub mod logging;