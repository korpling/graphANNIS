use super::cast_const;
use super::data::{vec_get, vec_size};
use graphannis::errors;
use libc::{c_char, size_t};
use std::error::Error as StdError;
use std::ffi::CString;

/// An representation of an internal error.
pub struct Error {
    /// The message for the user.
    pub msg: CString,
    // The general kind or type of error.
    pub kind: CString,
}

/// A list of multiple errors.
pub type ErrorList = Vec<Error>;

struct CauseIterator<'a> {
    current: Option<&'a dyn StdError>,
}

impl<'a> std::iter::Iterator for CauseIterator<'a> {
    type Item = Error;

    fn next(&mut self) -> std::option::Option<Error> {
        let std_error = self.current?;
        let result = Error {
            msg: CString::new(std_error.to_string()).unwrap_or_default(),
            kind: CString::new("Cause").unwrap_or_default(),
        };
        self.current = std_error.source();
        Some(result)
    }
}

#[allow(clippy::borrowed_box)]
fn error_kind(e: &Box<dyn StdError>) -> &'static str {
    if let Some(annis_err) = e.downcast_ref::<errors::GraphAnnisError>() {
        annis_err.into()
    } else {
        // Check for several known types
        if e.is::<std::io::Error>() {
            "IO"
        } else if e.is::<log::SetLoggerError>() {
            "SetLoggerError"
        } else {
            "Unknown"
        }
    }
}

pub fn create_error_list(e: Box<dyn StdError>) -> ErrorList {
    let mut result = vec![Error {
        msg: CString::new(e.to_string()).unwrap_or_default(),
        kind: CString::new(error_kind(&e)).unwrap_or_default(),
    }];
    let cause_it = CauseIterator {
        current: e.source(),
    };
    for e in cause_it {
        result.push(e)
    }
    result
}

impl From<log::SetLoggerError> for Error {
    fn from(e: log::SetLoggerError) -> Error {
        if let Ok(error_msg) = CString::new(e.to_string()) {
            Error {
                msg: error_msg,
                kind: CString::new("SetLoggerError").unwrap(),
            }
        } else {
            // meta-error
            Error {
                msg: CString::new(String::from("Some error occurred")).unwrap(),
                kind: CString::new("SetLoggerError").unwrap(),
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        if let Ok(error_msg) = CString::new(e.to_string()) {
            Error {
                msg: error_msg,
                kind: CString::new("std::io::Error").unwrap(),
            }
        } else {
            // meta-error
            Error {
                msg: CString::new(String::from("Some error occurred")).unwrap(),
                kind: CString::new("std::io::Error").unwrap(),
            }
        }
    }
}
/// Creates a new error from the internal type
pub fn new(err: Box<dyn StdError>) -> *mut ErrorList {
    Box::into_raw(Box::new(create_error_list(err)))
}

/// Returns the number of errors in the list.
#[no_mangle]
pub extern "C" fn annis_error_size(ptr: *const ErrorList) -> size_t {
    vec_size(ptr)
}

/// Get the message for the error at position `i` in the list.
#[no_mangle]
pub extern "C" fn annis_error_get_msg(ptr: *const ErrorList, i: size_t) -> *const c_char {
    let item = vec_get(ptr, i);
    if item.is_null() {
        return std::ptr::null();
    }
    let err: &Error = cast_const(item);
    err.msg.as_ptr()
}

/// Get the kind or type for the error at position `i` in the list.
#[no_mangle]
pub extern "C" fn annis_error_get_kind(ptr: *const ErrorList, i: size_t) -> *const c_char {
    let item = vec_get(ptr, i);
    if item.is_null() {
        return std::ptr::null();
    }
    let err: &Error = cast_const(item);
    err.kind.as_ptr()
}
