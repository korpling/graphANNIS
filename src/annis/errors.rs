use std::error::Error;
use std::fmt::Display;
use crate::annis::errors_legacy;


pub type Result<T> = std::result::Result<T, AnnisError>;

#[derive(Debug)]
pub enum AnnisError {
    LoadingGraphFailed { name: String },
    Generic(String),
    Legacy(errors_legacy::Error),
    IO(std::io::Error),
    Bincode(::bincode::Error),
    CSV(::csv::Error),
    ParseIntError(::std::num::ParseIntError),
    Fmt(::std::fmt::Error),
    Strum(::strum::ParseError),
    Regex(::regex::Error),
}

impl AnnisError {
    pub fn kind(&self) -> &str {
        match self {
            AnnisError::LoadingGraphFailed {..} => "LoadingGraphFailed",
            AnnisError::Legacy(e) => e.kind().description(),
            AnnisError::Generic(_) => "Generic",
            AnnisError::IO(_) => "IO",
            AnnisError::Bincode(_) => "Bincode",
            AnnisError::CSV(_) => "CSV",
            AnnisError::ParseIntError(_) => "ParseIntError",
            AnnisError::Fmt(_) => "Fmt",
            AnnisError::Strum(_) => "Strum",
            AnnisError::Regex(_) => "Regex",
        }
    }
}

impl std::convert::From<errors_legacy::ErrorKind> for AnnisError {
    fn from(e : errors_legacy::ErrorKind) -> AnnisError {
        AnnisError::Legacy(e.into())
    }
}

impl std::convert::From<errors_legacy::Error> for AnnisError {
    fn from(e : errors_legacy::Error) -> AnnisError {
        AnnisError::Legacy(e)
    }
}

impl std::convert::From<std::io::Error> for AnnisError {
    fn from(e : std::io::Error) -> AnnisError {
        AnnisError::IO(e)
    }
}

impl std::convert::From<::bincode::Error> for AnnisError {
    fn from(e : ::bincode::Error) -> AnnisError {
        AnnisError::Bincode(e)
    }
}

impl std::convert::From<::csv::Error> for AnnisError {
    fn from(e : ::csv::Error) -> AnnisError {
        AnnisError::CSV(e)
    }
}

impl std::convert::From<std::num::ParseIntError> for AnnisError {
    fn from(e : std::num::ParseIntError) -> AnnisError {
        AnnisError::ParseIntError(e)
    }
}

impl std::convert::From<std::fmt::Error> for AnnisError {
    fn from(e : std::fmt::Error) -> AnnisError {
        AnnisError::Fmt(e)
    }
}

impl std::convert::From<strum::ParseError> for AnnisError {
    fn from(e : strum::ParseError) -> AnnisError {
        AnnisError::Strum(e)
    }
}

impl std::convert::From<regex::Error> for AnnisError {
    fn from(e : regex::Error) -> AnnisError {
        AnnisError::Regex(e)
    }
}


impl std::convert::From<&str> for AnnisError {
    fn from(e : &str) -> AnnisError {
        AnnisError::Generic(e.to_string())
    }
}

impl std::convert::From<String> for AnnisError {
    fn from(e : String) -> AnnisError {
        AnnisError::Generic(e)
    }
}

impl Display for AnnisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnnisError::LoadingGraphFailed { name } => {
                write!(f, "Could not load graph {} from disk", &name)
            },
            AnnisError::Legacy(e) => {
                e.fmt(f)
            },
            AnnisError::Generic(e) => {
                write!(f, "{}", e)
            }
            AnnisError::IO(e) => {
                e.fmt(f)
            },
            AnnisError::Bincode(e) => {
                e.fmt(f)
            },
            AnnisError::CSV(e) => {
                e.fmt(f)
            }
            AnnisError::ParseIntError(e) => {
                e.fmt(f)
            },
            AnnisError::Fmt(e) => {
                e.fmt(f)
            },
            AnnisError::Strum(e) => {
                e.fmt(f)
            },
            AnnisError::Regex(e) => {
                e.fmt(f)
            }
        }
    }
}

impl Error for AnnisError {
    fn source(&self) -> Option<&(Error + 'static)> {
        match self {
            AnnisError::LoadingGraphFailed { .. } | AnnisError::Generic(_) => None,
            AnnisError::Legacy(e) => e.source(),
            AnnisError::Bincode(e) => Some(e),
            AnnisError::IO(e) => Some(e),
            AnnisError::CSV(e) => Some(e),
            AnnisError::ParseIntError(e) => Some(e),
            AnnisError::Fmt(e) => Some(e),
            AnnisError::Strum(e) => Some(e),
            AnnisError::Regex(e) => Some(e),
        }
    }
}
