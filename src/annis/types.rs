use num::{Bounded, FromPrimitive, Num, ToPrimitive};
use std;
use std::fmt;
use std::ops::AddAssign;
use std::string::String;

use crate::malloc_size_of::MallocSizeOf;

/// Unique internal identifier for a single node.
pub type NodeID = u64;

/// The fully qualified name of an annotation.
#[derive(
    Serialize,
    Deserialize,
    Default,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Clone,
    Debug,
    MallocSizeOf,
    Hash,
)]
pub struct AnnoKey {
    /// Name of the annotation.
    pub name: String,
    /// Namespace of the annotation.
    pub ns: String,
}

/// An annotation with a qualified name and a value.
#[derive(Serialize, Deserialize, Default, Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct Annotation {
    /// Qualified name or unique "key" for the annotation
    pub key: AnnoKey,
    /// Value of the annotation
    pub val: String,
}

pub type AnnoKeyID = usize;

/// A struct that contains the extended results of the count query.
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct CountExtra {
    /// Total number of matches.
    pub match_count: u64,
    /// Number of documents with at least one match.
    pub document_count: u64,
}

/// Directed edge between a source and target node which are identified by their ID.
#[derive(
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Clone,
    Debug,
    Hash,
    MallocSizeOf,
    Default,
)]
#[repr(C)]
pub struct Edge {
    pub source: NodeID,
    pub target: NodeID,
}

impl Edge {
    pub fn inverse(&self) -> Edge {
        Edge {
            source: self.target,
            target: self.source,
        }
    }
}

/// Specifies the type of component. Types determine certain semantics about the edges of this graph components.
#[derive(
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Debug,
    EnumIter,
    EnumString,
    MallocSizeOf,
)]
#[repr(C)]
pub enum ComponentType {
    /// Edges between a span node and its tokens. Implies text coverage.
    Coverage,
    /// Edges between a token and a span node.
    InverseCoverage,
    /// Edges between a structural node and any other structural node, span or token. Implies text coverage.
    Dominance,
    /// Edge between any node.
    Pointing,
    /// Edge between two tokens implying that the source node comes before the target node in the textflow.
    Ordering,
    /// Explicit edge between any non-token node and the left-most token it covers.
    LeftToken,
    /// Explicit edge between any non-token node and the right-most token it covers.
    RightToken,
    /// Implies that the source node belongs to the parent corpus/subcorpus/document node.
    PartOfSubcorpus,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Identifies an edge component of the graph.
#[derive(
    Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, MallocSizeOf,
)]
pub struct Component {
    /// Type of the component
    pub ctype: ComponentType,
    /// Name of the component
    pub name: String,
    /// A layer name which allows to group different components into the same layer. Can be empty.
    pub layer: String,
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}/{}/{}", self.ctype, self.layer, self.name)
    }
}

pub trait NumValue:
    Send + Sync + Ord + Num + AddAssign + Clone + Bounded + FromPrimitive + ToPrimitive + MallocSizeOf
{
}

impl NumValue for u64 {}
impl NumValue for u32 {}
impl NumValue for u16 {}
impl NumValue for u8 {}

/// Definition of the result of a `frequency` query.
///
/// This is a vector of rows, and each row is a vector of columns with the different
/// attribute values and a number of matches having this combination of attribute values.
pub type FrequencyTable<T> = Vec<(Vec<T>, usize)>;

/// Description of an attribute of a query.
pub struct QueryAttributeDescription {
    /// ID of the alternative this attribute is part of.
    pub alternative: usize,
    /// Textual representation of the query fragment for this attribute.
    pub query_fragment: String,
    // Variable name of this attribute.
    pub variable: String,
    // Optional annotation name represented by this attribute.
    pub anno_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
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
