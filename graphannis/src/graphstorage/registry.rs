use graphstorage::{GraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use super::prepost::PrePostOrderStorage;
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

#[derive(ToString, EnumString)]
pub enum ImplTypes {
    AdjacencyListV1,
    PrePostOrderO32L32V1,
    PrePostOrderO32L8V1,
    PrePostOrderO16L32V1,
    PrePostOrderO16L8V1,
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
        }
    }
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
    }
    return Err(RegistryError::TypeNotFound);
}


