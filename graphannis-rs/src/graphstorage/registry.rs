use graphstorage::{GraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std;
use std::rc::Rc;
use std::any::Any;
use bincode;

#[derive(Debug)]
pub enum RegistryError {
    Empty,
    ImplementationNameNotFound,
    TypeNotFound,
    Serialization(Box<bincode::ErrorKind>),
    Other,
}

impl From<Box<bincode::ErrorKind>> for RegistryError {
    fn from(e: Box<bincode::ErrorKind>) -> RegistryError {
        RegistryError::Serialization(e)
    }
}

type Result<T> = std::result::Result<T, RegistryError>;

pub fn create_writeable() -> AdjacencyListStorage {
    // TODO: make this configurable when there are more writeable graph storage implementations
    AdjacencyListStorage::new()
}

pub fn load_by_name(impl_name : &str, input : &mut std::io::Read) -> Result<Rc<GraphStorage>> {

    match impl_name {
        "AdjacencyListStorage" => {
            let gs : AdjacencyListStorage =  bincode::deserialize_from(input, bincode::Infinite)?;
            Ok(Rc::new(gs))
        },
        _ => Err(RegistryError::ImplementationNameNotFound)
    }
}


