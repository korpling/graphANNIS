use stringstorage::StringStorage;
use annostorage::AnnoStorage;
use graphstorage::{GraphStorage, WriteableGraphStorage};
use {Component, ComponentType, Edge, NodeID, StringID};
use AnnoKey;
use graphstorage::registry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io::prelude::*;
use std;
use strum::IntoEnumIterator;
use std::string::ToString;
use bincode;
use serde;

pub const ANNIS_NS: &str = "annis";
pub const NODE_NAME: &str = "node_name";
pub const TOK: &str = "tok";
pub const NODE_TYPE: &str = "node_type";


#[derive(Debug)]
pub enum Error {
    IOerror(std::io::Error),
    StringError(std::ffi::OsString),
    RegistryError(registry::RegistryError),
    SerializationError(bincode::Error),
    LocationEmpty,
    InvalidType,
    MissingComponent,
    ComponentInUse,
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

impl From<std::ffi::OsString> for Error {
    fn from(e: std::ffi::OsString) -> Error {
        Error::StringError(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Error {
        Error::SerializationError(e)
    }
}


pub struct GraphDB {
    pub strings: StringStorage,
    pub node_annos: AnnoStorage<NodeID>,

    location: Option<PathBuf>,

    components: BTreeMap<Component, Option<Arc<GraphStorage>>>,
    id_annis_ns: StringID,
    id_node_name: StringID,
    id_tok: StringID,
    id_node_type: StringID,
}


fn load_component_from_disk(component_path: Option<PathBuf>) -> Result<Arc<GraphStorage>, Error> {
    let cpath = try!(component_path.ok_or(Error::LocationEmpty));

    // load component into memory
    let mut impl_path = PathBuf::from(&cpath);
    impl_path.push("impl.cfg");
    let mut f_impl = std::fs::File::open(impl_path)?;
    let mut impl_name = String::new();
    f_impl.read_to_string(&mut impl_name)?;

    let mut data_path = PathBuf::from(&cpath);
    data_path.push("component.bin");
    let f_data = std::fs::File::open(data_path)?;
    let mut buf_reader = std::io::BufReader::new(f_data);

    let gs = registry::deserialize(&impl_name, &mut buf_reader)?;

    return Ok(gs);
}

fn component_to_relative_path(c: &Component) -> PathBuf {
    let mut p = PathBuf::new();
    p.push("gs");
    p.push(c.ctype.to_string());
    p.push(&c.layer);
    p.push(&c.name);
    return p;
}


fn load_bincode<T>(location: &Path, path: &str) -> Result<T, Error>
where
    for<'de> T: serde::Deserialize<'de>,
{
    let mut full_path = PathBuf::from(location);
    full_path.push(path);

    let f = std::fs::File::open(full_path)?;
    let mut reader = std::io::BufReader::new(f);
    let result: T = bincode::deserialize_from(&mut reader, bincode::Infinite)?;
    return Ok(result);
}

fn save_bincode<T>(location: &Path, path: &str, object: &T) -> Result<(), Error>
where
    T: serde::Serialize,
{
    let mut full_path = PathBuf::from(location);
    full_path.push(path);

    let f = std::fs::File::create(full_path)?;
    let mut writer = std::io::BufWriter::new(f);
    bincode::serialize_into(&mut writer, object, bincode::Infinite)?;
    return Ok(());
}


impl GraphDB {
    /// Create a new and empty instance without any location on the disk
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

    pub fn clear(&mut self) {
        self.strings.clear();
        self.node_annos.clear();
        self.components.clear();
    }

    pub fn load_from(&mut self, location: &Path, preload: bool) -> Result<(), Error> {
        self.clear();

        let mut location = PathBuf::from(location);

        self.location = Some(location.clone());

        // TODO: implement WAL support
        location.push("current");
        self.strings = load_bincode(&location, "strings.bin")?;
        self.node_annos = load_bincode(&location, "nodes.bin")?;

        self.find_components_from_disk(&location)?;

        if preload {
            let all_components: Vec<Component> = self.components.keys().cloned().collect();
            for c in all_components {
                self.ensure_loaded(&c)?;
            }
        }

        Ok(())
    }

    fn find_components_from_disk(&mut self, location: &Path) -> Result<(), Error> {
        self.components.clear();

        // for all component types
        for c in ComponentType::iter() {
            let mut cpath = PathBuf::from(location);
            cpath.push("gs");
            cpath.push(c.to_string());
            if cpath.is_dir() {
                // get all the namespaces/layers
                for layer in cpath.read_dir()? {
                    let layer = layer?;
                    // try to load the component with the empty name
                    let empty_name_component = Component {
                        ctype: c.clone(),
                        layer: layer.file_name().into_string()?,
                        name: String::from(""),
                    };
                    {
                        let mut input_file = PathBuf::from(location);
                        input_file.push(component_to_relative_path(&empty_name_component));
                        input_file.push("component.bin");
                        if input_file.is_file() {
                            self.components.insert(empty_name_component.clone(), None);
                            debug!("Registered component {}", empty_name_component);
                        }
                    }
                    // also load all named components
                    for name in layer.path().read_dir()? {
                        let name = name?;
                        let named_component = Component {
                            ctype: c.clone(),
                            layer: layer.file_name().into_string()?,
                            name: name.file_name().into_string()?,
                        };
                        let mut data_file = PathBuf::from(location);
                        data_file.push(component_to_relative_path(&named_component));
                        data_file.push("component.bin");
                        let mut cfg_file = PathBuf::from(location);
                        cfg_file.push(component_to_relative_path(&named_component));
                        cfg_file.push("impl.cfg");
                        if data_file.is_file() && cfg_file.is_file() {
                            self.components.insert(named_component.clone(), None);
                            debug!("Registered component {}", named_component);
                        }
                    }
                }
            }
        } // end for all components
        Ok(())
    }

    fn internal_save(&self, location: &Path) -> Result<(), Error> {
        let mut location = PathBuf::from(location);
        location.push("current");

        std::fs::create_dir_all(&location)?; 

        save_bincode(&location, "strings.bin", &self.strings)?;
        save_bincode(&location, "nodes.bin", &self.node_annos)?;

        for (c, e) in self.components.iter() {
            if let Some(ref data) = *e {
                let mut dir = PathBuf::from(&location);
                dir.push(component_to_relative_path(c));
                std::fs::create_dir_all(&dir)?;

                let mut data_path = PathBuf::from(&dir);
                data_path.push("component.bin");
                let f_data = std::fs::File::create(data_path)?;
                let mut writer = std::io::BufWriter::new(f_data);
                let impl_name = registry::serialize(data.clone(), &mut writer)?;

                let mut cfg_path = PathBuf::from(&dir);
                cfg_path.push("impl.cfg");
                let mut f_cfg = std::fs::File::create(cfg_path)?;
                f_cfg.write_all(impl_name.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn save_to(&mut self, location: &Path) -> Result<(), Error> {
        
        // make sure all components are loaded, otherwise saving them does not make any sense
        self.ensure_loaded_all()?;

        return self.internal_save(location);       
    }

    /// Save the current database at is original location
    pub fn persist(&self) -> Result<(), Error> {
        if let Some(ref loc) = self.location {
            return self.internal_save(loc);
        } else {
            return Err(Error::LocationEmpty);
        }
    }

    fn component_path(&self, c: &Component) -> Option<PathBuf> {
        match self.location {
            Some(ref loc) => {
                let mut p = PathBuf::from(loc);
                // don't use the backup-folder per default
                p.push("current");
                p.push(component_to_relative_path(c));
                Some(p)
            }
            None => None,
        }
    }

    fn insert_or_copy_writeable(&mut self, c: &Component) -> Result<(), Error> {
        // move the old entry into the ownership of this function
        let entry = self.components.remove(c);
        // component exists?
        if entry.is_some() {
            let gs_opt = entry.unwrap();

            let mut loaded_comp: Arc<GraphStorage> = if gs_opt.is_none() {
                load_component_from_disk(self.component_path(c))?
            } else {
                gs_opt.unwrap()
            };

            // copy to writable implementation if needed
            let is_writable = {
                Arc::get_mut(&mut loaded_comp)
                    .ok_or(Error::Other)?
                    .as_writeable()
                    .is_some()
            };

            let loaded_comp = if is_writable {
                loaded_comp
            } else {
                let mut gs_copy = registry::create_writeable();
                gs_copy.copy(&self, loaded_comp.as_ref());
                Arc::from(gs_copy)
            };

            // (re-)insert the component into map again
            self.components.insert(c.clone(), Some(loaded_comp));
        }
        return Ok(());
    }

    pub fn calculate_component_statistics(&mut self, c: &Component) -> Result<(), Error> {
        let mut result : Result<(), Error> = Ok(());
        let mut entry = self.components.remove(c).ok_or(Error::MissingComponent)?;
        if let Some(ref mut gs) = entry {
            if let Some(gs_mut) = Arc::get_mut(gs) {
                gs_mut.calculate_statistics(&self.strings);
            } else {
                result = Err(Error::ComponentInUse);
            }
        }
        // re-insert component entry
        self.components.insert(c.clone(), entry);
        return result;
        
    }

    pub fn get_or_create_writable(
        &mut self,
        c: Component,
    ) -> Result<&mut WriteableGraphStorage, Error> {
        if self.components.contains_key(&c) {
            // make sure the component is actually writable and loaded
            self.insert_or_copy_writeable(&c)?;
        } else {
            let w = registry::create_writeable();

            self.components.insert(c.clone(), Some(Arc::from(w)));
        }

        // get and return the reference to the entry
        let entry: &mut Arc<GraphStorage> = self.components
            .get_mut(&c)
            .ok_or(Error::Other)?
            .as_mut()
            .ok_or(Error::Other)?;
        let gs_mut_ref: &mut GraphStorage = Arc::get_mut(entry).ok_or(Error::Other)?;
        return Ok(gs_mut_ref.as_writeable().ok_or(Error::InvalidType)?);
    }

    pub fn ensure_loaded_all(&mut self) -> Result<(), Error> {
        let all_components: Vec<Component> = self.components.keys().cloned().collect();
        for c in all_components {
            self.ensure_loaded(&c)?;
        }
        Ok(())
    }

    pub fn ensure_loaded(&mut self, c: &Component) -> Result<(), Error> {
        // get and return the reference to the entry if loaded
        let entry: Option<Option<Arc<GraphStorage>>> = self.components.remove(c);
        if let Some(gs_opt) = entry {
            let loaded: Arc<GraphStorage> = if gs_opt.is_none() {
            info!("Loading component {} from disk", c);
                load_component_from_disk(self.component_path(c))?
            } else {
                gs_opt.unwrap()
            };

            self.components.insert(c.clone(), Some(loaded));
        }
        return Ok(());
    }

    pub fn optimize_impl(&mut self, c: &Component) {
        if let Some(gs) = self.get_graphstorage(c) {
            let existing_type = registry::get_type(gs.clone());

            if let Some(stats) = gs.get_statistics() {
                let opt_type = registry::get_optimal_impl_heuristic(stats);
                
                // convert if necessary
                if existing_type.is_err() || opt_type != existing_type.unwrap() {
                    let mut new_gs = registry::create_from_type(opt_type.clone());
                    let converted = if let Some(new_gs_mut) = Arc::get_mut(&mut new_gs) {
                        new_gs_mut.copy(self, gs.as_ref());
                        true
                    } else {
                        false
                    };
                    if converted {
                        // insert into components map
                        info!("Converted component {} to implementation {}", c, opt_type.to_string());
                        self.components.insert(c.clone(), Some(new_gs.clone()));
                    }
                }


            }
        }
    }

    pub fn get_graphstorage(&self, c: &Component) -> Option<Arc<GraphStorage>> {
        // get and return the reference to the entry if loaded
        let entry: Option<&Option<Arc<GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if let Some(ref impl_type) = *gs_opt {
                return Some(impl_type.clone());
            }
        }
        return None;
    }

    pub fn get_all_components(&self, ctype : Option<ComponentType>, name : Option<&str>) -> Vec<Component> {
        
        if let (Some(ctype),Some(name)) = (ctype.clone(), name) {
            // lookup component from sorted map
            let mut result : Vec<Component> = Vec::new();
            let ckey = Component {
                ctype,
                name: String::from(name),
                layer: String::from(""),
            };

            for (c, _) in self.components.range(ckey..) {
                if c.name != name {
                    break;
                }
                result.push(c.clone());
            }
            return result;
        } else {
            // filter all entries
            let filtered_components = self.components.keys().cloned().filter(move |c : &Component| {
                if let Some(ctype) = ctype.clone() {
                    if ctype != c.ctype {
                        return false;
                    }
                }
                if let Some(name) = name {
                    if name != c.name {
                        return false;
                    }
                }
                return true;
            });
            return filtered_components.collect();
        }
    }

    pub fn get_direct_connected(&mut self, edge: &Edge) -> Result<Vec<Component>, Error> {
        let mut result = Vec::new();

        let all_components: Vec<Component> = self.components.keys().map(|c| c.clone()).collect();

        for c in all_components {
            self.ensure_loaded(&c)?;
            if let Some(gs) = self.get_graphstorage(&c) {
                if gs.is_connected(&edge.source, &edge.target, 1, 1) {
                    result.push(c.clone());
                }
            }
        }
        return Ok(result);
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

#[cfg(test)]
mod tests {
    use super::*;
    use {AnnoKey, Annotation, ComponentType, Edge};

    #[test]
    fn create_writeable_gs() {
        let mut db = GraphDB::new();

        let anno_key = AnnoKey {
            ns: db.strings.add("test"),
            name: db.strings.add("edge_anno"),
        };
        let anno_val = db.strings.add("testValue");

        let gs: &mut WriteableGraphStorage = db.get_or_create_writable(Component {
            ctype: ComponentType::Pointing,
            layer: String::from("test"),
            name: String::from("dep"),
        }).unwrap();

        gs.add_edge(Edge {
            source: 0,
            target: 1,
        });

        gs.add_edge_annotation(
            Edge {
                source: 0,
                target: 1,
            },
            Annotation {
                key: anno_key,
                val: anno_val,
            },
        );
    }
}
