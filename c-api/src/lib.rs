extern crate libc;
extern crate graphannis;

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


pub mod corpusstorage;

// put all the C-APIs in parent scope
pub use corpusstorage::*;
