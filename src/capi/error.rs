use std::ffi::CString;
use libc::c_char;
use std;
use log;
use errors;

pub struct Error {
    msg: CString,
}


impl From<errors::Error> for Error {
    fn from(e: errors::Error) -> Error {
        let err = if let Ok(error_msg) = CString::new(String::from(format!("{:?}", e))) {
            Error {
                msg: error_msg,
            }
        } else {
            // meta-error
            Error {
                msg:  CString::new(String::from("Some error occured")).unwrap(),
            }
        };
        return err;
    }
}


impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        let err = if let Ok(error_msg) = CString::new(String::from(format!("{:?}", e))) {
            Error {
                msg: error_msg,
            }
        } else {
            // meta-error
            Error {
                msg:  CString::new(String::from("Some error occured")).unwrap(),
            }
        };
        return err;
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(e: log::SetLoggerError) -> Error {
        let err = if let Ok(error_msg) = CString::new(String::from(format!("{:?}", e))) {
            Error {
                msg: error_msg,
            }
        } else {
            // meta-error
            Error {
                msg:  CString::new(String::from("Some error occured")).unwrap(),
            }
        };
        return err;
    }
}

pub fn new(err : errors::Error) -> * mut Error {
    Box::into_raw(Box::new(Error::from(err)))
}

#[no_mangle]
pub extern "C" fn annis_error_get_msg(ptr : * const Error) -> * const c_char {
    let err: &Error = cast_const!(ptr);
    return err.msg.as_ptr();

}

