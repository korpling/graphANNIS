use std::ffi::CString;
use libc::c_char;
use std;
use log;
use graphdb;
use api;
use relannis;

pub struct Error {
    msg: CString,
}


impl From<api::corpusstorage::Error> for Error {
    fn from(e: api::corpusstorage::Error) -> Error {
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

impl From<relannis::Error> for Error {
    fn from(e: relannis::Error) -> Error {
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

impl From<graphdb::Error> for Error {
    fn from(e: graphdb::Error) -> Error {
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

pub fn new(err : api::corpusstorage::Error) -> * mut Error {
    Box::into_raw(Box::new(Error::from(err)))
}

#[no_mangle]
pub extern "C" fn annis_error_get_msg(ptr : * const Error) -> * const c_char {
    let err: &Error = cast_const!(ptr);
    return err.msg.as_ptr();

}

