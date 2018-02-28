use graphstorage::{GraphStorage, GraphStatistic};
use super::adjacencylist::AdjacencyListStorage;
use super::prepost::PrePostOrderStorage;
use super::linear::LinearGraphStorage;
use std;
use std::sync::Arc;
use bincode;
use std::any::Any;
use std::str::FromStr;
use strum;

#[derive(Debug)]
pub enum RegistryError {
    Empty,
    ImplementationNameNotFound,
    TypeNotFound,
    Serialization(Box<bincode::ErrorKind>),
    Other,
}

#[derive(ToString, Debug, Clone, EnumString,PartialEq)]
pub enum ImplTypes {
    AdjacencyListV1,
    PrePostOrderO32L32V1,
    PrePostOrderO32L8V1,
    PrePostOrderO16L32V1,
    PrePostOrderO16L8V1,
    LinearO32V1,
    LinearO16V1,
    LinearO8V1,
}

impl From<Box<bincode::ErrorKind>> for RegistryError {
    fn from(e: Box<bincode::ErrorKind>) -> RegistryError {
        RegistryError::Serialization(e)
    }
}

impl From<strum::ParseError> for RegistryError {
    fn from(_: strum::ParseError) -> RegistryError {
        RegistryError::ImplementationNameNotFound
    }
}

type Result<T> = std::result::Result<T, RegistryError>;

pub fn create_writeable() -> AdjacencyListStorage {
    // TODO: make this configurable when there are more writeable graph storage implementations
    AdjacencyListStorage::new()
}

pub fn create_from_type(impl_type : ImplTypes) -> Arc<GraphStorage> {
    match impl_type {
        ImplTypes::AdjacencyListV1 => Arc::new(AdjacencyListStorage::new()),
        ImplTypes::PrePostOrderO32L32V1 => Arc::new(PrePostOrderStorage::<u32,u32>::new()),
        ImplTypes::PrePostOrderO32L8V1 => Arc::new(PrePostOrderStorage::<u32,u8>::new()),
        ImplTypes::PrePostOrderO16L32V1 => Arc::new(PrePostOrderStorage::<u16,u32>::new()),
        ImplTypes::PrePostOrderO16L8V1 => Arc::new(PrePostOrderStorage::<u16,u8>::new()),
        ImplTypes::LinearO32V1 => Arc::new(LinearGraphStorage::<u32>::new()),
        ImplTypes::LinearO16V1 => Arc::new(LinearGraphStorage::<u16>::new()),
        ImplTypes::LinearO8V1 => Arc::new(LinearGraphStorage::<u8>::new()),
    }
}

fn get_prepostorder_by_size(stats : &GraphStatistic) -> ImplTypes {
    if stats.rooted_tree {
        // There are exactly two order values per node and there can be only one order value per node
        // in a tree.
        if stats.nodes < (u16::max_value() / 2) as usize {
            if stats.max_depth < u8::max_value() as usize {
                return ImplTypes::PrePostOrderO16L8V1;
            } else {
                return ImplTypes::PrePostOrderO16L32V1;
            }
        } else if stats.nodes < (u32::max_value() / 2) as usize {
            if stats.max_depth < u8::max_value() as usize {
                return ImplTypes::PrePostOrderO32L8V1;
            } else {
                return ImplTypes::PrePostOrderO32L32V1;
            }
        }
    } else {
        if stats.max_depth < u8::max_value() as usize {
            return ImplTypes::PrePostOrderO32L8V1;
        }
    }
    return ImplTypes::PrePostOrderO32L32V1;
}

fn get_linear_by_size(stats : &GraphStatistic) -> ImplTypes {
    if stats.max_depth < u8::max_value() as usize {
        return ImplTypes::LinearO8V1;
    } else if stats.max_depth < u16::max_value() as usize {
        return ImplTypes::LinearO16V1;
    }else {
        return ImplTypes::LinearO32V1;
    }
}

pub fn get_optimal_impl_heuristic(stats : &GraphStatistic) -> ImplTypes {

    if stats.max_depth <= 1 {
        // if we don't have any deep graph structures an adjencency list is always fasted (and has no overhead)
        return ImplTypes::AdjacencyListV1;
    } else if stats.rooted_tree {
        if stats.max_fan_out <= 1 {
            return get_linear_by_size(stats);
        } else {
            return get_prepostorder_by_size(stats);
        }
    } else if !stats.cyclic {
        // it might be still wise to use pre/post order if the graph is "almost" a tree, thus
        // does not have many exceptions
        if stats.dfs_visit_ratio <= 1.03 {
            // there is no more than 3% overhead
            // TODO: how to determine the border?
            return get_prepostorder_by_size(stats);
        }
    }

    // fallback
    return ImplTypes::AdjacencyListV1;
}

pub fn deserialize(impl_name : &str, input : &mut std::io::Read) -> Result<Arc<GraphStorage>> {

    let impl_type = ImplTypes::from_str(impl_name)?;

    match impl_type {
        ImplTypes::AdjacencyListV1 => {
            let gs : AdjacencyListStorage =  bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::PrePostOrderO32L32V1 => {
            let gs : PrePostOrderStorage<u32,u32> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::PrePostOrderO32L8V1 => {
            let gs : PrePostOrderStorage<u32,u8> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::PrePostOrderO16L32V1 => {
            let gs : PrePostOrderStorage<u16,u32> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::PrePostOrderO16L8V1 => {
            let gs : PrePostOrderStorage<u16,u8> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::LinearO32V1 => {
            let gs : LinearGraphStorage<u32> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::LinearO16V1 => {
            let gs : LinearGraphStorage<u16> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
        ImplTypes::LinearO8V1 => {
            let gs : LinearGraphStorage<u8> = bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Arc::new(gs))
        },
    }
}

pub fn get_type(data : Arc<GraphStorage>) -> Result<ImplTypes> {
    let data :&Any = data.as_any();
    if let Some(_) = data.downcast_ref::<AdjacencyListStorage>() {
        return Ok(ImplTypes::AdjacencyListV1);
    } else if let Some(_) = data.downcast_ref::<PrePostOrderStorage<u32,u32>>() {
        return Ok(ImplTypes::PrePostOrderO32L32V1);
    } else if let Some(_) = data.downcast_ref::<PrePostOrderStorage<u32,u8>>() {
        return Ok(ImplTypes::PrePostOrderO32L8V1);
    } else if let Some(_) = data.downcast_ref::<PrePostOrderStorage<u16,u32>>() {
        return Ok(ImplTypes::PrePostOrderO16L32V1);
    } else if let Some(_) = data.downcast_ref::<PrePostOrderStorage<u16,u8>>() {
        return Ok(ImplTypes::PrePostOrderO16L8V1);
    } else if let Some(_) = data.downcast_ref::<LinearGraphStorage<u32>>() {
        return Ok(ImplTypes::LinearO32V1);
    } else if let Some(_) = data.downcast_ref::<LinearGraphStorage<u16>>() {
        return Ok(ImplTypes::LinearO16V1);
    } else if let Some(_) = data.downcast_ref::<LinearGraphStorage<u8>>() {
        return Ok(ImplTypes::LinearO8V1);
    }
    return Err(RegistryError::TypeNotFound);
}

pub fn serialize(data : Arc<GraphStorage>, writer : &mut std::io::Write) -> Result<String> {
    let data :&Any = data.as_any();
    if let Some(gs) = data.downcast_ref::<AdjacencyListStorage>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::AdjacencyListV1.to_string());
    } else if let Some(gs) = data.downcast_ref::<PrePostOrderStorage<u32,u32>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::PrePostOrderO32L32V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<PrePostOrderStorage<u32,u8>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::PrePostOrderO32L8V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<PrePostOrderStorage<u16,u32>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::PrePostOrderO16L32V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<PrePostOrderStorage<u16,u8>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::PrePostOrderO16L8V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<LinearGraphStorage<u32>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::LinearO32V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<LinearGraphStorage<u16>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::LinearO16V1.to_string());
    } else if let Some(gs) = data.downcast_ref::<LinearGraphStorage<u8>>() {
        bincode::serialize_into(writer, gs, bincode::Infinite)?;
        return Ok(ImplTypes::LinearO8V1.to_string());
    }
    return Err(RegistryError::TypeNotFound);
}


