#[derive(Clone, Default, Debug)]
pub struct Config {
    pub use_parallel_joins: bool,
}

pub mod conjunction;
pub mod disjunction;
