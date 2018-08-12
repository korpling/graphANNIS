//use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Expr {
    TokenSearch(TextSearch),
    AnnoSearch(QName, Option<TextSearch>),
    BinaryOp(String, BinaryOpSpec, String),
    Conjunction(Vec<Box<Expr>>),
    Disjunction(Vec<Box<Expr>>),
    Empty,
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