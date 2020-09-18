#[macro_use]
extern crate log;

use std::borrow::Cow;

fn cast_mut<'a, T>(x: *mut T) -> &'a mut T {
    unsafe {
        assert!(!x.is_null());
        &mut (*x)
    }
}

fn cast_const<'a, T>(x: *const T) -> &'a T {
    unsafe {
        assert!(!x.is_null(), "Object argument was null");
        &(*x)
    }
}

fn cstr<'a>(orig: *const libc::c_char) -> Cow<'a, str> {
    unsafe {
        if orig.is_null() {
            Cow::from("")
        } else {
            std::ffi::CStr::from_ptr(orig).to_string_lossy()
        }
    }
}

macro_rules! try_cerr {
    ($x:expr , $err_ptr:ident, $default_return_val:expr) => {
        match $x {
            Ok(v) => v,
            Err(err) => {
                if !$err_ptr.is_null() {
                    unsafe {
                        *$err_ptr = cerror::new(err.into());
                    }
                }
                return $default_return_val;
            }
        }
    };
}

/// Simple definition of a matrix from a single data type.
pub type Matrix<T> = Vec<Vec<T>>;

pub mod cerror;
pub mod corpusstorage;
pub mod data;
pub mod graph;
pub mod logging;
pub mod update;
