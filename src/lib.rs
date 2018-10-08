// workaround for doc.rs bug that uses nightly compiler and complains that the global allocator is not stabilized yet
#![cfg_attr(docs_rs_workaround, feature(global_allocator))]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate graphannis_malloc_size_of as malloc_size_of;
#[macro_use]
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;
#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

extern crate linked_hash_map;
extern crate multimap;
extern crate rand;
extern crate regex;
extern crate regex_syntax;
extern crate rustc_hash;
extern crate tempdir;

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

#[cfg(feature = "c-api")]
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

pub use annis::api::corpusstorage::{
    CorpusInfo, CorpusStorage, FrequencyDefEntry, LoadStatus, ResultOrder,
};
pub use annis::api::update;
pub use annis::graphdb::GraphDB;
pub use annis::graphstorage::GraphStorage;
pub use annis::types::{
    Annotation, Component, ComponentType, CountExtra, Edge, FrequencyTable, Match, Matrix,
    NodeDesc, NodeID,
};

pub mod relannis {
    pub use annis::relannis::load;
}

pub mod errors {
    pub use annis::errors::*;
}

pub mod util {
    pub use annis::util::get_queries_from_folder;
    pub use annis::util::SearchDef;
}
