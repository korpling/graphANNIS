use annis::db::annostorage::AnnoStorage;
use annis::db::graphstorage::adjacencylist::AdjacencyListStorage;
use annis::db::graphstorage::registry;
use annis::db::graphstorage::{GraphStorage, WriteableGraphStorage};
use annis::db::update::{GraphUpdate, UpdateEvent};
use annis::errors::*;
use annis::types::AnnoKey;
use annis::types::{AnnoKeyID, Annotation, Component, ComponentType, Edge, NodeID};
use bincode;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use rayon::prelude::*;
use serde;
use std;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;
use tempdir::TempDir;

pub mod annostorage;
pub mod aql;
pub mod corpusstorage;
pub mod exec;
pub mod graphstorage;
mod plan;
pub mod query;
pub mod relannis;
pub mod sort_matches;
pub mod token_helper;
pub mod update;

pub const ANNIS_NS: &str = "annis";
pub const NODE_NAME: &str = "node_name";
pub const TOK: &str = "tok";
pub const NODE_TYPE: &str = "node_type";

/// A match is the result of a query on an annotation storage.
#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Match {
    node: NodeID,
    /// A unique internal identifier for the qualified annotation name.
    anno_key: AnnoKeyID,
}

impl Match {
    /// Get the node identifier this match refers to.
    pub fn get_node(&self) -> NodeID {
        self.node
    }   

    /// Extract the annotation for this match . The annotation value
    /// is retrieved from the `graph` given as argument.
    pub fn extract_annotation(&self, graph: &Graph) -> Option<Annotation> {
        let val = graph.node_annos.get_value_for_item_by_id(&self.node, self.anno_key)?.to_owned();
        let key = graph.node_annos.get_key_value(self.anno_key)?;
        Some(Annotation {
            key,
            val,
        })        
    }
}

/// Access annotations for nodes or edges.
pub trait AnnotationStorage<T> {
    /// Get all annotations for an `item` (node or edge).
    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation>;

    /// Return the total number of annotations contained in this `AnnotationStorage`.
    fn number_of_annotations(&self) -> usize;

    /// Return the number of annotations contained in this `AnnotationStorage` filtered by `name` and optional namespace (`ns`).
    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize;

    /// Returns an iterator for all items that match the given annotation constraints.
    /// The annotation `name` must be given as argument, the other arguments are optional.
    ///
    /// - `namespace`- If given, only annotations having this namespace are returned.
    /// - `name`  - Only annotations with this name are returned.
    /// - `value` - If given, only annotation having the given value are returned.
    ///
    /// The result is an iterator over matches.
    /// A match contains the node ID and the qualifed name of the matched annotation
    /// (e.g. there can be multiple annotations with the same name if the namespace is different).
    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a>;

    /// Estimate the number of results for an (annotation exact search)[#exact_anno_search] for a given an inclusive value range.
    ///
    /// - `ns` - If given, only annotations having this namespace are considered.
    /// - `name`  - Only annotations with this name are considered.
    /// - `lower_val`- Inclusive lower bound for the annotation value.
    /// - `upper_val`- Inclusive upper bound for the annotation value.
    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize;

    /// Return a list of all existing values for a given annotation `key`.
    /// If the `most_frequent_first`parameter is true, the results are sorted by their frequency.
    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str>;

    /// Get all the annotation keys which are part of this annotation storage
    fn annotation_keys(&self) -> Vec<AnnoKey>;
}

/// A representation of a graph including node annotations and edges.
/// Edges are partioned into components and each component is implemented by specialized graph storage implementation.
///
/// Use the [CorpusStorage](struct.CorpusStorage.html) struct to create and manage instances of a `Graph`.
///
/// Graphs can have an optional location on the disk.
/// In this case, changes to the graph via the [apply_update(...)](#method.apply_update) function are automatically persisted to this location.
///
pub struct Graph {
    node_annos: Arc<AnnoStorage<NodeID>>,

    location: Option<PathBuf>,

    components: BTreeMap<Component, Option<Arc<GraphStorage>>>,
    current_change_id: u64,

    background_persistance: Arc<Mutex<()>>,
}

impl MallocSizeOf for Graph {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut size = self.node_annos.size_of(ops);

        for (c, _) in self.components.iter() {
            // TODO: overhead by map is not measured
            size += c.size_of(ops);
            let gs_size = if let Some(gs) = self.get_graphstorage_as_ref(c) {
                gs.size_of(ops) + std::mem::size_of::<usize>()
            } else {
                // Option has the size of the nullable pointer/Arc
                std::mem::size_of::<usize>()
            };
            size += gs_size;
        }

        return size;
    }
}

fn load_component_from_disk(component_path: Option<PathBuf>) -> Result<Arc<GraphStorage>> {
    let cpath = try!(component_path.ok_or("Can't load component with empty path"));

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
    p.push(if c.layer.is_empty() {
        "default_layer"
    } else {
        &c.layer
    });
    p.push(&c.name);
    return p;
}

fn save_bincode<T>(location: &Path, path: &str, object: &T) -> Result<()>
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

impl AnnotationStorage<NodeID> for Graph {
    fn get_annotations_for_item(&self, item: &NodeID) -> Vec<Annotation> {
        self.node_annos.get_annotations_for_item(item)
    }

    fn number_of_annotations(&self) -> usize {
        self.node_annos.number_of_annotations()
    }

    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize {
        self.node_annos.number_of_annotations_by_name(ns, name)
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        self.node_annos.exact_anno_search(namespace, name, value)
    }

    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        self.node_annos
            .guess_max_count(ns, name, lower_val, upper_val)
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str> {
        self.node_annos.get_all_values(key, most_frequent_first)
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        self.node_annos.annotation_keys()
    }
}

impl Graph {
    /// Create a new and empty instance without any location on the disk.
    fn new() -> Graph {
        Graph {
            node_annos: Arc::new(AnnoStorage::<NodeID>::new()),
            components: BTreeMap::new(),

            location: None,

            current_change_id: 0,

            background_persistance: Arc::new(Mutex::new(())),
        }
    }

    fn set_location(&mut self, location: &Path) -> Result<()> {
        self.location = Some(PathBuf::from(location));

        Ok(())
    }

    /// Clear the graph content.
    /// This removes all node annotations, edges and knowledge about components.
    fn clear(&mut self) {
        self.node_annos = Arc::new(AnnoStorage::new());
        self.components.clear();
    }

    /// Load the graph from an external location.
    /// This sets the location of this instance to the given location.
    ///
    /// * `location` - The path on the disk
    /// * `preload` - If `true`, all components are loaded from disk into main memory.
    fn load_from(&mut self, location: &Path, preload: bool) -> Result<()> {
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

        let mut node_annos_tmp: AnnoStorage<NodeID> = AnnoStorage::new();
        node_annos_tmp.load_from_file(&dir2load.join("nodes.bin").to_string_lossy())?;
        self.node_annos = Arc::from(node_annos_tmp);

        let log_path = dir2load.join("update_log.bin");

        let logfile_exists = log_path.exists() && log_path.is_file();

        self.find_components_from_disk(&dir2load)?;

        // If backup is active or a write log exists, always  a pre-load to get the complete corpus.
        if preload | logfile_exists | backup_was_loaded {
            self.ensure_loaded_all()?;
        }

        if logfile_exists {
            // apply any outstanding log file updates
            let f_log = std::fs::File::open(log_path)?;
            let mut buf_reader = std::io::BufReader::new(f_log);
            let update: GraphUpdate = bincode::deserialize_from(&mut buf_reader)?;
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
            let tmp_dir = TempDir::new_in(location, "temporary-graphannis-backup")?;
            std::fs::rename(&backup, tmp_dir.path())?;
            // remove it after renaming it
            tmp_dir.close()?;
        }

        Ok(())
    }

    fn find_components_from_disk(&mut self, location: &Path) -> Result<()> {
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
                            layer: layer.file_name().to_string_lossy().to_string(),
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
                                layer: layer.file_name().to_string_lossy().to_string(),
                                name: name.file_name().to_string_lossy().to_string(),
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

    fn internal_save(&self, location: &Path) -> Result<()> {
        let location = PathBuf::from(location);

        std::fs::create_dir_all(&location)?;

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

    /// Save the current database to a `location` on the disk, but do not remember this location.
    fn save_to(&mut self, location: &Path) -> Result<()> {
        // make sure all components are loaded, otherwise saving them does not make any sense
        self.ensure_loaded_all()?;
        return self.internal_save(&location.join("current"));
    }

    /// Save the current database at a new `location` and remember it as new internal location.
    fn persist_to(&mut self, location: &Path) -> Result<()> {
        self.set_location(location)?;
        return self.internal_save(&location.join("current"));
    }

    fn apply_update_in_memory(&mut self, u: &GraphUpdate) -> Result<()> {
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
                        let new_node_id: NodeID =
                            if let Some(id) = self.node_annos.get_largest_item() {
                                id + 1
                            } else {
                                0
                            };
                        let new_anno_name = Annotation {
                            key: self.get_node_name_key(),
                            val: node_name,
                        };
                        let new_anno_type = Annotation {
                            key: self.get_node_type_key(),
                            val: node_type,
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
                            for a in node_annos.get_annotations_for_item(&existing_node_id) {
                                node_annos.remove_annotation_for_item(&existing_node_id, &a.key);
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
                                ns: anno_ns,
                                name: anno_name,
                            },
                            val: anno_value,
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
                            ns: anno_ns,
                            name: anno_name,
                        };
                        Arc::make_mut(&mut self.node_annos)
                            .remove_annotation_for_item(&existing_node_id, &key);
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
                            let gs = self.get_or_create_writable(c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(&source, &target, 1, 1) {
                                let anno = Annotation {
                                    key: AnnoKey {
                                        ns: anno_ns,
                                        name: anno_name,
                                    },
                                    val: anno_value,
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
                            let gs = self.get_or_create_writable(c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(&source, &target, 1, 1) {
                                let key = AnnoKey {
                                    ns: anno_ns,
                                    name: anno_name,
                                };
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

    /// Apply a sequence of updates (`u` parameter) to this graph.
    /// If the graph has a location on the disk, the changes are persisted.
    pub fn apply_update(&mut self, mut u: &mut GraphUpdate) -> Result<()> {
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
    fn background_sync_wal_updates(&self) -> Result<()> {
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
                std::fs::rename(
                    location.join("current"),
                    location.join(location.join("backup")),
                )?;
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

    fn insert_or_copy_writeable(&mut self, c: &Component) -> Result<()> {
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
                    .ok_or("Could not get mutable reference")?
                    .as_writeable()
                    .is_some()
            };

            let loaded_comp = if is_writable {
                loaded_comp
            } else {
                let mut gs_copy: AdjacencyListStorage = registry::create_writeable();
                gs_copy.copy(&self, loaded_comp.as_edgecontainer());
                Arc::from(gs_copy)
            };

            // (re-)insert the component into map again
            self.components.insert(c.clone(), Some(loaded_comp));
        }
        return Ok(());
    }

    fn calculate_component_statistics(&mut self, c: &Component) -> Result<()> {
        let mut result: Result<()> = Ok(());
        let mut entry = self
            .components
            .remove(c)
            .ok_or(format!("Component {} is missing", c.clone()))?;
        if let Some(ref mut gs) = entry {
            if let Some(gs_mut) = Arc::get_mut(gs) {
                // Since immutable graph storages can't change, only writable graph storage statistics need to be re-calculated
                if let Some(writeable_gs) = gs_mut.as_writeable() {
                    writeable_gs.calculate_statistics();
                }
            } else {
                result = Err(format!("Component {} is currently used", c.clone()).into());
            }
        }
        // re-insert component entry
        self.components.insert(c.clone(), entry);
        return result;
    }

    fn get_or_create_writable(&mut self, c: Component) -> Result<&mut WriteableGraphStorage> {
        if self.components.contains_key(&c) {
            // make sure the component is actually writable and loaded
            self.insert_or_copy_writeable(&c)?;
        } else {
            let w = registry::create_writeable();

            self.components.insert(c.clone(), Some(Arc::from(w)));
        }

        // get and return the reference to the entry
        let entry: &mut Arc<GraphStorage> = self
            .components
            .get_mut(&c)
            .ok_or("Could not get mutable reference")?
            .as_mut()
            .ok_or("Could not get mutable reference to optional value")?;
        let gs_mut_ref: &mut GraphStorage =
            Arc::get_mut(entry).ok_or("Could not get mutable reference")?;
        return Ok(gs_mut_ref.as_writeable().ok_or("Invalid type")?);
    }

    fn is_loaded(&self, c: &Component) -> bool {
        let entry: Option<&Option<Arc<GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if gs_opt.is_some() {
                return true;
            }
        }
        return false;
    }

    fn ensure_loaded_all(&mut self) -> Result<()> {
        let mut components_to_load: Vec<Component> = Vec::with_capacity(self.components.len());

        // colllect all missing components
        for (c, gs) in self.components.iter() {
            if gs.is_none() {
                components_to_load.push(c.clone());
            }
        }

        // load missing components in parallel
        let loaded_components: Vec<(Component, Result<Arc<GraphStorage>>)> = components_to_load
            .into_par_iter()
            .map(|c| {
                info!("Loading component {} from disk", c);
                let cpath = self.component_path(&c);
                let loaded_component = load_component_from_disk(cpath);
                (c, loaded_component)
            }).collect();

        // insert all the loaded components
        for (c, gs) in loaded_components.into_iter() {
            let gs = gs?;
            self.components.insert(c, Some(gs));
        }
        Ok(())
    }

    fn ensure_loaded(&mut self, c: &Component) -> Result<()> {
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

    fn optimize_impl(&mut self, c: &Component) {
        if let Some(gs) = self.get_graphstorage(c) {
            if let Some(stats) = gs.get_statistics() {
                let opt_info = registry::get_optimal_impl_heuristic(stats);

                // convert if necessary
                if opt_info.id != gs.serialization_id() {
                    let mut new_gs = registry::create_from_info(&opt_info);
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
                            c, opt_info.id,
                        );
                        self.components.insert(c.clone(), Some(new_gs.clone()));
                    }
                }
            }
        }
    }

    fn get_node_id_from_name(&self, node_name: &str) -> Option<NodeID> {
        let mut all_nodes_with_anno = self.node_annos.exact_anno_search(
            Some(ANNIS_NS.to_owned()),
            NODE_NAME.to_owned(),
            Some(node_name.to_owned()),
        );
        if let Some(m) = all_nodes_with_anno.next() {
            return Some(m.node);
        }
        return None;
    }

    /// Get a read-only graph storage reference for the given component `c`.
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

    fn get_graphstorage_as_ref<'a>(&'a self, c: &Component) -> Option<&'a GraphStorage> {
        // get and return the reference to the entry if loaded
        let entry: Option<&Option<Arc<GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if let Some(ref impl_type) = *gs_opt {
                return Some(impl_type.as_ref());
            }
        }
        return None;
    }

    /// Returns all components of the graph given an optional type (`ctype`) and `name`.
    /// This allows to filter which components to recieve.
    /// If you want to retrieve all components, use `None` as value for both arguments.
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
            let filtered_components =
                self.components
                    .keys()
                    .cloned()
                    .filter(move |c: &Component| {
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

    fn get_token_key(&self) -> AnnoKey {
        AnnoKey {
            ns: ANNIS_NS.to_owned(),
            name: TOK.to_owned(),
        }
    }

    fn get_node_name_key(&self) -> AnnoKey {
        AnnoKey {
            ns: ANNIS_NS.to_owned(),
            name: NODE_NAME.to_owned(),
        }
    }

    /// Return the annotation key which is used for the special `annis::node_type` annotation which every node must have to mark its existance.
    pub fn get_node_type_key(&self) -> AnnoKey {
        AnnoKey {
            ns: ANNIS_NS.to_owned(),
            name: NODE_TYPE.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use annis::types::{AnnoKey, Annotation, ComponentType, Edge};

    #[test]
    fn create_writeable_gs() {
        let mut db = Graph::new();

        let anno_key = AnnoKey {
            ns: "test".to_owned(),
            name: "edge_anno".to_owned(),
        };
        let anno_val = "testValue".to_owned();

        let gs: &mut WriteableGraphStorage = db
            .get_or_create_writable(Component {
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
