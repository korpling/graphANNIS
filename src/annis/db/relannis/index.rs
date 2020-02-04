use crate::annis::errors::*;
use crate::annis::types::{Component, ComponentType, Edge, NodeID};
use crate::annis::util::disk_collections::{DiskMap, DiskMapBuilder};

use std::ops::Bound::Included;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
struct ComponentEntry {
    pre: u32,
    component: Component,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
struct EdgeEntry {
    pre: u32,
    edge: Edge,
}

pub struct LoadRankResultBuilder {
    components_by_pre: DiskMapBuilder<u32, Component>,
    edges_by_pre: DiskMapBuilder<u32, Edge>,
    text_coverage_edges: DiskMapBuilder<Edge, bool>,
}

impl LoadRankResultBuilder {
    pub fn new() -> Result<LoadRankResultBuilder> {
        let components_by_pre = DiskMapBuilder::new()?;
        let edges_by_pre = DiskMapBuilder::new()?;
        let text_coverage_edges = DiskMapBuilder::new()?;

        Ok(LoadRankResultBuilder {
            components_by_pre,
            edges_by_pre,
            text_coverage_edges,
        })
    }

    pub fn add_edge(&mut self, pre: u32, component: Component, edge: Edge) -> Result<()> {
        if component.ctype == ComponentType::Coverage {
            self.text_coverage_edges.insert(edge.clone(), true)?;
        }

        self.components_by_pre.insert(pre, component)?;
        self.edges_by_pre.insert(pre, edge)?;
        Ok(())
    }

    pub fn finish(self) -> Result<LoadRankResult> {
        let components_by_pre = self.components_by_pre.finish()?;
        let edges_by_pre = self.edges_by_pre.finish()?;
        let text_coverage_edges = self.text_coverage_edges.finish()?;
        Ok(LoadRankResult {
            components_by_pre,
            edges_by_pre,
            text_coverage_edges,
        })
    }
}

pub struct LoadRankResult {
    components_by_pre: DiskMap<u32, Component>,
    edges_by_pre: DiskMap<u32, Edge>,
    text_coverage_edges: DiskMap<Edge, bool>,
}

impl LoadRankResult {
    pub fn get_component_by_pre(&self, pre: u32) -> Result<Option<Component>> {
        self.components_by_pre.get(&pre)
    }

    pub fn get_edge_by_pre(&self, pre: u32) -> Result<Option<Edge>> {
        self.edges_by_pre.get(&pre)
    }

    pub fn is_text_coverage(&self, edge: &Edge) -> Result<bool> {
        self.text_coverage_edges.get(edge).map(|e| e.is_some())
    }

    pub fn has_outgoing_text_coverage_edge(&self, n: NodeID) -> Result<bool> {
        let nodes_with_same_source = (
            Included(Edge {
                source: n,
                target: NodeID::min_value(),
            }),
            Included(Edge {
                source: n,
                target: NodeID::max_value(),
            }),
        );
        let mut it = self.text_coverage_edges.range(nodes_with_same_source)?;
        Ok(it.next().is_some())
    }
}
