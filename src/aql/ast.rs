use std::collections::VecDeque;
use std::rc::Rc;

use exec::nodesearch::NodeSearchSpec;
use std::ops::Range;
use aql::operators::{
    OverlapSpec, 
    IdenticalCoverageSpec,
    PrecedenceSpec,
    DominanceSpec,
    PointingSpec,
    PartOfSubCorpusSpec,
};

pub type Pos = Range<usize>;

#[derive(Debug)]
pub enum Factor {
    Literal(Literal),
    Disjunction(Disjunction),
}

pub type Conjunction = VecDeque<Factor>;
pub type Disjunction = VecDeque<Conjunction>;


#[derive(Debug, Clone)]
pub enum Literal {
    NodeSearch{spec: NodeSearchSpec, pos : Option<Pos>, variable: Option<String>},
    BinaryOp {lhs : Operand, op: BinaryOpSpec, rhs : Operand, pos : Option<Pos>},
    LegacyMetaSearch{spec: NodeSearchSpec, pos : Pos}
}

#[derive(Debug, Clone)]
pub enum Operand {
    NodeRef(NodeRef),
    Literal{spec: Rc<NodeSearchSpec>, pos : Pos, variable : Option<String> }
}


#[derive(Debug, Clone)]
pub struct TextSearch(pub String, pub StringMatchType);   

#[derive(Debug, Clone)]
pub struct QName (pub Option<String>, pub String);


#[derive(Debug, Clone)]
pub enum StringMatchType {
    Exact,
    Regex,
}

#[derive(Debug,Clone)]
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
}

#[derive(Debug, Clone)]
pub struct RangeSpec { 
    pub min_dist: usize, 
    pub max_dist: usize,
}