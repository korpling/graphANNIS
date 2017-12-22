use plan::{ExecutionNode, Desc};
use {Match};
use std;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it : Box<Iterator<Item=Vec<Match>> + 'a>,

    desc: Option<Desc>,
}

impl<'a> NodeSearch<'a> {   
    pub fn new (it : Box<Iterator<Item = Match> + 'a>, desc : Option<Desc>) -> NodeSearch<'a> {
        // map result of iterator to vector in order to be compatible to
        // the other execution plan operations

        NodeSearch {
            it : Box::from(it.map(|n| vec![n])),
            desc,
        }
    }
}

impl<'a> ExecutionNode for NodeSearch<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item=Vec<Match>> {self}

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}

impl<'a> Iterator for NodeSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}