use crate::annis::types::LineColumnRange;
use std::error::Error;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

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
        }
    }
}

impl Error for AnnisError {}
