use std::ffi::CString;
use libc::c_char;
use std;
use log;
use errors;

pub struct Error {
    pub msg: CString,
    pub kind: CString,
}

pub type ErrorList = Vec<Error>;

impl From<errors::Error> for ErrorList {
    fn from(e: errors::Error) -> ErrorList {
        let mut result = ErrorList::new();
        result.push(Error {
            msg: CString::new(e.to_string()).unwrap_or(CString::default()),
            kind: CString::new(e.kind().description()).unwrap_or(CString::default()), 
        });
        for e in e.iter().skip(1) {
            result.push(Error {
                msg: CString::new(e.to_string()).unwrap_or(CString::default()),
                kind: CString::new("Cause").unwrap_or(CString::default()), 
            });
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
                msg:  CString::new(String::from("Some error occured")).unwrap(),
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
                msg:  CString::new(String::from("Some error occured")).unwrap(),
                kind: CString::new("std::io::Error").unwrap(),
            }
        };
        return err;
    }
}

pub fn new(err : errors::Error) -> * mut ErrorList {
    Box::into_raw(Box::new(ErrorList::from(err)))
}

#[no_mangle]
pub extern "C" fn annis_error_get_msg(ptr : * const Error) -> * const c_char {
    let err: &Error = cast_const!(ptr);
    return err.msg.as_ptr();

}

