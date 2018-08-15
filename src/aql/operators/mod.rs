fn format_range(min_dist: usize, max_dist: usize) -> String {
    if min_dist == 1 && max_dist == usize::max_value() {
        String::from("*")
    } else if min_dist == 1 && max_dist == 1 {
        String::from("")
    } else {
        format!("{},{}", min_dist, max_dist)
    }
}

pub mod precedence;
pub mod edge_op;
pub mod identical_cov;
pub mod inclusion;
pub mod overlap;
pub mod identical_node;

pub use self::precedence::PrecedenceSpec;
pub use self::edge_op::{DominanceSpec, PartOfSubCorpusSpec, PointingSpec};
pub use self::inclusion::InclusionSpec;
pub use self::overlap::OverlapSpec;
pub use self::identical_cov::IdenticalCoverageSpec;
pub use self::identical_node::IdenticalNodeSpec;
