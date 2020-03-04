use super::data::{vec_get, vec_size};
use crate::errors;
use libc::{c_char, size_t};
use log;
use std;
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
            msg: CString::new(std_error.to_string()).unwrap_or(CString::default()),
            kind: CString::new("Cause").unwrap_or(CString::default()),
        };
        self.current = std_error.source();
        Some(result)
    }
}

fn error_kind(e: &errors::Error) -> &str {
    match e {
        errors::Error::AQLSyntaxError { .. } => "AQLSyntaxError",
        errors::Error::AQLSemanticError { .. } => "AQLSemanticError",
        errors::Error::LoadingGraphFailed { .. } => "LoadingGraphFailed",
        errors::Error::ImpossibleSearch(_) => "ImpossibleSearch",
        errors::Error::NoSuchCorpus(_) => "NoSuchCorpus",
        errors::Error::Generic { .. } => "Generic",
        errors::Error::IO(_) => "IO",
        errors::Error::Bincode(_) => "Bincode",
        errors::Error::CSV(_) => "CSV",
        errors::Error::ParseIntError(_) => "ParseIntError",
        errors::Error::Fmt(_) => "Fmt",
        errors::Error::Strum(_) => "Strum",
        errors::Error::Regex(_) => "Regex",
        errors::Error::RandomGenerator(_) => "RandomGenerator",
        errors::Error::Failure(_) => "Failure",
        errors::Error::SSTable(_) => "SSTable",
    }
}

impl From<errors::Error> for ErrorList {
    fn from(e: errors::Error) -> ErrorList {
        let mut result = ErrorList::new();
        result.push(Error {
            msg: CString::new(e.to_string()).unwrap_or(CString::default()),
            kind: CString::new(error_kind(&e)).unwrap_or(CString::default()),
        });
        let cause_it = CauseIterator {
            current: e.source(),
        };
        for e in cause_it {
            result.push(e)
        }
        return result;
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(e: log::SetLoggerError) -> Error {
        let err = if let Ok(error_msg) = CString::new(e.to_string()) {
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
        };
        return err;
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        let err = if let Ok(error_msg) = CString::new(e.to_string()) {
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
        };
        return err;
    }
}
/// Creates a new error from the internal type
pub fn new(err: errors::Error) -> *mut ErrorList {
    Box::into_raw(Box::new(ErrorList::from(err)))
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
    let err: &Error = cast_const!(item);
    return err.msg.as_ptr();
}

/// Get the kind or type for the error at position `i` in the list.
#[no_mangle]
pub extern "C" fn annis_error_get_kind(ptr: *const ErrorList, i: size_t) -> *const c_char {
    let item = vec_get(ptr, i);
    if item.is_null() {
        return std::ptr::null();
    }
    let err: &Error = cast_const!(item);
    return err.kind.as_ptr();
}
