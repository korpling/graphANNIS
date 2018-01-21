use {Match, Component};
use graphdb::GraphDB;

pub trait Operator {
    fn retrieve_matches<'a>(&'a self, lhs : &Match) -> Box<Iterator<Item = Match> + 'a>;

    fn filter_match(&self, lhs : &Match, rhs : &Match) -> bool;
}

pub trait OperatorSpec {
    fn necessary_components(&self) -> Vec<Component>;

    fn create_operator<'a>(&self, db: &'a GraphDB) -> Option<Box<Operator + 'a>>;
    
}

pub mod precedence;
pub mod edge_op;
pub mod identical_cov;
pub mod inclusion;
pub mod overlap;

pub use self::precedence::PrecedenceSpec;
pub use self::edge_op::{DominanceSpec, PointingSpec};
pub use self::inclusion::InclusionSpec;
pub use self::overlap::OverlapSpec;
pub use self::identical_cov::IdenticalCoverageSpec;