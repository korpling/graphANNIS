use super::{WriteableGraphStorage, ReadableGraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std::sync::Arc;
use std;
use bincode;
use graphdb::ImplType;

#[derive(Debug)]
pub enum RegistryError {
    Empty,
    ImplementationNameNotFound,
    Serialization(Box<bincode::ErrorKind>),
    Other,
}

impl From<Box<bincode::ErrorKind>> for RegistryError {
    fn from(e: Box<bincode::ErrorKind>) -> RegistryError {
        RegistryError::Serialization(e)
    }
}

type Result<T> = std::result::Result<T, RegistryError>;

pub fn create_writeable() -> Box<WriteableGraphStorage> {
    // TODO: make this configurable when there are more writeable graph storage implementations
    Box::new(AdjacencyListStorage::new())
}

pub fn load_by_name(impl_name : &str, input : &mut std::io::Read) -> Result<ImplType> {

    match impl_name {
        "AdjacencyListStorage" => {
            let gs : AdjacencyListStorage =  bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(ImplType::Writable(Box::new(gs)))
        },
        _ => Err(RegistryError::ImplementationNameNotFound)
    }
}