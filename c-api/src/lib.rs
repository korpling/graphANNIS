extern crate libc;
extern crate graphannis;

macro_rules! cast_mut {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&mut (*$x).0)
            }
        }
    };
}

macro_rules! cast_const {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&(*$x).0)
            }
        }
    };
}


pub mod corpusstorage;

// put all the C-APIs in parent scope
pub use corpusstorage::*;
