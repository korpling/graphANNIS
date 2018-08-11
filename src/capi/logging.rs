use simplelog::{LevelFilter, WriteLogger, Config};
use std::fs::File;
use super::error::Error;
use libc;
use std;
use simplelog;

#[repr(C)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for simplelog::LevelFilter {
    fn from(level : LogLevel) -> simplelog::LevelFilter {
        match level {
            LogLevel::Off => simplelog::LevelFilter::Off,
            LogLevel::Error => simplelog::LevelFilter::Error,
            LogLevel::Warn => simplelog::LevelFilter::Warn,
            LogLevel::Info => simplelog::LevelFilter::Info,
            LogLevel::Debug => simplelog::LevelFilter::Debug,   
            LogLevel::Trace => simplelog::LevelFilter::Trace, 
        }
    }
}

#[no_mangle]
pub extern "C" fn annis_init_logging(logfile : * const libc::c_char, level : LogLevel) -> * mut Error {
    
    if !logfile.is_null() {
        let logfile : &str = &cstr!(logfile);
        
        match File::create(logfile) {
            Ok(f) => {
                if let Err(err) = WriteLogger::init(LevelFilter::from(level), Config::default(), f) {
                    // File was created, but logger was not.
                    return Box::into_raw(Box::new(Error::from(err)));
                }
            }
            Err(err) => {
                return Box::into_raw(Box::new(Error::from(err)));
            }
        };
    }
    return std::ptr::null_mut();
}
