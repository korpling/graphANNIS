use super::EdgeContainer;
use crate::types::NodeID;
use rustc_hash::FxHashSet;

#[derive(MallocSizeOf)]
pub struct UnionEdgeContainer<'a> {
    containers: Vec<&'a dyn EdgeContainer>,
}

impl<'a> UnionEdgeContainer<'a> {
    pub fn new(containers: Vec<&'a dyn EdgeContainer>) -> UnionEdgeContainer<'a> {
        UnionEdgeContainer { containers }
    }
}

impl<'a> EdgeContainer for UnionEdgeContainer<'a> {
    fn get_outgoing_edges<'b>(&'b self, node: NodeID) -> Box<dyn Iterator<Item = NodeID> + 'b> {
        let mut targets = FxHashSet::default();
        for c in self.containers.iter() {
            targets.extend(c.get_outgoing_edges(node));
        }
        Box::from(targets.into_iter())
    }

    fn get_ingoing_edges<'b>(&'b self, node: NodeID) -> Box<dyn Iterator<Item = NodeID> + 'b> {
        let mut sources = FxHashSet::default();
        for c in self.containers.iter() {
            sources.extend(c.get_ingoing_edges(node));
        }
        Box::from(sources.into_iter())
    }

    fn source_nodes<'b>(&'b self) -> Box<dyn Iterator<Item = NodeID> + 'b> {
        let mut sources = FxHashSet::default();
        for c in self.containers.iter() {
            sources.extend(c.source_nodes());
        }
        Box::from(sources.into_iter())
    }
}
