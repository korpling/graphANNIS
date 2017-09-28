pub type NodeID = u32;
pub type StringID = u32;



#[derive(Serialize, Deserialize, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct AnnoKey {
    pub name: StringID,
    pub ns: StringID,
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct Annotation {
    pub key: AnnoKey,
    pub val: StringID,
}

#[derive(Debug, Default)]
pub struct Match {
    pub node: NodeID,
    pub anno: Annotation,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct Edge {
    pub source: NodeID,
    pub target: NodeID,
}

impl Edge {
    pub fn inverse(&self) -> Edge {
        Edge {source: self.target, target: self.source}
    }
}

#[macro_use]
pub mod util;

pub mod dfs;
pub mod annostorage;
pub mod stringstorage;
pub mod graphstorage;
pub mod graphdb;
pub mod operator;
