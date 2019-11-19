use crate::annis::db::corpusstorage::PreparationResult;
use crate::annis::db::corpusstorage::QueryLanguage;
use crate::annis::db::corpusstorage::ResultOrder;
use crate::annis::db::Match;
use std::collections::BTreeMap;

pub struct FindIterator<'a> {
    preps_by_corpusname: BTreeMap<String, PreparationResult<'a>>,
    query: String,
    query_language: QueryLanguage,
    offset: usize,
    limit: usize,
    order: ResultOrder,
}

impl<'a> FindIterator<'a> {
    pub fn new(
        preps_by_corpusname: BTreeMap<String, PreparationResult<'a>>,
        query: &str,
        query_language: QueryLanguage,
        offset: usize,
        limit: usize,
        order: ResultOrder,
    ) -> FindIterator<'a> {
        FindIterator {
            preps_by_corpusname,
            query: query.to_string(),
            query_language,
            offset,
            limit,
            order,
        }
    }
}

impl<'a> std::iter::Iterator for FindIterator<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> std::option::Option<Vec<Match>> {
        None
    }
}
