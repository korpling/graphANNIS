use std::{fmt::Display, sync::PoisonError};

use crate::annis::types::LineColumnRange;
use graphannis_core::{
    errors::{ComponentTypeError, GraphAnnisCoreError},
    types::NodeID,
};
use thiserror::Error;

use super::db::relannis::TextProperty;

pub type Result<T> = std::result::Result<T, GraphAnnisError>;

#[derive(Error, Debug, strum_macros::IntoStaticStr)]
#[non_exhaustive]
pub enum GraphAnnisError {
    #[error(transparent)]
    Core(#[from] GraphAnnisCoreError),
    #[error("{0}")]
    AQLSyntaxError(AQLError),
    #[error("{0}")]
    AQLSemanticError(AQLError),
    #[error("impossible search expression detected: {0}")]
    ImpossibleSearch(String),
    #[error("timeout")]
    Timeout,
    #[error("could not load graph {name} from disk")]
    LoadingGraphFailed { name: String },
    #[error("corpus {0} not found")]
    NoSuchCorpus(String),
    #[error("corpus {0} already exists.")]
    CorpusExists(String),
    #[error("could not get internal node ID for node {0}")]
    NoSuchNodeID(String),
    #[error("could not covered token for node {0}")]
    NoCoveredTokenForNode(String),
    #[error("could not covered token for subgraph")]
    NoCoveredTokenForSubgraph,
    #[error("plan description missing")]
    PlanDescriptionMissing,
    #[error("plan cost missing")]
    PlanCostMissing,
    #[error("no execution node for component {0}")]
    NoExecutionNode(usize),
    #[error("no component for node #{0}")]
    NoComponentForNode(usize),
    #[error("LHS operand not found")]
    LHSOperandNotFound,
    #[error("RHS operand not found")]
    RHSOperandNotFound,
    #[error("Could not peek next element in index join: {0}")]
    PeekInIndexJoin(String),
    #[error(
        "frequency definition must consists of two parts: \
    the referenced node and the annotation name or \"tok\" separated by \":\""
    )]
    InvalidFrequencyDefinition,
    #[error(transparent)]
    CorpusStorage(#[from] CorpusStorageError),
    #[error(transparent)]
    RelAnnisImportError(#[from] RelAnnisError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDeserializer(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSerializer(#[from] toml::ser::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    StripPathPrefix(#[from] std::path::StripPrefixError),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Lock poisoning ({0})")]
    LockPoisoning(String),
    #[error(transparent)]
    BtreeIndex(#[from] transient_btree_index::Error),
    #[error("index out of bounds {0}")]
    IndexOutOfBounds(usize),
    #[error("The annotation graph that shall be queried is not fully loaded.")]
    QueriedGraphNotFullyLoaded,
}

impl<T> From<PoisonError<T>> for GraphAnnisError {
    fn from(e: PoisonError<T>) -> Self {
        Self::LockPoisoning(e.to_string())
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CorpusStorageError {
    #[error("listing directories from {path} failed")]
    ListingDirectories {
        path: String,
        source: std::io::Error,
    },
    #[error("could not get directory entry for {path}")]
    DirectoryEntry {
        path: String,
        source: std::io::Error,
    },
    #[error("could not determine file type for {path}")]
    FileTypeDetection {
        path: String,
        source: std::io::Error,
    },
    #[error("loading corpus-config.toml for corpus {corpus} failed")]
    LoadingCorpusConfig {
        corpus: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("could not create corpus with name {corpus}")]
    CreateCorpus {
        corpus: String,
        source: GraphAnnisCoreError,
    },
    #[error("this format can only export one corpus but {0} corpora have been given as argument")]
    MultipleCorporaForSingleCorpusFormat(usize),
    #[error("error when removing existing files for corpus {corpus}")]
    RemoveFileForCorpus {
        corpus: String,
        source: std::io::Error,
    },
    #[error("could not lock corpus directory {path}")]
    LockCorpusDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("the corpus cache entry is not loaded")]
    CorpusCacheEntryNotLoaded,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RelAnnisError {
    #[error("directory {0} not found")]
    DirectoryNotFound(String),
    #[error("missing column at position {pos} ({name}) in file {file}")]
    MissingColumn {
        pos: usize,
        name: String,
        file: String,
    },
    #[error("unexpected value NULL in column {pos} ({name}) in file {file} at line {}", line.map_or("<unkown>".to_string(), |l| l.to_string()))]
    UnexpectedNull {
        pos: usize,
        name: String,
        file: String,
        line: Option<u64>,
    },
    #[error("toplevel corpus not found")]
    ToplevelCorpusNotFound,
    #[error("corpus with ID {0} not found")]
    CorpusNotFound(u32),
    #[error("node with ID {0} not found")]
    NodeNotFound(NodeID),
    #[error("can't get find any aligned token (left or right) for node with ID {0}")]
    AlignedNotFound(NodeID),
    #[error("can't get left aligned token for node with ID {0}")]
    LeftAlignedNotFound(NodeID),
    #[error("can't get right aligned token for node with ID {0}")]
    RightAlignedNotFound(NodeID),
    #[error("can't get token ID for position {0:?}")]
    NoTokenForPosition(TextProperty),
    #[error("can't get left position of node {0}")]
    NoLeftPositionForNode(NodeID),
    #[error("can't get right position of node {0}")]
    NoRightPositionForNode(NodeID),
    #[error("invalid component type short name '{0}'")]
    InvalidComponentShortName(String),
}

#[derive(Debug, Serialize, Clone)]
pub struct AQLError {
    pub desc: String,
    pub location: Option<LineColumnRange>,
}

impl Display for AQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.location {
            write!(f, "[{}] {}", location, self.desc)
        } else {
            write!(f, "{}", self.desc)
        }
    }
}

impl From<GraphAnnisError> for ComponentTypeError {
    fn from(e: GraphAnnisError) -> Self {
        ComponentTypeError(Box::new(e))
    }
}

#[macro_export]
macro_rules! try_as_boxed_iter {
    ($x:expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                return std::boxed::Box::new(std::iter::once(Err(e.into())));
            }
        }
    };
}

#[macro_export]
macro_rules! try_as_option {
    ($x:expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                return Some(Err(e.into()));
            }
        }
    };
}
