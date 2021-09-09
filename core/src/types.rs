use num_traits::{Bounded, FromPrimitive, Num, ToPrimitive};
use smartstring::alias::String;
use std::fmt;
use std::ops::AddAssign;

use std::borrow::Cow;
use std::{convert::TryInto, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use super::serializer::{FixedSizeKeySerializer, KeySerializer};
use crate::{
    errors::{ComponentTypeError, GraphAnnisCoreError},
    graph::{update::UpdateEvent, Graph},
};
use fmt::Debug;
use malloc_size_of::MallocSizeOf;
use std::result::Result as StdResult;

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
    fn create_key(&self) -> Cow<[u8]> {
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

pub trait ComponentType:
    Into<u16> + From<u16> + FromStr + ToString + Send + Sync + Clone + Debug + Ord
{
    type UpdateGraphIndex;

    fn init_update_graph_index(
        _graph: &Graph<Self>,
    ) -> StdResult<Self::UpdateGraphIndex, ComponentTypeError>;

    fn before_update_event(
        _update: &UpdateEvent,
        _graph: &Graph<Self>,
        _index: &mut Self::UpdateGraphIndex,
    ) -> StdResult<(), ComponentTypeError> {
        Ok(())
    }
    fn after_update_event(
        _update: UpdateEvent,
        _graph: &Graph<Self>,
        _index: &mut Self::UpdateGraphIndex,
    ) -> StdResult<(), ComponentTypeError> {
        Ok(())
    }
    fn apply_update_graph_index(
        _index: Self::UpdateGraphIndex,
        _graph: &mut Graph<Self>,
    ) -> StdResult<(), ComponentTypeError> {
        Ok(())
    }

    fn all_component_types() -> Vec<Self>;

    fn default_components() -> Vec<Component<Self>> {
        Vec::default()
    }

    fn update_graph_index_components(_graph: &Graph<Self>) -> Vec<Component<Self>> {
        Vec::default()
    }
}

/// A simplified implementation of a `ComponentType` that only has one type of edges.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, EnumString, EnumIter, Debug)]
pub enum DefaultComponentType {
    Edge,
}

#[allow(clippy::from_over_into)]
impl Into<u16> for DefaultComponentType {
    fn into(self) -> u16 {
        0
    }
}

impl From<u16> for DefaultComponentType {
    fn from(_: u16) -> Self {
        DefaultComponentType::Edge
    }
}

impl fmt::Display for DefaultComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct DefaultGraphIndex;

impl ComponentType for DefaultComponentType {
    type UpdateGraphIndex = DefaultGraphIndex;

    fn init_update_graph_index(
        _graph: &Graph<Self>,
    ) -> StdResult<Self::UpdateGraphIndex, ComponentTypeError> {
        Ok(DefaultGraphIndex {})
    }
    fn all_component_types() -> Vec<Self> {
        DefaultComponentType::iter().collect()
    }
}

/// Identifies an edge component of the graph.
#[derive(
    Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, MallocSizeOf,
)]
pub struct Component<CT: ComponentType> {
    /// Type of the component
    ctype: u16,
    /// Name of the component
    pub name: String,
    /// A layer name which allows to group different components into the same layer. Can be empty.
    pub layer: String,

    phantom: std::marker::PhantomData<CT>,
}

impl<CT: ComponentType> Component<CT> {
    pub fn new(ctype: CT, layer: String, name: String) -> Component<CT> {
        Component {
            ctype: ctype.into(),
            name,
            layer,
            phantom: std::marker::PhantomData::<CT>::default(),
        }
    }

    /// Get type of the component
    pub fn get_type(&self) -> CT {
        self.ctype.into()
    }

    /// Set type of the component
    pub fn set_type(&mut self, ctype: CT) {
        self.ctype = ctype.into();
    }
}

impl<CT: ComponentType> std::fmt::Display for Component<CT> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ctype: CT = self.ctype.into();
        write!(f, "{:?}/{}/{}", ctype, self.layer, self.name)
    }
}

impl<CT: ComponentType> std::str::FromStr for Component<CT> {
    type Err = GraphAnnisCoreError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let splitted: Vec<_> = s.splitn(3, '/').collect();
        if splitted.len() == 3 {
            if let Ok(ctype) = CT::from_str(splitted[0]) {
                let result = Component {
                    ctype: ctype.into(),
                    layer: splitted[1].into(),
                    name: splitted[2].into(),
                    phantom: std::marker::PhantomData::<CT>::default(),
                };
                Ok(result)
            } else {
                Err(GraphAnnisCoreError::InvalidComponentType(
                    splitted[0].to_string(),
                ))
            }
        } else {
            Err(GraphAnnisCoreError::InvalidComponentDescriptionFormat(
                s.to_string(),
            ))
        }
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
