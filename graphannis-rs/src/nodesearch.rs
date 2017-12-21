use plan::ExecutionNode;
use {Match};

/// An [ExecutionNode](../plan/trait.ExecutionNode.html) which wraps base node (annotation) searches.
pub struct NodeSearch {
    /// The actual search implementation
    it : Box<Iterator<Item=Vec<Match>>>,
}

impl ExecutionNode for NodeSearch {
    fn as_iter(&mut self) -> &mut Iterator<Item=Vec<Match>> {self}
}

impl Iterator for NodeSearch {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}