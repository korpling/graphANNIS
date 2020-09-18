use super::cerror::{Error, ErrorList};
use super::safe_cstr;
use libc;
use simplelog;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::File;

/// Different levels of logging. Higher levels activate logging of events of lower levels as well.
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
    fn from(level: LogLevel) -> simplelog::LevelFilter {
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

/// Initialize the logging of this library.
///
/// - `logfile` - The file that is used to output the log messages.
/// - `level` - Minimum level to output.
/// - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
#[no_mangle]
pub extern "C" fn annis_init_logging(
    logfile: *const libc::c_char,
    level: LogLevel,
    err: *mut *mut ErrorList,
) {
    if !logfile.is_null() {
        let logfile: &str = &safe_cstr(logfile);

        match File::create(logfile) {
            Ok(f) => {
                if let Err(e) = WriteLogger::init(LevelFilter::from(level), Config::default(), f) {
                    // File was created, but logger was not.
                    if !err.is_null() {
                        unsafe {
                            *err = Box::into_raw(Box::new(vec![Error::from(e)]));
                        }
                    }
                }
            }
            Err(e) => {
                if !err.is_null() {
                    unsafe {
                        *err = Box::into_raw(Box::new(vec![Error::from(e)]));
                    }
                }
            }
        };
    }
}
