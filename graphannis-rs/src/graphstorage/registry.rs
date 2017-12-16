use graphstorage::{GraphStorage, WriteableGraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std;
use std::rc::Rc;
use bincode;

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

pub fn create_writeable() -> Box<GraphStorage> {
    // TODO: make this configurable when there are more writeable graph storage implementations
    Box::new(AdjacencyListStorage::new())
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