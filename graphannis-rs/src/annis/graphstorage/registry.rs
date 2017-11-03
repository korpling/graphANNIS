use std::collections::BTreeMap;
use std::string::String;
use super::{WriteableGraphStorage, ReadableGraphStorage};
use super::adjacencylist::AdjacencyListStorage;
use std;
use bincode;

#[derive(Debug)]
pub enum RegistryError {
    Empty,
    ImplementationNameNotFound,
}

type Result<T> = std::result::Result<T, RegistryError>;

enum Type {
    Readable{
        copy_constructor : fn(&ReadableGraphStorage) -> Box<ReadableGraphStorage>,
        deserializer : fn(&mut std::io::Read) -> Box<ReadableGraphStorage>
    },
    Writable{
        copy_constructor : fn(&ReadableGraphStorage) -> Box<WriteableGraphStorage>, 
        empty_constructor : fn() -> Box<WriteableGraphStorage>,
        deserializer : fn(&mut std::io::Read) -> Box<ReadableGraphStorage>,
    },
}

pub struct GraphStorageRegistry {

    types : BTreeMap<String, Type>,
}

impl GraphStorageRegistry {

    pub fn new() -> GraphStorageRegistry {
        let mut types = BTreeMap::new();

        types.insert(String::from("AdjacencyListStorage"), Type::Writable{
            copy_constructor : |orig : &ReadableGraphStorage| Box::new(AdjacencyListStorage::new()),
            empty_constructor : || Box::new(AdjacencyListStorage::new()),
            deserializer : |input : &mut std::io::Read| {
                let loaded = 
                    bincode::deserialize_from(input, bincode::Infinite)
                    .unwrap_or(Box::new(AdjacencyListStorage::new()));
                return loaded;
            },
        });
        // TODO: add other graph storages

        GraphStorageRegistry{types}
    }

    pub fn create_writable(&self) -> Result<Box<WriteableGraphStorage>> {
        for (_, value) in self.types.iter() {
            // just use the first writable graph storage available
            if let Type::Writable{copy_constructor, empty_constructor, deserializer} = *value {
                return Ok(empty_constructor());
            }
        }
        Err(RegistryError::Empty)
    }

    pub fn load_by_name(&self, impl_name : &str, input : &mut std::io::Read) -> Result<Box<ReadableGraphStorage>> {
        let t = try!(self.types.get(impl_name).ok_or(RegistryError::ImplementationNameNotFound));

        let gs = match *t {
            Type::Readable{copy_constructor, deserializer} => {
                deserializer(input)
            },
            Type::Writable{copy_constructor, empty_constructor, deserializer} => {
                deserializer(input)
            }
        };

        Ok(gs)
    }
}