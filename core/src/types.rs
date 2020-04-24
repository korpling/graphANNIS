use num_traits::{Bounded, FromPrimitive, Num, ToPrimitive};
use std;
use std::fmt;
use std::ops::AddAssign;
use std::string::String;

use std::borrow::Cow;
use std::convert::TryInto;
use strum_macros::{EnumIter, EnumString};

use super::serializer::{FixedSizeKeySerializer, KeySerializer};
use malloc_size_of::MallocSizeOf;

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

impl KeySerializer for Edge {
    fn create_key<'a>(&'a self) -> Cow<'a, [u8]> {
        let mut result = Vec::with_capacity(std::mem::size_of::<NodeID>() * 2);
        result.extend(&self.source.to_be_bytes());
        result.extend(&self.target.to_be_bytes());
        Cow::Owned(result)
    }

    fn parse_key(key: &[u8]) -> Self {
        let id_size = std::mem::size_of::<NodeID>();

        let source = NodeID::from_be_bytes(
            key[..id_size]
                .try_into()
                .expect("Edge deserialization key was too small"),
        );
        let target = NodeID::from_be_bytes(
            key[id_size..]
                .try_into()
                .expect("Edge deserialization key has wrong size"),
        );
        Edge { source, target }
    }
}

impl FixedSizeKeySerializer for Edge {
    fn key_size() -> usize {
        std::mem::size_of::<NodeID>() * 2
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
pub enum AQLComponentType {
    /// Edges between a span node and its tokens. Implies text coverage.
    Coverage,
    /// Edges between a structural node and any other structural node, span or token. Implies text coverage.
    Dominance = 2,
    /// Edge between any node.
    Pointing,
    /// Edge between two tokens implying that the source node comes before the target node in the textflow.
    Ordering,
    /// Explicit edge between any non-token node and the left-most token it covers.
    LeftToken,
    /// Explicit edge between any non-token node and the right-most token it covers.
    RightToken,
    /// Implies that the source node belongs to the parent corpus/subcorpus/document/datasource node.
    PartOf,
}

impl fmt::Display for AQLComponentType {
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
    pub ctype: AQLComponentType,
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
