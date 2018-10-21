use std;
use std::collections::VecDeque;
use std::rc::Rc;

use annis::db::aql::operators::{
    DominanceSpec, IdenticalCoverageSpec, IdenticalNodeSpec, InclusionSpec, OverlapSpec,
    PartOfSubCorpusSpec, PointingSpec, PrecedenceSpec,
};
use annis::db::exec::nodesearch::NodeSearchSpec;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub start: usize,
    pub end: usize,
}

impl From<Pos> for std::ops::Range<usize> {
    fn from(p: Pos) -> Self {
        p.start..p.end
    }
}

impl From<std::ops::Range<usize>> for Pos {
    fn from(p: std::ops::Range<usize>) -> Self {
        Pos {
            start: p.start,
            end: p.end,
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy))]
#[derive(Debug)]
pub enum Factor {
    Literal(Literal),
    Disjunction(Disjunction),
}

pub type Conjunction = VecDeque<Factor>;
pub type Disjunction = VecDeque<Conjunction>;

#[derive(Debug, Clone)]
pub enum Literal {
    NodeSearch {
        spec: NodeSearchSpec,
        pos: Option<Pos>,
        variable: Option<String>,
    },
    BinaryOp {
        lhs: Operand,
        op: BinaryOpSpec,
        rhs: Operand,
        pos: Option<Pos>,
    },
    LegacyMetaSearch {
        spec: NodeSearchSpec,
        pos: Pos,
    },
}

#[derive(Debug, Clone)]
pub enum Operand {
    NodeRef(NodeRef),
    Literal {
        spec: Rc<NodeSearchSpec>,
        pos: Pos,
        variable: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct TextSearch(pub String, pub StringMatchType);

#[derive(Debug, Clone)]
pub struct QName(pub Option<String>, pub String);

#[derive(Debug, Clone)]
pub enum StringMatchType {
    Exact,
    Regex,
}

#[derive(Debug, Clone)]
pub enum NodeRef {
    ID(usize),
    Name(String),
}

#[derive(Debug, Clone)]
pub enum BinaryOpSpec {
    Dominance(DominanceSpec),
    Pointing(PointingSpec),
    Precedence(PrecedenceSpec),
    Overlap(OverlapSpec),
    IdenticalCoverage(IdenticalCoverageSpec),
    PartOfSubCorpus(PartOfSubCorpusSpec),
    Inclusion(InclusionSpec),
    IdenticalNode(IdenticalNodeSpec),
}

#[derive(Debug, Clone)]
pub struct RangeSpec {
    pub min_dist: usize,
    pub max_dist: usize,
}
