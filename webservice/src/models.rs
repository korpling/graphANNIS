use crate::schema::{corpus_groups, groups};

#[derive(Queryable, Insertable)]
pub struct CorpusGroup {
    pub group: String,
    pub corpus: String,
}

#[derive(Insertable)]
pub struct Group {
    pub name: String,
}
