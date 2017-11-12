use annis::stringstorage::StringStorage;
use annis::annostorage::AnnoStorage;
use annis::graphstorage::{WriteableGraphStorage, ReadableGraphStorage};
use annis::{Component, NodeID, StringID};
use annis::AnnoKey;
use annis::graphstorage::registry;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::boxed::Box;
use std::io::prelude::*;
use std;


const ANNIS_NS: &str = "annis";
const NODE_NAME: &str = "node_name";
const TOK: &str = "tok";
const NODE_TYPE: &str = "node_type";

pub enum ImplType {
    Readable(Box<ReadableGraphStorage>),
    Writable(Box<WriteableGraphStorage>),
}

#[derive(Debug)]
pub enum Error {
    IOerror(std::io::Error),
    RegistryError(registry::RegistryError),
    LocationEmpty,
    Other,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOerror(e)
    }
}

impl From<registry::RegistryError> for Error {
    fn from(e: registry::RegistryError) -> Error {
        Error::RegistryError(e)
    }
}


pub struct GraphDB {
    pub strings: StringStorage,
    pub node_annos: AnnoStorage<NodeID>,

    location: Option<PathBuf>,

    components: BTreeMap<Component, Option<ImplType>>,
    id_annis_ns: StringID,
    id_node_name: StringID,
    id_tok: StringID,
    id_node_type: StringID,
}

fn load_component_from_disk(c: Component, component_path: Option<PathBuf> ) -> Result<ImplType, Error> {
    let cpath = try!(component_path.ok_or(Error::LocationEmpty));
    
    // load component into memory
    let mut impl_path = PathBuf::from(&cpath);
    impl_path.push("impl.cfg");
    let mut f_impl = std::fs::File::open(impl_path)?;
    let mut impl_name = String::new();
    f_impl.read_to_string(&mut impl_name)?;

    let mut data_path = PathBuf::from(&cpath);
    data_path.push("data");
    let f_data = std::fs::File::open(data_path)?;
    let mut buf_reader = std::io::BufReader::new(f_data);
    let gs = registry::load_by_name(&impl_name, &mut buf_reader)?;

    return Ok(gs);
}

impl GraphDB {
    /**
     * Create a new and empty instance.
     */
    pub fn new() -> GraphDB {
        let mut strings = StringStorage::new();

        GraphDB {
            id_annis_ns: strings.add(ANNIS_NS),
            id_node_name: strings.add(NODE_NAME),
            id_tok: strings.add(TOK),
            id_node_type: strings.add(NODE_TYPE),

            strings,
            node_annos: AnnoStorage::<NodeID>::new(),
            components: BTreeMap::new(),

            location: None,
        }
    }

    fn component_path(&self, c: &Component) -> Option<PathBuf> {
        match self.location {
            Some(ref loc) => {
                let mut p = PathBuf::from(loc);
                p.push("gs");
                p.push(c.ctype.to_string());
                p.push(&c.layer);
                p.push(&c.name);
                Some(p)
            }
            None => None,
        }
    }

    /// Helper function to unwrap an Option<ImplType> by loading it from disk if necessary.
    fn unwrap_or_load(&self, entry : Option<ImplType>, c : &Component) -> Result<ImplType, Error> {
        // check if component is not loaded yet
        if entry.is_none() {
            let loaded : ImplType = load_component_from_disk(c.clone(), self.component_path(c))?;
            return Ok(loaded);
        } else {
            return Ok(entry.unwrap());
        }
    }

    fn insert_or_copy_writeable(&mut self, c : &Component) ->Result<(), Error> {
        // move the old entry into the ownership of this function
        let entry = self.components.remove(c);
        // component exists?
        if entry.is_some() {
            let loaded_comp = self.unwrap_or_load(entry.unwrap(), c)?;
            // copy to writable implementation if needed
            let loaded_comp = match loaded_comp {
                ImplType::Readable(gs_orig) => {
                    let mut gs_copy = registry::create_writeable();
                    gs_copy.copy(gs_orig.as_ref());
                    gs_copy 
                },
                ImplType::Writable(gs) => {
                    gs
                }
            };
            // (re-)insert the component into map again
            self.components.insert(c.clone(), Some(ImplType::Writable(loaded_comp)));
        }
        return Ok(());
    }

    pub fn get_or_create_writable(&mut self, c : &Component) -> Result<&mut WriteableGraphStorage, Error> {
        
        if self.components.contains_key(c) {
            // make sure the component is actually writable and loaded
            self.insert_or_copy_writeable(c)?;
        } else {
            self.components.insert(c.clone(), Some(ImplType::Writable(registry::create_writeable())));
        }
        
        // get and return the reference to the entry
        let entry : &mut Option<ImplType> = self.components.get_mut(c).ok_or(Error::Other)?;
        if entry.is_some() {
            let impl_type : &mut ImplType = entry.as_mut().unwrap();
            if let &mut ImplType::Writable(ref mut gs) = impl_type {
                return Ok(gs.as_mut());
            }
        }
        return Err(Error::Other);
    }

    

    pub fn get_token_key(&self) -> AnnoKey {
        AnnoKey {
            ns: self.id_annis_ns,
            name: self.id_tok,
        }
    }

    pub fn get_node_name_key(&self) -> AnnoKey {
        AnnoKey {
            ns: self.id_annis_ns,
            name: self.id_node_name,
        }
    }

    pub fn get_node_type_key(&self) -> AnnoKey {
        AnnoKey {
            ns: self.id_annis_ns,
            name: self.id_node_type,
        }
    }
}
