use crate::annis::types::LineColumnRange;
use std::error::Error;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Debug)]
pub enum GraphAnnisError {
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

impl Display for GraphAnnisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GraphAnnisError::AQLSyntaxError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            GraphAnnisError::AQLSemanticError { desc, location } => write!(f, "{}", {
                if let Some(location) = location {
                    format!("[{}] {}", &location, desc)
                } else {
                    desc.clone()
                }
            }),
            GraphAnnisError::LoadingGraphFailed { name } => {
                write!(f, "Could not load graph {} from disk", &name)
            }
            GraphAnnisError::ImpossibleSearch(reason) => {
                write!(f, "Impossible search expression detected: {}", reason)
            }
            GraphAnnisError::NoSuchCorpus(name) => write!(f, "Corpus {} not found", &name),
        }
    }
}

impl Error for GraphAnnisError {}
