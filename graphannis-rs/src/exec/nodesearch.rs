use super::{Desc, ExecutionNode, NodeSearchDesc};
use Match;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<Iterator<Item = Vec<Match>> + 'a>,

    desc: Option<Desc>,
    node_search_desc: Option<NodeSearchDesc<'a>>,
}

impl<'a> NodeSearch<'a> {
    pub fn new(
        it: Box<Iterator<Item = Match> + 'a>,
        node_search_desc: Option<NodeSearchDesc<'a>>,
        desc: Option<Desc>,
    ) -> NodeSearch<'a> {
        // map result of iterator to vector in order to be compatible to
        // the other execution plan operations

        NodeSearch {
            it: Box::from(it.map(|n| vec![n])),
            desc,
            node_search_desc,
        }
    }
}

impl<'a> ExecutionNode for NodeSearch<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }

    fn get_node_search_desc(&self) -> Option<&NodeSearchDesc> {
        self.node_search_desc.as_ref()
    }
}

impl<'a> Iterator for NodeSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}
