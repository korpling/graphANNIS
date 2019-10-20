//! This is a graph-based linguistic corpus query system which implements the ANNIS Query Language (AQL).
//! The main entry point to the API is the [CorpusStorage](struct.CorpusStorage.html) struct which allows to manage and query a database of corpora.

// workaround for doc.rs bug that uses nightly compiler and complains that the global allocator is not stabilized yet
#![cfg_attr(docs_rs_workaround, feature(global_allocator, allocator_api))]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate graphannis_malloc_size_of as malloc_size_of;
#[macro_use]
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;
#[macro_use]
extern crate log;

#[macro_use]
extern crate percent_encoding;

extern crate boolean_expression;
extern crate linked_hash_map;
extern crate multimap;
extern crate rand;
extern crate regex;
extern crate regex_syntax;
extern crate rustc_hash;
extern crate tempfile;

extern crate bincode;
extern crate serde;

extern crate csv;

extern crate strum;
#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate fs2;
extern crate itertools;
#[macro_use]
extern crate lalrpop_util;
extern crate num;
extern crate rayon;
extern crate sys_info;

extern crate libc;
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

pub use crate::annis::db::update;

pub use crate::annis::db::Graph;
pub use crate::annis::db::NODE_TYPE_KEY;

/// Types that are used by the `Graph` API.
pub mod graph {
    pub use crate::annis::db::graphstorage::GraphStatistic;
    pub use crate::annis::db::graphstorage::{EdgeContainer, GraphStorage, WriteableGraphStorage};
    pub use crate::annis::db::AnnotationStorage;
    pub use crate::annis::db::Match;
    pub use crate::annis::types::{AnnoKey, Annotation, Component, ComponentType, Edge, NodeID};
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
