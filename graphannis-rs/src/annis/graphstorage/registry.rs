use std::collections::BTreeMap;
use std::string::String;
use super::{WriteableGraphStorage, ReadableGraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std;

#[derive(Debug)]
pub enum RegistryError {
    Empty,
    ImplementationNameNotFound,
}

type Result<T> = std::result::Result<T, RegistryError>;

pub struct GraphStorageRegistry {
    readable_constructors : BTreeMap<String, fn(&ReadableGraphStorage) -> Box<ReadableGraphStorage>>,
    writable_constructors : BTreeMap<String, fn() -> Box<WriteableGraphStorage>>,
}

impl GraphStorageRegistry {

    pub fn new() -> GraphStorageRegistry {
        let mut readable_constructors = BTreeMap::new();
        let mut writable_constructors = BTreeMap::<String, fn() -> Box<WriteableGraphStorage>>::new();

        writable_constructors.insert(String::from("AdjacencyListStorage"), || Box::new(AdjacencyListStorage::new()));
        // TODO: add other graph storages

        GraphStorageRegistry{readable_constructors, writable_constructors}
    }

    pub fn create_writable(&self) -> Result<Box<WriteableGraphStorage>> {
        // just use the first one available
        let mut it = self.writable_constructors.iter();
        let constructor = try!(it.next().ok_or(RegistryError::Empty));

        Ok(constructor.1())
    }
}