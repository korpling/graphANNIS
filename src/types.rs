use num::{Num,FromPrimitive, Bounded, ToPrimitive};
use std::string::String;
use std::fmt;
use std::ops::AddAssign;
use std;

use malloc_size_of::MallocSizeOf;

pub type NodeID = u32;
pub type StringID = u32;



#[derive(Serialize, Deserialize, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Debug, MallocSizeOf, Hash)]
#[repr(C)]
pub struct AnnoKey {
    pub name: StringID,
    pub ns: StringID,
}

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Debug, MallocSizeOf, Hash)]
#[repr(C)]
pub struct Annotation {
    pub key: AnnoKey,
    pub val: StringID,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Match {
    pub node: NodeID,
    pub anno: Annotation,
}

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct CountExtra {
    pub match_count: u64,
    pub document_count: u64,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Hash, MallocSizeOf)]
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

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, EnumIter, EnumString, MallocSizeOf)]
#[repr(C)]
pub enum ComponentType {
    Coverage,
    InverseCoverage,
    Dominance,
    Pointing,
    Ordering,
    LeftToken,
    RightToken,
    PartOfSubcorpus,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, MallocSizeOf)]
pub struct Component {
    pub ctype : ComponentType,
    pub name : String,
    pub layer : String,
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}/{}/{}", self.ctype, self.layer, self.name)
    }
}

pub trait NumValue : Send + Sync + Ord + Num + AddAssign + Clone + Bounded + FromPrimitive + ToPrimitive + MallocSizeOf {

}

impl NumValue for u64 {}
impl NumValue for u32 {}
impl NumValue for u16 {}
impl NumValue for u8 {}

/// Very simple definition of a matrix from a single data type. Not optimized at all.
/// TODO: Maybe a sparse matrix could be used.
pub type Matrix<T> = Vec<Vec<T>>;

pub type FrequencyTable<T> = Vec<(Vec<T>, usize)>;

pub struct NodeDesc {
    pub component_nr: usize,
    pub aql_fragment : String,
    pub variable: String,
    pub anno_name : Option<String>,
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct LineColumn {
    pub line : usize,
    pub column : usize,
}

impl std::fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct LineColumnRange {
    pub start : LineColumn,
    pub end : Option<LineColumn>,
}

impl std::fmt::Display for LineColumnRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(end) = self.end.clone() {
            if self.start == end {
                write!(f, "{}", self.start)
            } else {
                write!(f, "{}-{}", self.start, end)
            }
        } else {
            write!(f, "{}", self.start)
        }
    }
}