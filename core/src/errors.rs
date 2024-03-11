use std::{
    array::TryFromSliceError, num::TryFromIntError, string::FromUtf8Error, sync::PoisonError,
};

use thiserror::Error;

use crate::types::AnnoKey;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GraphAnnisCoreError {
    #[error("invalid component type {0}")]
    InvalidComponentType(String),
    #[error("invalid format for component description, expected ctype/layer/name, but got {0}")]
    InvalidComponentDescriptionFormat(String),
    #[error("could not load annotation storage from file {path}: {source}")]
    LoadingAnnotationStorage {
        path: String,
        source: std::io::Error,
    },
    #[error("could not find implementation for graph storage with name '{0}'")]
    UnknownGraphStorageImpl(String),
    #[error("can't load component with empty path")]
    EmptyComponentPath,
    #[error("could not find annotation key ID for {0:?} when mapping to GraphML")]
    GraphMLMissingAnnotationKey(AnnoKey),
    #[error("could not get mutable reference for component {0}")]
    NonExclusiveComponentReference(String),
    #[error("component {0} is missing")]
    MissingComponent(String),
    #[error("component {0} was not loaded")]
    ComponentNotLoaded(String),
    #[error("component {0} is read-only")]
    ReadOnlyComponent(String),
    #[error(transparent)]
    ModelError(#[from] ComponentTypeError),
    #[error(transparent)]
    BincodeSerialization(#[from] bincode::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PersistingTemporaryFile(#[from] tempfile::PersistError),
    #[error(transparent)]
    SortedStringTable(#[from] sstable::error::Status),
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),
    #[error("Cache error: {0}")]
    LfuCache(String),
    #[error("File to persist graph updates is missing.")]
    GraphUpdatePersistanceFileMissing,
    #[error(transparent)]
    BtreeIndex(#[from] transient_btree_index::Error),
    #[error(transparent)]
    IntConversion(#[from] TryFromIntError),
    #[error(transparent)]
    SliceConversion(#[from] TryFromSliceError),
    #[error("Lock poisoning ({0})")]
    LockPoisoning(String),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Too man unique items added to symbol table")]
    SymbolTableOverflow,
    #[error("Annotation key with ID {0} is not in symbol table")]
    UnknownAnnoKeySymbolId(usize),
    #[error("The choose cache size is zero, which is not allowed.")]
    ZeroCacheSize,
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<T> From<PoisonError<T>> for GraphAnnisCoreError {
    fn from(e: PoisonError<T>) -> Self {
        Self::LockPoisoning(e.to_string())
    }
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct ComponentTypeError(pub Box<dyn std::error::Error + Send + Sync>);

impl From<GraphAnnisCoreError> for ComponentTypeError {
    fn from(e: GraphAnnisCoreError) -> Self {
        ComponentTypeError(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, GraphAnnisCoreError>;

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
