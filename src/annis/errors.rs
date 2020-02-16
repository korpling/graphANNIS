use crate::annis::types::LineColumnRange;
use serde::{de, ser};
use std::error::Error as StdError;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AQLSyntaxError {
        desc: String,
        location: Option<LineColumnRange>,
    },
    AQLSemanticError {
        desc: String,
        location: Option<LineColumnRange>,
    },
    ImpossibleSearch(String),
    LoadingGraphFailed {
        name: String,
    },
    NoSuchCorpus(String),
    Generic {
        msg: String,
        cause: Option<Box<dyn StdError + 'static + Send>>,
    },
    IO(std::io::Error),
    Bincode(::bincode::Error),
    CSV(::csv::Error),
    ParseIntError(::std::num::ParseIntError),
    Fmt(::std::fmt::Error),
    Strum(::strum::ParseError),
    Regex(::regex::Error),
    RandomGenerator(::rand::Error),
    Failure(failure::Compat<failure::Error>),
    SSTable(sstable::error::Status),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl std::convert::From<tempfile::PersistError> for Error {
    fn from(e: tempfile::PersistError) -> Error {
        Error::IO(e.error)
    }
}

impl std::convert::From<::bincode::Error> for Error {
    fn from(e: ::bincode::Error) -> Error {
        Error::Bincode(e)
    }
}

impl std::convert::Into<::bincode::Error> for Error {
    fn into(self) -> ::bincode::Error {
        Box::new(::bincode::ErrorKind::Custom(format!("{}", &self)))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Generic {
            msg: msg.to_string(),
            cause: None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Generic {
            msg: msg.to_string(),
            cause: None,
        }
    }
}

impl std::convert::From<::csv::Error> for Error {
    fn from(e: ::csv::Error) -> Error {
        Error::CSV(e)
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

impl std::convert::From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Error {
        Error::Fmt(e)
    }
}

impl std::convert::From<strum::ParseError> for Error {
    fn from(e: strum::ParseError) -> Error {
        Error::Strum(e)
    }
}

impl std::convert::From<regex::Error> for Error {
    fn from(e: regex::Error) -> Error {
        Error::Regex(e)
    }
}

impl std::convert::From<::rand::Error> for Error {
    fn from(e: ::rand::Error) -> Error {
        Error::RandomGenerator(e)
    }
}

impl std::convert::From<failure::Error> for Error {
    fn from(e: failure::Error) -> Error {
        Error::Failure(e.compat())
    }
}

impl std::convert::From<sstable::error::Status> for Error {
    fn from(e: sstable::error::Status) -> Error {
        Error::SSTable(e)
    }
}

impl std::convert::From<&str> for Error {
    fn from(e: &str) -> Error {
        Error::Generic {
            msg: e.to_string(),
            cause: None,
        }
    }
}

impl std::convert::From<String> for Error {
    fn from(e: String) -> Error {
        Error::Generic {
            msg: e,
            cause: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::AQLSyntaxError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            Error::AQLSemanticError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            Error::LoadingGraphFailed { name } => {
                write!(f, "Could not load graph {} from disk", &name)
            }
            Error::ImpossibleSearch(reason) => {
                write!(f, "Impossible search expression detected: {}", reason)
            }
            Error::NoSuchCorpus(name) => write!(f, "Corpus {} not found", &name),
            Error::Generic { msg, .. } => write!(f, "{}", msg),
            Error::IO(e) => e.fmt(f),
            Error::Bincode(e) => e.fmt(f),
            Error::CSV(e) => e.fmt(f),
            Error::ParseIntError(e) => e.fmt(f),
            Error::Fmt(e) => e.fmt(f),
            Error::Strum(e) => e.fmt(f),
            Error::Regex(e) => e.fmt(f),
            Error::RandomGenerator(e) => e.fmt(f),
            Error::Failure(e) => e.fmt(f),
            Error::SSTable(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::AQLSyntaxError { .. }
            | Error::AQLSemanticError { .. }
            | Error::LoadingGraphFailed { .. }
            | Error::ImpossibleSearch(_)
            | Error::NoSuchCorpus(_) => None,
            Error::Generic { cause, .. } => {
                if let Some(cause) = cause {
                    Some(cause.as_ref())
                } else {
                    None
                }
            }
            Error::Bincode(e) => Some(e),
            Error::IO(e) => Some(e),
            Error::CSV(e) => Some(e),
            Error::ParseIntError(e) => Some(e),
            Error::Fmt(e) => Some(e),
            Error::Strum(e) => Some(e),
            Error::Regex(e) => Some(e),
            Error::RandomGenerator(e) => Some(e),
            Error::Failure(e) => Some(e),
            Error::SSTable(e) => Some(e),
        }
    }
}
