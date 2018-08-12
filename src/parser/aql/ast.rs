use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub enum Factor {
    Literal(VecDeque<Literal>),
    Disjunction(Disjunction),
}

pub type Conjunction = VecDeque<Factor>;
pub type Disjunction = VecDeque<Conjunction>;

#[derive(Debug, Clone)]
pub struct Pos {
    pub start : usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub enum Literal {
    TokenSearch{val : TextSearch, pos : Option<Pos>},
    AnnoSearch{name : QName, val : Option<TextSearch>, pos: Option<Pos>},
    BinaryOp {lhs : Operand, op: BinaryOpSpec, rhs : Operand, pos : Option<Pos>},
}

#[derive(Debug, Clone)]
pub enum Operand {
    NodeRef(NodeRef),
    Literal(Rc<Literal>)
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

#[derive(Debug, Clone)]
pub enum BinaryOpSpec {
    Dominance,
    Pointing,
    Precedence,
    Overlap,
    IdenticalCoverage,
}

#[derive(Debug,Clone)]
pub enum NodeRef {
    ID(u32),
    Name(String),
}