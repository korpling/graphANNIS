use boolean_expression;
use std;
use std::rc::Rc;

use crate::annis::db::aql::operators::{
    DominanceSpec, IdenticalCoverageSpec, IdenticalNodeSpec, InclusionSpec, OverlapSpec,
    PartOfSubCorpusSpec, PointingSpec, PrecedenceSpec, LeftAlignmentSpec,
};
use crate::annis::db::exec::nodesearch::NodeSearchSpec;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
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

pub type Expr = boolean_expression::Expr<Literal>;

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
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

pub enum ComparisionOperator {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub struct QName(pub Option<String>, pub String);

#[derive(Debug, Clone)]
pub enum StringMatchType {
    Exact,
    Regex,
}

#[derive(Debug, Clone, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub enum NodeRef {
    ID(usize),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BinaryOpSpec {
    Dominance(DominanceSpec),
    Pointing(PointingSpec),
    Precedence(PrecedenceSpec),
    Overlap(OverlapSpec),
    IdenticalCoverage(IdenticalCoverageSpec),
    PartOfSubCorpus(PartOfSubCorpusSpec),
    Inclusion(InclusionSpec),
    LeftAlignment(LeftAlignmentSpec),
    IdenticalNode(IdenticalNodeSpec),
}

pub use crate::annis::db::aql::operators::RangeSpec;
