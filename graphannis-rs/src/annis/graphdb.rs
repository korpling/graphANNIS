use annis::stringstorage::StringStorage;
use annis::annostorage::AnnoStorage;
use annis::graphstorage::{WriteableGraphStorage, ReadableGraphStorage};
use annis::graphstorage::adjacencylist::AdjacencyListStorage;
use annis::{Component, NodeID, StringID};
use annis::AnnoKey;
use annis::graphstorage::registry;
use annis::graphstorage::registry::{RegistryError};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::boxed::Box;


const ANNIS_NS: &str = "annis";
const NODE_NAME: &str = "node_name";
const TOK: &str = "tok";
const NODE_TYPE: &str = "node_type";

pub enum ImplType {
    Readable(Box<ReadableGraphStorage>),
    Writable(Box<WriteableGraphStorage>),
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

    fn create_writable_graphstorage(&mut self, c: Component) -> Result<&Box<WriteableGraphStorage>, RegistryError> {

        unimplemented!();
        
        // TODO: no suitable component found, create a new one and register it
        return Err(RegistryError::Other);
        
    }

    pub fn ensure_component_loaded(&mut self, c: Component) -> Option<&ImplType> {
        if self.component_keys.contains(&c) {
            // check if not loaded yet
            let cpath = self.component_path(&c);
            let e = self.loaded_components
                .entry(c)
                .or_insert_with(|| match cpath {
                    Some(ref _loc) => {
                        // let f = std::fs::File::open(loc);
                        // if f.is_ok() {
                        //     let mut buf_reader = std::io::BufReader::new(f.unwrap());

                        //     let loaded: Result<Box<ReadableGraphStorage>, _> =
                        //         bincode::deserialize_from(&mut buf_reader, bincode::Infinite);
                        //     if loaded.is_ok() {
                        //         *self = loaded.unwrap();
                        //     }
                        // }
                        ImplType::Writable(registry::create_writeable())
                    }
                    None => {
                        ImplType::Writable(registry::create_writeable())
                    }
                });
            return Some(e);
        }
        return None;
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
