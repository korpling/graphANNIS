use std::collections::BTreeMap;
use std::string::String;
use super::{WriteableGraphStorage, ReadableGraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std;
use bincode;
use annis;

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

pub fn create_writable_copy(orig : &ReadableGraphStorage) -> Box<WriteableGraphStorage> {
    let mut gs  = create_writeable();

    gs.copy(orig);
    return gs;
}

pub fn load_by_name(impl_name : &str, input : &mut std::io::Read) -> Result<annis::graphdb::ImplType> {

    match impl_name {
        "AdjacencyListStorage" => {
            let gs : AdjacencyListStorage =  bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(annis::graphdb::ImplType::Writable(Box::new(gs)))
        },
        _ => Err(RegistryError::ImplementationNameNotFound)
    }
}