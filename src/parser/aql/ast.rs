use std::collections::VecDeque;

#[derive(Debug)]
pub enum Expr {
    Conjunction(Conjunction),
    Disjunction(Disjunction),
    Empty,
}

pub type Conjunction = VecDeque<Term>;
pub type Disjunction = VecDeque<Term>;

#[derive(Debug)]
pub enum Term {
    TokenSearch(TextSearch),
    AnnoSearch(QName, Option<TextSearch>),
    BinaryOp(NodeRef, BinaryOpSpec, NodeRef),
}


#[derive(Debug)]
pub struct TextSearch(pub String, pub StringMatchType);   

#[derive(Debug)]
pub struct QName (pub Option<String>, pub String);


#[derive(Debug)]
pub enum StringMatchType {
    Exact,
    Regex,
}

#[derive(Debug)]
pub enum BinaryOpSpec {
    Dominance,
    Pointing,
    Precedence,
    Overlap,
    IdenticalCoverage,
}

#[derive(Debug)]
pub enum NodeRef {
    ID(u32),
    Name(String),
}