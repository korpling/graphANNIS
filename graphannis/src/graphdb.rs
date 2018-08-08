use graphstorage::adjacencylist::AdjacencyListStorage;
use stringstorage::StringStorage;
use annostorage::AnnoStorage;
use graphstorage::{GraphStorage, WriteableGraphStorage};
use api::update::{GraphUpdate, UpdateEvent};
use {Annotation, Component, ComponentType, Edge, NodeID, StringID};
use AnnoKey;
use graphstorage::registry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::{Arc,Mutex};
use std;
use strum::IntoEnumIterator;
use std::string::ToString;
use bincode;
use serde;
use malloc_size_of::{MallocSizeOf,MallocSizeOfOps};
use fs2::FileExt;
use std::fs::File;
use std::fs::OpenOptions;
use tempdir::TempDir;


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
    pub strings: Arc<StringStorage>,
    pub node_annos: Arc<AnnoStorage<NodeID>>,

    location: Option<PathBuf>,
    lock_file: Option<File>,

    components: BTreeMap<Component, Option<Arc<GraphStorage>>>,
    id_annis_ns: StringID,
    id_node_name: StringID,
    id_tok: StringID,
    id_node_type: StringID,

    current_change_id : u64,

    background_persistance : Arc<Mutex<()>>,
}

impl MallocSizeOf for GraphDB {

    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut size =
            self.strings.size_of(ops) + self.node_annos.size_of(ops);

        for (c, gs) in self.components.iter() {
            // TODO: overhead by map is not measured
            size += c.size_of(ops) + gs.size_of(ops);
        }

        return size;
    }
}


fn load_component_from_disk(component_path: Option<PathBuf>) -> Result<Arc<GraphStorage>, Error> {
    let cpath = try!(component_path.ok_or(Error::LocationEmpty));

    // load component into memory
    let impl_path = PathBuf::from(&cpath).join("impl.cfg");
    let mut f_impl = std::fs::File::open(impl_path)?;
    let mut impl_name = String::new();
    f_impl.read_to_string(&mut impl_name)?;

    let data_path = PathBuf::from(&cpath).join("component.bin");
    let f_data = std::fs::File::open(data_path)?;
    let mut buf_reader = std::io::BufReader::new(f_data);

    let gs = registry::deserialize(&impl_name, &mut buf_reader)?;

    return Ok(gs);
}

fn component_to_relative_path(c: &Component) -> PathBuf {
    let mut p = PathBuf::new();
    p.push("gs");
    p.push(c.ctype.to_string());
    p.push(if c.layer.is_empty() {"default_layer"} else {&c.layer});
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
    let result: T = bincode::deserialize_from(&mut reader)?;
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
    bincode::serialize_into(&mut writer, object)?;
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

            strings: Arc::new(strings),
            node_annos: Arc::new(AnnoStorage::<NodeID>::new()),
            components: BTreeMap::new(),

            location: None,
            lock_file: None,

            current_change_id: 0,

            background_persistance: Arc::new(Mutex::new(())),
        }
    }


    fn set_location(&mut self, location : &Path) -> Result<(), Error> {
        let lock_file_path = location.join("db.lock");
        // check if we can get the file lock
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(lock_file_path.as_path())?;
        lock_file.try_lock_exclusive()?;

        self.lock_file = Some(lock_file);
        self.location = Some(PathBuf::from(location));

        Ok(())
    }

    pub fn clear(&mut self) {
        self.strings = Arc::new(StringStorage::new());
        self.node_annos = Arc::new(AnnoStorage::new());
        self.components.clear();
    }

    pub fn load_from(&mut self, location: &Path, preload: bool) -> Result<(), Error> {
        self.clear();

        let location = PathBuf::from(location);

        self.set_location(location.as_path())?;
        let backup = location.join("backup");
        
        let mut backup_was_loaded = false;
        let dir2load = if backup.exists() && backup.is_dir() {
            backup_was_loaded = true;
            backup.clone()
        } else {
            location.join("current")
        };

        let strings_tmp : StringStorage = load_bincode(&dir2load, "strings.bin")?;
        self.strings = Arc::new(strings_tmp);
        let node_annos_tmp : AnnoStorage<NodeID> = load_bincode(&dir2load, "nodes.bin")?; 
        self.node_annos = Arc::from(node_annos_tmp);

        let log_path = dir2load.join("update_log.bin");

        let logfile_exists = log_path.exists() && log_path.is_file();

        self.find_components_from_disk(&dir2load)?;

        // If backup is active or a write log exists, always  a pre-load to get the complete corpus.
        if preload | logfile_exists | backup_was_loaded {
            let all_components: Vec<Component> = self.components.keys().cloned().collect();
            for c in all_components {
                self.ensure_loaded(&c)?;
            }
        }

        if logfile_exists {
            // apply any outstanding log file updates
            let f_log = std::fs::File::open(log_path)?;
            let mut buf_reader = std::io::BufReader::new(f_log);
            let update : GraphUpdate = bincode::deserialize_from(&mut buf_reader)?;
            if update.get_last_consistent_change_id() > self.current_change_id {
                self.apply_update_in_memory(&update)?;
            }
        } else {
            self.current_change_id = 0;
        }

        if backup_was_loaded {
            // save the current corpus under the actual location
            self.save_to(&location.join("current"))?;
            // rename backup folder (renaming is atomic and deleting could leave an incomplete backup folder on disk)
            let tmp_dir = TempDir::new_in(location,"temporary-graphannis-backup")?;
            std::fs::rename(&backup, tmp_dir.path())?;
            // remove it after renaming it
            tmp_dir.close()?;
        } 

        Ok(())
    }

    fn find_components_from_disk(&mut self, location: &Path) -> Result<(), Error> {
        self.components.clear();

        // for all component types
        for c in ComponentType::iter() {
            let cpath = PathBuf::from(location).join("gs").join(c.to_string());
          
            if cpath.is_dir() {
                // get all the namespaces/layers
                for layer in cpath.read_dir()? {
                    let layer = layer?;
                    if layer.path().is_dir() {
                        // try to load the component with the empty name
                        let empty_name_component = Component {
                            ctype: c.clone(),
                            layer: layer.file_name().into_string()?,
                            name: String::from(""),
                        };
                        {
                            let input_file = PathBuf::from(location)
                                .join(component_to_relative_path(&empty_name_component))
                                .join("component.bin");
                        
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
                            let data_file = PathBuf::from(location)
                                .join(component_to_relative_path(&named_component))
                                .join("component.bin");

                            let cfg_file = PathBuf::from(location)
                                .join(component_to_relative_path(&named_component))
                                .join("impl.cfg");
                                
                            if data_file.is_file() && cfg_file.is_file() {
                                self.components.insert(named_component.clone(), None);
                                debug!("Registered component {}", named_component);
                            }
                        }
                    }
                }
            }
        } // end for all components
        Ok(())
    }

    fn internal_save(&self, location: &Path) -> Result<(), Error> {
        let location = PathBuf::from(location);

        std::fs::create_dir_all(&location)?;

        save_bincode(&location, "strings.bin", self.strings.as_ref())?;
        save_bincode(&location, "nodes.bin", self.node_annos.as_ref())?;

        for (c, e) in self.components.iter() {
            if let Some(ref data) = *e {
                let dir = PathBuf::from(&location).join(component_to_relative_path(c));
                std::fs::create_dir_all(&dir)?;

                let data_path = PathBuf::from(&dir).join("component.bin");
                let f_data = std::fs::File::create(&data_path)?;
                let mut writer = std::io::BufWriter::new(f_data);
                let impl_name = registry::serialize(data.clone(), &mut writer)?;

                let cfg_path = PathBuf::from(&dir).join("impl.cfg");
                let mut f_cfg = std::fs::File::create(cfg_path)?;
                f_cfg.write_all(impl_name.as_bytes())?;
            }
        }
        Ok(())
    }

    // Save the current database to a location, but do not remember this location
    pub fn save_to(&mut self, location: &Path) -> Result<(), Error> {
        // make sure all components are loaded, otherwise saving them does not make any sense
        self.ensure_loaded_all()?;

        let lock_file_path = location.join("db.lock");
        // check if we can get the file lock
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(lock_file_path.as_path())?;
        lock_file.try_lock_exclusive()?;
        return self.internal_save(&location.join("current"));
    }

    /// Save the current database at is original location
    pub fn persist(&self) -> Result<(), Error> {
        if let Some(ref loc) = self.location {
            return self.internal_save(&loc.join("current"));
        } else {
            return Err(Error::LocationEmpty);
        }
    }

    /// Save the current database at a new location and remember it
    pub fn persist_to(&mut self, location: &Path) -> Result<(), Error> {
        
        self.set_location(location)?;
        return self.internal_save(&location.join("current"));
    }

    fn apply_update_in_memory(&mut self, u : &GraphUpdate) -> Result<(), Error> {
        for (id, change) in u.consistent_changes() {
            trace!("applying event {:?}", &change);
            match change {
                UpdateEvent::AddNode {
                    node_name,
                    node_type,
                } => {
                    let existing_node_id = self.get_node_id_from_name(&node_name);
                    // only add node if it does not exist yet
                    if existing_node_id.is_none() {
                        let new_node_id: NodeID = if let Some(id) = self.node_annos.get_largest_item() {
                            id + 1
                        } else {
                            0
                        };
                        let new_anno_name = Annotation {
                            key: self.get_node_name_key(),
                            val: Arc::make_mut(&mut self.strings).add(&node_name),
                        };
                        let new_anno_type = Annotation {
                            key: self.get_node_type_key(),
                            val: Arc::make_mut(&mut self.strings).add(&node_type),
                        };

                        // add the new node (with minimum labels)
                        let node_annos = Arc::make_mut(&mut self.node_annos);
                        node_annos.insert(new_node_id, new_anno_name);
                        node_annos.insert(new_node_id, new_anno_type);
                    }
                }
                UpdateEvent::DeleteNode { node_name } => {
                    if let Some(existing_node_id) = self.get_node_id_from_name(&node_name) {
                        // delete all annotations
                        {
                            let node_annos = Arc::make_mut(&mut self.node_annos);
                            for a in node_annos.get_all(&existing_node_id) {
                                node_annos.remove(&existing_node_id, &a.key);
                            }
                        }
                        // delete all edges pointing to this node either as source or target
                        for c in self.get_all_components(None, None) {
                            self.components.remove(&c);
                        }
                    }
                }
                UpdateEvent::AddNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                    anno_value,
                } => {
                    if let Some(existing_node_id) = self.get_node_id_from_name(&node_name) {
                        let anno = Annotation {
                            key: AnnoKey {
                                ns: Arc::make_mut(&mut self.strings).add(&anno_ns),
                                name: Arc::make_mut(&mut self.strings).add(&anno_name),
                            },
                            val: Arc::make_mut(&mut self.strings).add(&anno_value),
                        };
                        Arc::make_mut(&mut self.node_annos).insert(existing_node_id, anno);
                    }
                }
                UpdateEvent::DeleteNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                } => {
                    if let Some(existing_node_id) = self.get_node_id_from_name(&node_name) {
                        let key = AnnoKey {
                            ns: Arc::make_mut(&mut self.strings).add(&anno_ns),
                            name: Arc::make_mut(&mut self.strings).add(&anno_name),
                        };
                        Arc::make_mut(&mut self.node_annos).remove(&existing_node_id, &key);
                    }
                }
                UpdateEvent::AddEdge {
                    source_node,
                    target_node,
                    layer,
                    component_type,
                    component_name,
                } => {
                    // only add edge if both nodes already exist
                    if let (Some(source), Some(target)) = (
                        self.get_node_id_from_name(&source_node),
                        self.get_node_id_from_name(&target_node),
                    ) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer,
                                name: component_name,
                            };
                            let gs = self.get_or_create_writable(c)?;
                            gs.add_edge(Edge { source, target });
                        }
                    }
                }
                UpdateEvent::DeleteEdge {
                    source_node,
                    target_node,
                    layer,
                    component_type,
                    component_name,
                } => {
                    if let (Some(source), Some(target)) = (
                        self.get_node_id_from_name(&source_node),
                        self.get_node_id_from_name(&target_node),
                    ) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer,
                                name: component_name,
                            };
                            let gs = self.get_or_create_writable(c)?;
                            gs.delete_edge(&Edge { source, target });
                        }
                    }
                }
                UpdateEvent::AddEdgeLabel {
                    source_node,
                    target_node,
                    layer,
                    component_type,
                    component_name,
                    anno_ns,
                    anno_name,
                    anno_value,
                } => {
                    if let (Some(source), Some(target)) = (
                        self.get_node_id_from_name(&source_node),
                        self.get_node_id_from_name(&target_node),
                    ) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer,
                                name: component_name,
                            };
                            let ns = Arc::make_mut(&mut self.strings).add(&anno_ns);
                            let name = Arc::make_mut(&mut self.strings).add(&anno_name);
                            let val = Arc::make_mut(&mut self.strings).add(&anno_value);
                            let gs = self.get_or_create_writable(c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(&source, &target, 1, 1) {
                                let anno = Annotation {
                                    key: AnnoKey { ns, name },
                                    val,
                                };
                                gs.add_edge_annotation(e, anno);
                            }
                        }
                    }
                }
                UpdateEvent::DeleteEdgeLabel {
                    source_node,
                    target_node,
                    layer,
                    component_type,
                    component_name,
                    anno_ns,
                    anno_name,
                } => {
                     if let (Some(source), Some(target)) = (
                        self.get_node_id_from_name(&source_node),
                        self.get_node_id_from_name(&target_node),
                    ) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer,
                                name: component_name,
                            };
                            let ns = Arc::make_mut(&mut self.strings).add(&anno_ns);
                            let name = Arc::make_mut(&mut self.strings).add(&anno_name);
                            let gs = self.get_or_create_writable(c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(&source, &target, 1, 1) {
                                let key = AnnoKey { ns, name };
                                gs.delete_edge_annotation(&e, &key);
                            }
                        }
                    }
                }
            } // end match update entry type
            self.current_change_id = id;
        } // end for each consistent update entry
        Ok(())
    }

    pub fn apply_update(&mut self, mut u: &mut GraphUpdate) -> Result<(), Error> {
        trace!("applying updates");
        // Always mark the update state as consistent, even if caller forgot this.
        if !u.is_consistent() {
            u.finish();
        }

        // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
        self.ensure_loaded_all()?;

        let result = self.apply_update_in_memory(&u);

        trace!("memory updates completed");

        if let Some(location) = self.location.clone() {
            trace!("output location for persisting updates is {:?}", location);
            if result.is_ok() {

                let current_path = PathBuf::from(location).join("current");
                // make sure the output path exits
                std::fs::create_dir_all(&current_path)?;

                // if successfull write log
                let log_path = current_path.join("update_log.bin");
                
                trace!("writing WAL update log to {:?}", &log_path);
                let f_log = std::fs::File::create(log_path)?;
                let mut buf_writer = std::io::BufWriter::new(f_log);
                bincode::serialize_into(&mut buf_writer, &mut u)?;

                trace!("finished writing WAL update log");

            } else {
                trace!("error occured while applying updates: {:?}", &result);
                // load corpus from disk again
                self.load_from(&location, true)?;
                return result;
            }
        }

        Ok(())
    }

    /// A function to persist the changes of a write-ahead-log update on the disk. Should be run in a background thread.
    pub fn background_sync_wal_updates(&self) -> Result<(), Error> {
        
        // TODO: friendly abort any currently running thread

        if let Some(ref location) = self.location {
    
            // Accuire lock, so that only one thread can write background data at the same time
            let _lock = self.background_persistance.lock().unwrap();

            // Move the old corpus to the backup sub-folder. When the corpus is loaded again and there is backup folder
            // the backup will be used instead of the original possible corrupted files.
            // The current version is only the real one if no backup folder exists. If there is a backup folder
            // there is nothing to do since the backup already contains the last consistent version.
            // A sub-folder is used to ensure that all directories are on the same file system and moving (instead of copying)
            // is possible.
            if !location.join("backup").exists() {
                std::fs::rename(location.join("current"), location.join(location.join("backup")))?;
            }

            // Save the complete corpus without the write log to the target location
            self.internal_save(&location.join("current"))?;

            // remove the backup folder (since the new folder was completly written)
            std::fs::remove_dir_all(location.join("backup"))?;
        } 

        Ok(())   

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
                let mut gs_copy : AdjacencyListStorage = registry::create_writeable();
                gs_copy.copy(&self, loaded_comp.as_edgecontainer());
                Arc::from(gs_copy)
            };

            // (re-)insert the component into map again
            self.components.insert(c.clone(), Some(loaded_comp));
        }
        return Ok(());
    }

    pub fn calculate_component_statistics(&mut self, c: &Component) -> Result<(), Error> {
        let mut result: Result<(), Error> = Ok(());
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

    pub fn is_loaded(&self, c: &Component) -> bool {
        let entry: Option<&Option<Arc<GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if gs_opt.is_some() {
                return true;
            }
        }
        return false;
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
                        new_gs_mut.copy(self, gs.as_edgecontainer());
                        true
                    } else {
                        false
                    };
                    if converted {
                        // insert into components map
                        info!(
                            "Converted component {} to implementation {}",
                            c,
                            opt_type.to_string()
                        );
                        self.components.insert(c.clone(), Some(new_gs.clone()));
                    }
                }
            }
        }
    }

    pub fn get_node_id_from_name(&self, node_name: &str) -> Option<NodeID> {
        if let Some(node_name_id) = self.strings.find_id(node_name) {
            let mut all_nodes_with_anno = self.node_annos.exact_anno_search(
                Some(self.id_annis_ns),
                self.id_node_name,
                Some(node_name_id.clone()),
            );
            if let Some(m) = all_nodes_with_anno.next() {
                return Some(m.node);
            }
        }
        return None;
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

    pub fn get_graphstorage_as_ref<'a>(&'a self, c: &Component) -> Option<&'a GraphStorage> {
        // get and return the reference to the entry if loaded
        let entry: Option<&Option<Arc<GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if let Some(ref impl_type) = *gs_opt {
                return Some(impl_type.as_ref());
            }
        }
        return None;
    }

    pub fn get_all_components(
        &self,
        ctype: Option<ComponentType>,
        name: Option<&str>,
    ) -> Vec<Component> {
        if let (Some(ctype), Some(name)) = (ctype.clone(), name) {
            // lookup component from sorted map
            let mut result: Vec<Component> = Vec::new();
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
            let filtered_components = self.components.keys().cloned().filter(
                move |c: &Component| {
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
                },
            );
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
            ns: Arc::make_mut(&mut db.strings).add("test"),
            name: Arc::make_mut(&mut db.strings).add("edge_anno"),
        };
        let anno_val = Arc::make_mut(&mut db.strings).add("testValue");

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
