#[derive(Debug)]
pub enum Expr {
    TokenSearch(TextSearch),
    AnnoSearch(QName, Option<TextSearch>),
    BinaryOp(String, BinaryOpType, String),
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
pub enum BinaryOpType {
    Dominance,
    Pointing,
    Precedence,
    Overlap,
    IdenticalCoverage,
}
