use annis::stringstorage::StringStorage;
use annis::annostorage::AnnoStorage;
use annis::graphstorage::{WriteableGraphStorage, ReadableGraphStorage};
use annis::{Component, NodeID, StringID};
use annis::AnnoKey;
use annis::graphstorage::registry;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::io::prelude::*;
use std;


const ANNIS_NS: &str = "annis";
const NODE_NAME: &str = "node_name";
const TOK: &str = "tok";
const NODE_TYPE: &str = "node_type";

#[derive(Clone)]
pub enum ImplType {
    Readable(Arc<ReadableGraphStorage>),
    Writable(Arc<WriteableGraphStorage>),
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
    component_keys: BTreeSet<Component>,
    loaded_components: BTreeMap<Component, ImplType>,

    id_annis_ns: StringID,
    id_node_name: StringID,
    id_tok: StringID,
    id_node_type: StringID,
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
            component_keys: BTreeSet::new(),
            loaded_components: BTreeMap::new(),

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

    pub fn create_writable_graphstorage(&mut self, c: Component) -> Result<Arc<WriteableGraphStorage>, Error> {

        match self.ensure_component_loaded(c.clone()) {
            Ok(impl_type) => {
                match impl_type {
                    ImplType::Readable(gs) => {
                        // convert the readable component to a writable one
                        let gs_copy :Arc<WriteableGraphStorage> = Arc::from(registry::create_writable_copy(gs.as_ref()));
                        // replace the current implementation
                        self.loaded_components.insert(c.clone(), ImplType::Writable(gs_copy.clone()));
                        Ok(gs_copy)
                    },
                    ImplType::Writable(gs) => {
                        // directly return the already loaded component
                        Ok(gs)
                    },
                }
            },
            Err(_) => {
                // no suitable component found, create a new one and register it
                let gs : Arc<WriteableGraphStorage> = Arc::from(registry::create_writeable());
                self.component_keys.insert(c.clone());
                self.loaded_components.insert(c.clone(), ImplType::Writable(gs.clone()));
                Ok(gs)
            }
        }

    }

    pub fn ensure_component_loaded(&mut self, c: Component) -> Result<ImplType, Error> {
        if self.component_keys.contains(&c) {
            // check if not loaded yet
            let cpath = try!(self.component_path(&c).ok_or(Error::LocationEmpty));
            if !self.loaded_components.contains_key(&c) {
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

                self.loaded_components.insert(c.clone(), gs);
            }

            return match self.loaded_components.get(&c) {
                Some(v) => Ok(v.clone()),
                None => Err(Error::Other),
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
