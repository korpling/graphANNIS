pub type NodeID = u32;
pub type StringID = u32;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct AnnoKey {
    pub name: StringID,
    pub ns: StringID,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct Annotation {
    pub key: AnnoKey,
    pub val: StringID,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
#[repr(C)]
pub struct Edge {
    pub source: NodeID,
    pub target: NodeID,
}

pub mod annostorage;
pub mod stringstorage;
pub mod util;
