use crate::annis::types::{Component, Edge};
use crate::annis::util::disk_collections::{DiskMap, DiskMapBuilder};
use crate::annis::errors::*;

use shardio::{ShardReader, ShardWriter};

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
}

impl LoadRankResultBuilder {
    pub fn new() -> Result<LoadRankResultBuilder> {

        let components_by_pre = DiskMapBuilder::new()?;
        let edges_by_pre = DiskMapBuilder::new()?;

        Ok(LoadRankResultBuilder {
            components_by_pre,
            edges_by_pre,
        })
    }

    pub fn add_edge(&mut self, pre: u32, component: Component, edge: Edge) -> Result<()> {
        self.components_by_pre.insert(pre, component)?;
        self.edges_by_pre.insert(pre, edge)?;
        Ok(())
    }

    pub fn finish(self) -> Result<LoadRankResult> {
        let components_by_pre = self.components_by_pre.finish()?;
        let edges_by_pre = self.edges_by_pre.finish()?;
        Ok(LoadRankResult {
            components_by_pre,
            edges_by_pre,
        })
    }
}

pub struct LoadRankResult {
    components_by_pre: DiskMap<u32, Component>,
    edges_by_pre: DiskMap<u32, Edge>,
}

impl LoadRankResult {
    pub fn get_component_by_pre(&self, pre: u32) -> Result<Option<Component>> {
        self.components_by_pre.get(&pre)
    }

    pub fn get_edge_by_pre(&self, pre: u32) -> Result<Option<Edge>> {
        self.edges_by_pre.get(&pre)
    }

    pub fn is_text_coverage(&self, edge: &Edge) -> Result<bool> {
        unimplemented!()
    }
}
