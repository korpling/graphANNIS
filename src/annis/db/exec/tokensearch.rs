use annis::db::exec::Desc;
use annis::db::exec::ExecutionNode;
use annis::db::Graph;
use annis::db::Match;
use std::fmt;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps the search for *all* token in a corpus.
pub struct TokenSearch<'a> {
    desc: Option<Desc>,
    db: &'a Graph,
}

impl<'a> TokenSearch<'a> {
    pub fn new(db : &'a Graph) -> TokenSearch<'a> {

        TokenSearch {
            db,
            desc: None,
        }
    }
}

impl<'a> fmt::Display for TokenSearch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tok")
    }
}

impl<'a> ExecutionNode for TokenSearch<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for TokenSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        unimplemented!()
    }
}
