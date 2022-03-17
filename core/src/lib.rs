#![warn(clippy::panic)]
#![warn(clippy::expect_used)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate graphannis_malloc_size_of as malloc_size_of;
#[macro_use]
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;

pub mod annostorage;
pub mod dfs;
pub mod errors;
pub mod graph;
pub mod serializer;
pub mod types;
pub mod util;
