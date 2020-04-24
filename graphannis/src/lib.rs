//! This is a graph-based linguistic corpus query system which implements the ANNIS Query Language (AQL).
//! The main entry point to the API is the [CorpusStorage](struct.CorpusStorage.html) struct which allows to manage and query a database of corpora.

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate graphannis_malloc_size_of as malloc_size_of;
#[macro_use]
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;
#[macro_use]
extern crate log;

#[macro_use]
extern crate percent_encoding;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate lalrpop_util;

#[cfg(feature = "c-api")]
extern crate simplelog;
#[cfg(feature = "c-api")]
pub mod capi;

// Make sure the allocator is always the one from the system, otherwise we can't make sure our memory estimations work
use std::alloc::System;
#[global_allocator]
static GLOBAL: System = System;

mod annis;

pub use crate::annis::db::corpusstorage::CorpusStorage;

/// Types that are used by the `CorpusStorage` API.
pub mod corpusstorage {
    pub use crate::annis::db::corpusstorage::{
        CacheStrategy, CorpusInfo, FrequencyDefEntry, GraphStorageInfo, ImportFormat, LoadStatus,
        QueryLanguage, ResultOrder,
    };
    pub use crate::annis::types::{CountExtra, FrequencyTable, QueryAttributeDescription};
}

use annis::db::aql::model::AQLComponentType;
pub use graphannis_core::graph::update;

pub type Graph = graphannis_core::graph::Graph<AQLComponentType>;

/// Types that are used by the `Graph` API.
pub mod graph {
    use crate::annis::db::aql::model::AQLComponentType;
    pub use graphannis_core::annostorage::AnnotationStorage;
    pub use graphannis_core::annostorage::Match;
    pub use graphannis_core::graph::storage::GraphStatistic;
    pub use graphannis_core::graph::storage::{EdgeContainer, GraphStorage, WriteableGraphStorage};
    pub use graphannis_core::types::{AnnoKey, Annotation, Edge, NodeID};
    pub type Component = graphannis_core::types::Component<AQLComponentType>;
}

/// Contains the graphANNIS-specific error types.
pub mod errors {
    pub use crate::annis::errors::*;
}

/// Utility functions.
pub mod util {
    pub use crate::annis::util::get_queries_from_csv;
    pub use crate::annis::util::node_names_from_match;
    pub use crate::annis::util::SearchDef;
}
