use crate::annis::types::{Component, Edge};

use crate::annis::errors::*;

use shardio::ShardWriter;

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
    tmp_dir: tempfile::TempDir,
    writer_components: ShardWriter<ComponentEntry>,
    writer_edges: ShardWriter<EdgeEntry>,
}

impl LoadRankResultBuilder {
    pub fn new() -> Result<LoadRankResultBuilder> {
        let tmp_dir = tempfile::tempdir()?;

        let mut writer_components: ShardWriter<ComponentEntry> =
            ShardWriter::new(&tmp_dir.path().join("components.idx"), 64, 256, 1 << 16)?;

        let mut writer_edges: ShardWriter<EdgeEntry> =
            ShardWriter::new(&tmp_dir.path().join("edges.idx"), 64, 256, 1 << 16)?;

        Ok(LoadRankResultBuilder {
            tmp_dir,
            writer_components,
            writer_edges,
        })
    }

    pub fn add_edge(&mut self, pre: u32, component: Component, edge: Edge) -> Result<()> {
        self.writer_components
            .get_sender()
            .send(ComponentEntry { pre, component })?;
        self.writer_edges
            .get_sender()
            .send(EdgeEntry { pre, edge })?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<LoadRankResult> {
        self.writer_components.finish()?;
        self.writer_edges.finish()?;
        unimplemented!()
    }
}

pub struct LoadRankResult {}

impl LoadRankResult {
    pub fn new() -> Result<LoadRankResult> {
        unimplemented!()
    }

    pub fn component_by_pre(&self, pre: u32) -> Result<Component> {
        unimplemented!()
    }

    pub fn edge_by_pre(&self, pre: u32) -> Result<Edge> {
        unimplemented!()
    }

    pub fn is_text_coverage(&self, edge: &Edge) -> Result<bool> {
        unimplemented!()
    }
}
