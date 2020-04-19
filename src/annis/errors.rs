use crate::annis::types::LineColumnRange;
use serde::{de, ser};
use std::error::Error as StdError;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, AnnisError>;

#[derive(Debug)]
pub enum AnnisError {
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
    SSTable(sstable::error::Status),
}

impl std::convert::From<std::io::Error> for AnnisError {
    fn from(e: std::io::Error) -> AnnisError {
        AnnisError::IO(e)
    }
}

impl std::convert::From<tempfile::PersistError> for AnnisError {
    fn from(e: tempfile::PersistError) -> AnnisError {
        AnnisError::IO(e.error)
    }
}

impl std::convert::From<::bincode::Error> for AnnisError {
    fn from(e: ::bincode::Error) -> AnnisError {
        AnnisError::Bincode(e)
    }
}

impl std::convert::Into<::bincode::Error> for AnnisError {
    fn into(self) -> ::bincode::Error {
        Box::new(::bincode::ErrorKind::Custom(format!("{}", &self)))
    }
}

impl ser::Error for AnnisError {
    fn custom<T: Display>(msg: T) -> Self {
        AnnisError::Generic {
            msg: msg.to_string(),
            cause: None,
        }
    }
}

impl de::Error for AnnisError {
    fn custom<T: Display>(msg: T) -> Self {
        AnnisError::Generic {
            msg: msg.to_string(),
            cause: None,
        }
    }
}

impl std::convert::From<::csv::Error> for AnnisError {
    fn from(e: ::csv::Error) -> AnnisError {
        AnnisError::CSV(e)
    }
}

impl std::convert::From<std::num::ParseIntError> for AnnisError {
    fn from(e: std::num::ParseIntError) -> AnnisError {
        AnnisError::ParseIntError(e)
    }
}

impl std::convert::From<std::fmt::Error> for AnnisError {
    fn from(e: std::fmt::Error) -> AnnisError {
        AnnisError::Fmt(e)
    }
}

impl std::convert::From<strum::ParseError> for AnnisError {
    fn from(e: strum::ParseError) -> AnnisError {
        AnnisError::Strum(e)
    }
}

impl std::convert::From<regex::Error> for AnnisError {
    fn from(e: regex::Error) -> AnnisError {
        AnnisError::Regex(e)
    }
}

impl std::convert::From<::rand::Error> for AnnisError {
    fn from(e: ::rand::Error) -> AnnisError {
        AnnisError::RandomGenerator(e)
    }
}

impl std::convert::From<sstable::error::Status> for AnnisError {
    fn from(e: sstable::error::Status) -> AnnisError {
        AnnisError::SSTable(e)
    }
}

impl std::convert::From<&str> for AnnisError {
    fn from(e: &str) -> AnnisError {
        AnnisError::Generic {
            msg: e.to_string(),
            cause: None,
        }
    }
}

impl std::convert::From<String> for AnnisError {
    fn from(e: String) -> AnnisError {
        AnnisError::Generic {
            msg: e,
            cause: None,
        }
    }
}

impl Display for AnnisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnnisError::AQLSyntaxError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            AnnisError::AQLSemanticError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            AnnisError::LoadingGraphFailed { name } => {
                write!(f, "Could not load graph {} from disk", &name)
            }
            AnnisError::ImpossibleSearch(reason) => {
                write!(f, "Impossible search expression detected: {}", reason)
            }
            AnnisError::NoSuchCorpus(name) => write!(f, "Corpus {} not found", &name),
            AnnisError::Generic { msg, .. } => write!(f, "{}", msg),
            AnnisError::IO(e) => e.fmt(f),
            AnnisError::Bincode(e) => e.fmt(f),
            AnnisError::CSV(e) => e.fmt(f),
            AnnisError::ParseIntError(e) => e.fmt(f),
            AnnisError::Fmt(e) => e.fmt(f),
            AnnisError::Strum(e) => e.fmt(f),
            AnnisError::Regex(e) => e.fmt(f),
            AnnisError::RandomGenerator(e) => e.fmt(f),
            AnnisError::SSTable(e) => e.fmt(f),
        }
    }
}

impl StdError for AnnisError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AnnisError::AQLSyntaxError { .. }
            | AnnisError::AQLSemanticError { .. }
            | AnnisError::LoadingGraphFailed { .. }
            | AnnisError::ImpossibleSearch(_)
            | AnnisError::NoSuchCorpus(_) => None,
            AnnisError::Generic { cause, .. } => {
                if let Some(cause) = cause {
                    Some(cause.as_ref())
                } else {
                    None
                }
            }
            AnnisError::Bincode(e) => Some(e),
            AnnisError::IO(e) => Some(e),
            AnnisError::CSV(e) => Some(e),
            AnnisError::ParseIntError(e) => Some(e),
            AnnisError::Fmt(e) => Some(e),
            AnnisError::Strum(e) => Some(e),
            AnnisError::Regex(e) => Some(e),
            AnnisError::RandomGenerator(e) => Some(e),
            AnnisError::SSTable(e) => Some(e),
        }
    }
}
