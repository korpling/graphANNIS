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

pub mod annostorage;
pub mod dfs;
pub mod errors;
pub mod graph;
pub mod serializer;
pub mod types;
pub mod util;
