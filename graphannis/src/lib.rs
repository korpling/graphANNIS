//! This is a graph-based linguistic corpus query system which implements the ANNIS Query Language (AQL).
//! The main entry point to the API is the [CorpusStorage](struct.CorpusStorage.html) struct which allows to manage and query a database of corpora.

#![deny(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::expect_used,
    clippy::exit,
    clippy::todo,
    clippy::unwrap_in_result
)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate lalrpop_util;

mod annis;

pub use crate::annis::db::corpusstorage::CorpusStorage;

/// Types that are used by the `CorpusStorage` API.
pub mod corpusstorage {
    pub use crate::annis::db::corpusstorage::SearchQuery;
    pub use crate::annis::db::corpusstorage::{
        CacheStrategy, CorpusInfo, ExportFormat, FrequencyDefEntry, GraphStorageInfo, ImportFormat,
        LoadStatus, QueryLanguage, ResultOrder,
    };
    pub use crate::annis::types::{
        CountExtra, FrequencyTable, FrequencyTableRow, QueryAttributeDescription,
    };
}

pub use graphannis_core::graph::update;

pub use graphannis_core::graph::Graph;

/// A specialization of the [`Graph`], using components needed to represent and query corpus annotation graphs.
pub type AnnotationGraph =
    graphannis_core::graph::Graph<annis::db::aql::model::AnnotationComponentType>;

/// Types that are used by the `Graph` API.
pub mod graph {
    pub use graphannis_core::annostorage::AnnotationStorage;
    pub use graphannis_core::annostorage::Match;
    pub use graphannis_core::annostorage::MatchGroup;
    pub use graphannis_core::graph::storage::GraphStatistic;
    pub use graphannis_core::graph::storage::{EdgeContainer, GraphStorage, WriteableGraphStorage};
    pub use graphannis_core::types::{AnnoKey, Annotation, Component, Edge, NodeID};
}

/// Types that define the annotation graph model.
pub mod model {
    pub use crate::annis::db::aql::model::AnnotationComponentType;
    pub type AnnotationComponent =
        graphannis_core::types::Component<crate::model::AnnotationComponentType>;
}

/// Helper functions to execute AQL queries on an [`AnnotationGraph`].
pub mod aql {
    pub use crate::annis::db::aql::disjunction::Disjunction;
    pub use crate::annis::db::aql::execute_query_on_graph;
    pub use crate::annis::db::aql::parse;
}

/// Contains the graphANNIS-specific error types.
pub mod errors {
    pub use crate::annis::errors::*;
}

/// Utility functions.
pub mod util {
    pub use crate::annis::util::SearchDef;
    pub use crate::annis::util::get_queries_from_csv;
    pub use crate::annis::util::node_names_from_match;
}
