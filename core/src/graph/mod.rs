pub mod serialization;
pub mod storage;
pub mod update;

use crate::{
    annostorage::{AnnotationStorage, NodeAnnotationStorage, ValueSearch},
    errors::Result,
    graph::storage::{registry, GraphStorage, WriteableGraphStorage},
};
use crate::{
    errors::GraphAnnisCoreError,
    types::{AnnoKey, Annotation, Component, ComponentType, Edge, NodeID},
};
use clru::CLruCache;
use rayon::prelude::*;
use smartstring::alias::String as SmartString;
use std::ops::Bound::Included;
use std::path::{Path, PathBuf};
use std::string::ToString;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};
use std::{collections::BTreeMap, num::NonZeroUsize};
use std::{collections::BTreeSet, io::prelude::*};
use update::{GraphUpdate, UpdateEvent};

pub const ANNIS_NS: &str = "annis";
pub const DEFAULT_NS: &str = "default_ns";
pub const NODE_NAME: &str = "node_name";
pub const NODE_TYPE: &str = "node_type";
pub const DEFAULT_EMPTY_LAYER: &str = "default_layer";

const GLOBAL_STATISTICS_FILE_NAME: &str = "global_statistics.toml";

lazy_static! {
    pub static ref DEFAULT_ANNO_KEY: Arc<AnnoKey> = Arc::from(AnnoKey::default());
    pub static ref NODE_NAME_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.into(),
        name: NODE_NAME.into(),
    });
    /// Return an annotation key which is used for the special `annis::node_type` annotation which every node must have to mark its existence.
    pub static ref NODE_TYPE_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.into(),
        name: NODE_TYPE.into(),
    });
}

/// A representation of a graph including node annotations and edges.
/// Edges are partioned into components and each component is implemented by specialized graph storage implementation.
///
/// Graphs can have an optional location on the disk.
/// In this case, changes to the graph via the [apply_update(...)](#method.apply_update) function are automatically persisted to this location.
///
pub struct Graph<CT: ComponentType> {
    node_annos: Box<dyn NodeAnnotationStorage>,

    location: Option<PathBuf>,

    components: BTreeMap<Component<CT>, Option<Arc<dyn GraphStorage>>>,
    current_change_id: u64,

    background_persistance: Arc<Mutex<()>>,

    pub global_statistics: Option<CT::GlobalStatistics>,

    disk_based: bool,
}

fn load_component_from_disk(component_path: &Path) -> Result<Arc<dyn GraphStorage>> {
    // load component into memory
    let impl_path = PathBuf::from(component_path).join("impl.cfg");
    let mut f_impl = std::fs::File::open(impl_path)?;
    let mut impl_name = String::new();
    f_impl.read_to_string(&mut impl_name)?;

    let gs = registry::deserialize(&impl_name, component_path)?;

    Ok(gs)
}

fn component_to_relative_path<CT: ComponentType>(c: &Component<CT>) -> PathBuf {
    let mut p = PathBuf::new();
    p.push("gs");
    p.push(c.get_type().to_string());
    p.push(if c.layer.is_empty() {
        DEFAULT_EMPTY_LAYER
    } else {
        &c.layer
    });
    p.push(c.name.as_str());
    p
}

fn component_path<CT: ComponentType>(
    location: &Option<PathBuf>,
    c: &Component<CT>,
) -> Option<PathBuf> {
    match location {
        Some(ref loc) => {
            let mut p = PathBuf::from(loc);
            // Check if we need to load the component from the backup folder
            let backup = loc.join("backup");
            if backup.exists() {
                p.push("backup");
            } else {
                p.push("current");
            }
            p.push(component_to_relative_path(c));
            Some(p)
        }
        None => None,
    }
}

/// List all the components that belong to corpus in the given directory.
pub fn find_components_from_disk<CT: ComponentType, P: AsRef<Path>>(
    location: P,
) -> Result<BTreeSet<Component<CT>>> {
    let mut result = BTreeSet::new();
    // for all component types
    for c in CT::all_component_types().into_iter() {
        let cpath = PathBuf::from(location.as_ref())
            .join("gs")
            .join(c.to_string());

        if cpath.is_dir() {
            // get all the namespaces/layers
            for layer in cpath.read_dir()? {
                let layer = layer?;
                if layer.path().is_dir() {
                    // try to load the component with the empty name
                    let layer_file_name = layer.file_name();
                    let layer_name_from_file = layer_file_name.to_string_lossy();
                    let layer_name: SmartString = if layer_name_from_file == DEFAULT_EMPTY_LAYER {
                        SmartString::default()
                    } else {
                        layer_name_from_file.into()
                    };
                    let empty_name_component =
                        Component::new(c.clone(), layer_name.clone(), SmartString::default());
                    {
                        let cfg_file = PathBuf::from(location.as_ref())
                            .join(component_to_relative_path(&empty_name_component))
                            .join("impl.cfg");

                        if cfg_file.is_file() {
                            result.insert(empty_name_component.clone());
                            debug!("Registered component {}", empty_name_component);
                        }
                    }
                    // also load all named components
                    for name in layer.path().read_dir()? {
                        let name = name?;
                        let named_component = Component::new(
                            c.clone(),
                            layer_name.clone(),
                            name.file_name().to_string_lossy().into(),
                        );
                        let cfg_file = PathBuf::from(location.as_ref())
                            .join(component_to_relative_path(&named_component))
                            .join("impl.cfg");

                        if cfg_file.is_file() {
                            result.insert(named_component.clone());
                            debug!("Registered component {}", named_component);
                        }
                    }
                }
            }
        }
    } // end for all components
    Ok(result)
}

impl<CT: ComponentType> Graph<CT> {
    /// Create a new and empty instance without any location on the disk.
    pub fn new(disk_based: bool) -> Result<Self> {
        let node_annos: Box<dyn NodeAnnotationStorage> = if disk_based {
            Box::new(crate::annostorage::ondisk::AnnoStorageImpl::new(None)?)
        } else {
            Box::new(crate::annostorage::inmemory::AnnoStorageImpl::<NodeID>::new())
        };

        Ok(Graph {
            node_annos,
            components: BTreeMap::new(),
            global_statistics: None,
            location: None,

            current_change_id: 0,

            background_persistance: Arc::new(Mutex::new(())),

            disk_based,
        })
    }

    /// Create a new instance without any location on the disk but with the default graph storage components.
    pub fn with_default_graphstorages(disk_based: bool) -> Result<Self> {
        let mut db = Graph::new(disk_based)?;
        for c in CT::default_components() {
            db.get_or_create_writable(&c)?;
        }
        Ok(db)
    }

    /// Opens the graph from an external location.
    /// All updates will be persisted to this location.
    ///
    /// * `location` - The path on the disk
    pub fn open(&mut self, location: &Path) -> Result<()> {
        debug!("Opening corpus from {}", location.to_string_lossy());
        self.clear()?;
        self.location = Some(location.to_path_buf());
        self.internal_open(location)?;

        Ok(())
    }

    /// Overwrites all content with the graph at the external location. Updates
    /// will *not* be persisted to this location and any old location will be
    /// cleared.
    ///
    /// * `location` - The path on the disk
    pub fn import(&mut self, location: &Path) -> Result<()> {
        debug!("Importing corpus from {}", location.to_string_lossy());
        self.clear()?;
        // Set the path temoporary to the actual path, so loading the components will not fail
        self.location = Some(location.to_path_buf());
        self.internal_open(location)?;
        self.ensure_loaded_all()?;
        // Unlink the location again
        self.location = None;
        Ok(())
    }

    /// Load the graph from an external location.
    /// This sets the location of this instance to the given location.
    ///
    /// * `location` - The path on the disk
    /// * `preload` - If `true`, all components are loaded from disk into main memory.
    #[deprecated(note = "Please use `open` instead")]
    pub fn load_from(&mut self, location: &Path, preload: bool) -> Result<()> {
        self.open(location)?;
        if preload {
            self.ensure_loaded_all()?;
        }
        Ok(())
    }

    /// Internal helper function that loads the content from an external
    /// location.
    ///
    /// It does not clear the current graph and does not alter the configured
    /// location to persist the graph to.
    fn internal_open(&mut self, location: &Path) -> Result<()> {
        let backup = location.join("backup");

        let mut load_from_backup = false;
        let dir2load = if backup.exists() && backup.is_dir() {
            load_from_backup = true;
            backup.clone()
        } else {
            location.join("current")
        };

        // Get the global statistics if available
        self.global_statistics = None;
        let global_statistics_file = dir2load.join(GLOBAL_STATISTICS_FILE_NAME);
        if global_statistics_file.exists() && global_statistics_file.is_file() {
            let file_content = std::fs::read_to_string(global_statistics_file)?;
            self.global_statistics = Some(toml::from_str(&file_content)?);
        }

        // Load the node annotations
        let ondisk_subdirectory = dir2load.join(crate::annostorage::ondisk::SUBFOLDER_NAME);
        if ondisk_subdirectory.exists() && ondisk_subdirectory.is_dir() {
            self.disk_based = true;
            // directly load the on disk storage from the given folder to avoid having a temporary directory
            let node_annos_tmp =
                crate::annostorage::ondisk::AnnoStorageImpl::new(Some(ondisk_subdirectory))?;
            self.node_annos = Box::new(node_annos_tmp);
        } else {
            // assume a main memory implementation
            self.disk_based = false;
            let mut node_annos_tmp = crate::annostorage::inmemory::AnnoStorageImpl::new();
            node_annos_tmp.load_annotations_from(&dir2load)?;
            self.node_annos = Box::new(node_annos_tmp);
        }

        let log_path = dir2load.join("update_log.bin");

        let logfile_exists = log_path.exists() && log_path.is_file();

        self.components = find_components_from_disk(&dir2load)?
            .into_iter()
            .map(|c| (c, None))
            .collect();

        // If backup is active or a write log exists, always  a pre-load to get the complete corpus.
        if logfile_exists | load_from_backup {
            self.ensure_loaded_all()?;
        }

        if logfile_exists {
            // apply any outstanding log file updates
            let log_reader = std::fs::File::open(&log_path)?;
            let mut update = bincode::deserialize_from(log_reader)?;
            self.apply_update_in_memory(&mut update, true, |_| {})?;
        } else {
            self.current_change_id = 0;
        }

        if load_from_backup {
            // save the current corpus under the actual location
            self.save_to(&location.join("current"))?;
            // rename backup folder (renaming is atomic and deleting could leave an incomplete backup folder on disk)
            let tmp_dir = tempfile::Builder::new()
                .prefix("temporary-graphannis-backup")
                .tempdir_in(location)?;
            // the target directory is created and can cause issues on windows: delete it first
            std::fs::remove_dir(tmp_dir.path())?;
            std::fs::rename(&backup, tmp_dir.path())?;
            // remove it after renaming it
            tmp_dir.close()?;
        }

        Ok(())
    }

    /// Save the current database to a `location` on the disk, but do not remember this location.
    pub fn save_to(&mut self, location: &Path) -> Result<()> {
        // make sure all components are loaded, otherwise saving them does not make any sense
        self.ensure_loaded_all()?;
        self.internal_save(&location.join("current"))
    }

    /// Save the current database at a new `location` and remember it as new internal location.
    pub fn persist_to(&mut self, location: &Path) -> Result<()> {
        self.location = Some(location.to_path_buf());
        self.internal_save(&location.join("current"))
    }

    /// Clear the graph content.
    /// This removes all node annotations, edges and knowledge about components.
    fn clear(&mut self) -> Result<()> {
        self.node_annos = Box::new(crate::annostorage::inmemory::AnnoStorageImpl::new());
        self.components.clear();
        Ok(())
    }

    fn internal_save(&self, location: &Path) -> Result<()> {
        let location = PathBuf::from(location);

        std::fs::create_dir_all(&location)?;

        self.node_annos.save_annotations_to(&location)?;

        for (c, e) in &self.components {
            if let Some(ref data) = *e {
                let dir = PathBuf::from(&location).join(component_to_relative_path(c));
                std::fs::create_dir_all(&dir)?;

                let impl_name = data.serialization_id();
                data.save_to(&dir)?;

                let cfg_path = PathBuf::from(&dir).join("impl.cfg");
                let mut f_cfg = std::fs::File::create(cfg_path)?;
                f_cfg.write_all(impl_name.as_bytes())?;
            }
        }

        // Save global statistics
        if let Some(s) = &self.global_statistics {
            let file_content = toml::to_string(s)?;
            std::fs::write(location.join(GLOBAL_STATISTICS_FILE_NAME), file_content)?;
        }

        Ok(())
    }

    fn get_cached_node_id_from_name(
        &self,
        node_name: Cow<String>,
        cache: &mut CLruCache<String, Option<NodeID>>,
    ) -> Result<Option<NodeID>> {
        if let Some(id) = cache.get(node_name.as_ref()) {
            Ok(*id)
        } else {
            let id = self.node_annos.get_node_id_from_name(&node_name)?;
            cache.put(node_name.to_string(), id);
            Ok(id)
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn apply_update_in_memory<F>(
        &mut self,
        u: &mut GraphUpdate,
        update_statistics: bool,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(&str),
    {
        let all_components = self.get_all_components(None, None);

        let mut update_graph_index = ComponentType::init_update_graph_index(self)?;
        // Cache the expensive mapping of node names to IDs
        let cache_size = NonZeroUsize::new(1_000).ok_or(GraphAnnisCoreError::ZeroCacheSize)?;
        let mut node_id_cache = CLruCache::new(cache_size);
        // Iterate once over all changes in the same order as the updates have been added
        let total_nr_updates = u.len()?;
        progress_callback(&format!("applying {} atomic updates", total_nr_updates));
        for (nr_updates, update_event) in u.iter()?.enumerate() {
            let (id, change) = update_event?;
            trace!("applying event {:?}", &change);
            ComponentType::before_update_event(&change, self, &mut update_graph_index)?;
            match &change {
                UpdateEvent::AddNode {
                    node_name,
                    node_type,
                } => {
                    // only add node if it does not exist yet
                    if !self.node_annos.has_node_name(node_name)? {
                        let new_node_id: NodeID =
                            if let Some(id) = self.node_annos.get_largest_item()? {
                                id + 1
                            } else {
                                0
                            };

                        let new_anno_name = Annotation {
                            key: NODE_NAME_KEY.as_ref().clone(),
                            val: node_name.into(),
                        };
                        let new_anno_type = Annotation {
                            key: NODE_TYPE_KEY.as_ref().clone(),
                            val: node_type.into(),
                        };

                        // add the new node (with minimum labels)
                        self.node_annos.insert(new_node_id, new_anno_name)?;
                        self.node_annos.insert(new_node_id, new_anno_type)?;

                        // update the internal cache
                        node_id_cache.put(node_name.clone(), Some(new_node_id));
                    }
                }
                UpdateEvent::DeleteNode { node_name } => {
                    if let Some(existing_node_id) = self.get_cached_node_id_from_name(
                        Cow::Borrowed(node_name),
                        &mut node_id_cache,
                    )? {
                        // delete all annotations
                        self.node_annos.remove_item(&existing_node_id)?;

                        // delete all edges pointing to this node either as source or target
                        for c in all_components.iter() {
                            if let Ok(gs) = self.get_or_create_writable(c) {
                                gs.delete_node(existing_node_id)?;
                            }
                        }

                        // update the internal cache
                        node_id_cache.put(node_name.clone(), None);
                    }
                }
                UpdateEvent::AddNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                    anno_value,
                } => {
                    if let Some(existing_node_id) = self.get_cached_node_id_from_name(
                        Cow::Borrowed(node_name),
                        &mut node_id_cache,
                    )? {
                        let anno = Annotation {
                            key: AnnoKey {
                                ns: anno_ns.into(),
                                name: anno_name.into(),
                            },
                            val: anno_value.into(),
                        };
                        self.node_annos.insert(existing_node_id, anno)?;
                    }
                }
                UpdateEvent::DeleteNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                } => {
                    if let Some(existing_node_id) = self.get_cached_node_id_from_name(
                        Cow::Borrowed(node_name),
                        &mut node_id_cache,
                    )? {
                        let key = AnnoKey {
                            ns: anno_ns.into(),
                            name: anno_name.into(),
                        };
                        self.node_annos
                            .remove_annotation_for_item(&existing_node_id, &key)?;
                    }
                }
                UpdateEvent::AddEdge {
                    source_node,
                    target_node,
                    layer,
                    component_type,
                    component_name,
                } => {
                    let source = self.get_cached_node_id_from_name(
                        Cow::Borrowed(source_node),
                        &mut node_id_cache,
                    )?;
                    let target = self.get_cached_node_id_from_name(
                        Cow::Borrowed(target_node),
                        &mut node_id_cache,
                    )?;
                    // only add edge if both nodes already exist
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = CT::from_str(component_type) {
                            let c = Component::new(ctype, layer.into(), component_name.into());
                            let gs = self.get_or_create_writable(&c)?;
                            gs.add_edge(Edge { source, target })?;
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
                    let source = self.get_cached_node_id_from_name(
                        Cow::Borrowed(source_node),
                        &mut node_id_cache,
                    )?;
                    let target = self.get_cached_node_id_from_name(
                        Cow::Borrowed(target_node),
                        &mut node_id_cache,
                    )?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = CT::from_str(component_type) {
                            let c = Component::new(ctype, layer.into(), component_name.into());

                            let gs = self.get_or_create_writable(&c)?;
                            gs.delete_edge(&Edge { source, target })?;
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
                    let source = self.get_cached_node_id_from_name(
                        Cow::Borrowed(source_node),
                        &mut node_id_cache,
                    )?;
                    let target = self.get_cached_node_id_from_name(
                        Cow::Borrowed(target_node),
                        &mut node_id_cache,
                    )?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = CT::from_str(component_type) {
                            let c = Component::new(ctype, layer.into(), component_name.into());
                            let gs = self.get_or_create_writable(&c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(source, target, 1, Included(1))? {
                                let anno = Annotation {
                                    key: AnnoKey {
                                        ns: anno_ns.into(),
                                        name: anno_name.into(),
                                    },
                                    val: anno_value.into(),
                                };
                                gs.add_edge_annotation(e, anno)?;
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
                    let source = self.get_cached_node_id_from_name(
                        Cow::Borrowed(source_node),
                        &mut node_id_cache,
                    )?;
                    let target = self.get_cached_node_id_from_name(
                        Cow::Borrowed(target_node),
                        &mut node_id_cache,
                    )?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = CT::from_str(component_type) {
                            let c = Component::new(ctype, layer.into(), component_name.into());
                            let gs = self.get_or_create_writable(&c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(source, target, 1, Included(1))? {
                                let key = AnnoKey {
                                    ns: anno_ns.into(),
                                    name: anno_name.into(),
                                };
                                gs.delete_edge_annotation(&e, &key)?;
                            }
                        }
                    }
                }
            } // end match update entry type
            ComponentType::after_update_event(change, self, &mut update_graph_index)?;
            self.current_change_id = id;

            if nr_updates > 0 && nr_updates % 100_000 == 0 {
                // Get progress in percentage
                let progress = ((nr_updates as f64) / (total_nr_updates as f64)) * 100.0;
                progress_callback(&format!(
                    "applied {:.2}% of the atomic updates ({}/{})",
                    progress, nr_updates, total_nr_updates,
                ));
            }
        } // end for each consistent update entry

        if update_statistics {
            progress_callback("calculating all statistics");
            self.calculate_all_statistics()?;
        }

        progress_callback("extending graph with model-specific index");
        ComponentType::apply_update_graph_index(update_graph_index, self)?;

        Ok(())
    }

    /// Apply a sequence of updates (`u` parameter) to this graph.
    /// If the graph has a location on the disk, the changes are persisted.
    pub fn apply_update<F>(&mut self, u: &mut GraphUpdate, progress_callback: F) -> Result<()>
    where
        F: Fn(&str),
    {
        progress_callback("applying list of atomic updates");

        // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
        self.ensure_loaded_all()?;

        let result = self.apply_update_in_memory(u, true, &progress_callback);
        progress_callback("memory updates completed, persisting updates to disk");
        self.persist_updates(u, result, progress_callback)?;
        Ok(())
    }

    /// Apply a sequence of updates (`u` parameter) to this graph but do not update the graph statistics.
    /// If the graph has a location on the disk, the changes are persisted.
    pub fn apply_update_keep_statistics<F>(
        &mut self,
        u: &mut GraphUpdate,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(&str),
    {
        progress_callback("applying list of atomic updates");

        // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
        self.ensure_loaded_all()?;

        let result = self.apply_update_in_memory(u, false, &progress_callback);
        progress_callback("memory updates completed, persisting updates to disk");
        self.persist_updates(u, result, progress_callback)?;
        Ok(())
    }

    fn persist_updates<F>(
        &mut self,
        u: &mut GraphUpdate,
        apply_update_result: Result<()>,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(&str),
    {
        if let Some(location) = self.location.clone() {
            trace!("output location for persisting updates is {:?}", location);
            if apply_update_result.is_ok() {
                let current_path = location.join("current");
                // make sure the output path exits
                std::fs::create_dir_all(&current_path)?;

                // If successfull write log
                let log_path = current_path.join("update_log.bin");

                // Create a temporary directory in the same file system as the output
                let temporary_dir = tempfile::tempdir_in(&current_path)?;
                let mut temporary_disk_file = tempfile::NamedTempFile::new_in(&temporary_dir)?;

                debug!("writing WAL update log to {:?}", temporary_disk_file.path());
                bincode::serialize_into(temporary_disk_file.as_file(), &u)?;
                temporary_disk_file.flush()?;
                debug!("moving finished WAL update log to {:?}", &log_path);
                // Since the temporary file should be on the same file system, persisting/moving it should be an atomic operation
                temporary_disk_file.persist(&log_path)?;

                progress_callback("finished writing WAL update log");
            } else {
                trace!(
                    "error occured while applying updates: {:?}",
                    &apply_update_result
                );
                // load corpus from disk again
                self.open(&location)?;
                self.ensure_loaded_all()?;
            }
        }

        apply_update_result
    }

    /// A function to persist the changes of a write-ahead-log update on the disk. Should be run in a background thread.
    pub fn background_sync_wal_updates(&self) -> Result<()> {
        // TODO: friendly abort any currently running thread

        if let Some(ref location) = self.location {
            // Acquire lock, so that only one thread can write background data at the same time
            let _lock = self.background_persistance.lock()?;

            self.internal_save_with_backup(location)?;
        }

        Ok(())
    }

    /// Save this graph to the given location using a temporary backup folder for the old graph.
    /// The backup folder is used to achieve some atomicity in combination with the `load_from` logic,
    // which will load the backup folder in case saving the corpus to the "current" location was aborted.
    fn internal_save_with_backup(&self, location: &Path) -> Result<()> {
        // Move the old corpus to the backup sub-folder. When the corpus is loaded again and there is backup folder
        // the backup will be used instead of the original possible corrupted files.
        // The current version is only the real one if no backup folder exists. If there is a backup folder
        // there is nothing to do since the backup already contains the last consistent version.
        // A sub-folder is used to ensure that all directories are on the same file system and moving (instead of copying)
        // is possible.
        let backup_location = location.join("backup");
        let current_location = location.join("current");
        if !backup_location.exists() {
            std::fs::rename(&current_location, &backup_location)?;
        }

        // Save the complete corpus without the write log to the target location
        self.internal_save(&current_location)?;

        // rename backup folder (renaming is atomic and deleting could leave an incomplete backup folder on disk)
        let tmp_dir = tempfile::Builder::new()
            .prefix("temporary-graphannis-backup")
            .tempdir_in(location)?;
        // the target directory is created and can cause issues on windows: delete it first
        std::fs::remove_dir(tmp_dir.path())?;
        std::fs::rename(&backup_location, tmp_dir.path())?;
        // remove it after renaming it, (since the new "current" folder was completely written)
        tmp_dir.close()?;
        Ok(())
    }

    fn ensure_writeable(&mut self, c: &Component<CT>) -> Result<()> {
        self.ensure_loaded(c)?;
        // Short path: exists and already writable
        if let Some(gs_opt) = self.components.get_mut(c) {
            // This should always work, since we just ensured the component is loaded
            let gs = gs_opt
                .as_mut()
                .ok_or_else(|| GraphAnnisCoreError::ComponentNotLoaded(c.to_string()))?;
            // copy to writable implementation if needed
            let is_writable = {
                Arc::get_mut(gs)
                    .ok_or_else(|| {
                        GraphAnnisCoreError::NonExclusiveComponentReference(c.to_string())
                    })?
                    .as_writeable()
                    .is_some()
            };
            if is_writable {
                return Ok(());
            }
        } else {
            // Component does not exist at all, we can abort here
            return Ok(());
        }

        // Component does exist, but is not writable, replace with writeable implementation
        let readonly_gs = self
            .components
            .get(c)
            .cloned()
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(c.to_string()))?
            .ok_or_else(|| GraphAnnisCoreError::ComponentNotLoaded(c.to_string()))?;
        let writable_gs = registry::create_writeable(self, Some(readonly_gs.as_ref()))?;
        self.components.insert(c.to_owned(), Some(writable_gs));

        Ok(())
    }

    /// (Re-) calculate the internal statistics needed for estimating graph components and annotations.
    pub fn calculate_all_statistics(&mut self) -> Result<()> {
        self.ensure_loaded_all()?;

        debug!("Calculating node statistics");
        self.node_annos.calculate_statistics()?;
        for c in self.get_all_components(None, None) {
            debug!("Calculating statistics for component {}", &c);
            self.calculate_component_statistics(&c)?;
        }

        debug!("Calculating global graph statistics");
        CT::calculate_global_statistics(self)?;

        Ok(())
    }

    /// Makes sure the statistics for the given component are up-to-date.
    pub fn calculate_component_statistics(&mut self, c: &Component<CT>) -> Result<()> {
        let mut result: Result<()> = Ok(());
        let mut entry = self
            .components
            .remove(c)
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(c.to_string()))?;
        if let Some(ref mut gs) = entry {
            if let Some(gs_mut) = Arc::get_mut(gs) {
                // Since immutable graph storages can't change, only writable graph storage statistics need to be re-calculated
                if let Some(writeable_gs) = gs_mut.as_writeable() {
                    writeable_gs.calculate_statistics()?;
                }
            } else {
                result = Err(GraphAnnisCoreError::NonExclusiveComponentReference(
                    c.to_string(),
                ));
            }
        }
        // re-insert component entry
        self.components.insert(c.clone(), entry);
        result
    }

    /// Gets the the given component.
    /// If the component does not exist yet, it creates a  new empty one.
    /// If the existing component is non-writable, a writable copy of it is created and returned.
    pub fn get_or_create_writable(
        &mut self,
        c: &Component<CT>,
    ) -> Result<&mut dyn WriteableGraphStorage> {
        if self.components.contains_key(c) {
            // make sure the component is actually writable and loaded
            self.ensure_writeable(c)?;
        } else {
            let w = registry::create_writeable(self, None)?;

            self.components.insert(c.clone(), Some(w));
        }

        // get and return the reference to the entry
        let entry: &mut Arc<dyn GraphStorage> = self
            .components
            .get_mut(c)
            .ok_or_else(|| GraphAnnisCoreError::MissingComponent(c.to_string()))?
            .as_mut()
            .ok_or_else(|| GraphAnnisCoreError::ComponentNotLoaded(c.to_string()))?;

        let gs_mut_ref: &mut dyn GraphStorage = Arc::get_mut(entry)
            .ok_or_else(|| GraphAnnisCoreError::NonExclusiveComponentReference(c.to_string()))?;
        let result = gs_mut_ref
            .as_writeable()
            .ok_or_else(|| GraphAnnisCoreError::ReadOnlyComponent(c.to_string()))?;
        Ok(result)
    }

    /// Returns `true` if the graph storage for this specific component is loaded and ready to use.
    pub fn is_loaded(&self, c: &Component<CT>) -> bool {
        let entry: Option<&Option<Arc<dyn GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if gs_opt.is_some() {
                return true;
            }
        }
        false
    }

    /// Ensure that the graph storages for all component are loaded and ready to use.
    pub fn ensure_loaded_all(&mut self) -> Result<()> {
        let mut components_to_load: Vec<_> = Vec::with_capacity(self.components.len());

        // colllect all missing components
        for (c, gs) in &self.components {
            if gs.is_none() {
                components_to_load.push(c.clone());
            }
        }

        self.ensure_loaded_parallel(&components_to_load)?;
        Ok(())
    }

    /// Ensure that the graph storage for a specific component is loaded and ready to use.
    pub fn ensure_loaded(&mut self, c: &Component<CT>) -> Result<()> {
        // We only load known components, so check the map if the entry exists
        if let Some(gs_opt) = self.components.get_mut(c) {
            // If this is none, the component is known but not loaded
            if gs_opt.is_none() {
                let component_path = component_path(&self.location, c)
                    .ok_or(GraphAnnisCoreError::EmptyComponentPath)?;
                debug!(
                    "loading component {} from {}",
                    c,
                    &component_path.to_string_lossy()
                );
                let component = load_component_from_disk(&component_path)?;
                gs_opt.get_or_insert_with(|| component);
            }
        }
        Ok(())
    }

    /// Ensure that the graph storage for a the given component is loaded and ready to use.
    /// Loading is done in parallel.
    ///
    /// Returns the list of actually loaded (and existing) components.
    pub fn ensure_loaded_parallel(
        &mut self,
        components_to_load: &[Component<CT>],
    ) -> Result<Vec<Component<CT>>> {
        // We only load known components, so check the map if the entry exists
        // and that is not loaded yet.
        let components_to_load: Vec<_> = components_to_load
            .iter()
            .filter(|c| {
                if let Some(e) = self.components.get(c) {
                    e.is_none()
                } else {
                    false
                }
            })
            .collect();

        // load missing components in parallel
        let loaded_components: Vec<(_, Result<Arc<dyn GraphStorage>>)> = components_to_load
            .into_par_iter()
            .map(|c| match component_path(&self.location, c) {
                Some(cpath) => {
                    debug!(
                        "loading component in parallel {} from {}",
                        c,
                        &cpath.to_string_lossy()
                    );
                    (c, load_component_from_disk(&cpath))
                }
                None => (c, Err(GraphAnnisCoreError::EmptyComponentPath)),
            })
            .collect();

        // insert all the loaded components
        let mut result = Vec::with_capacity(loaded_components.len());
        for (c, gs) in loaded_components {
            let gs = gs?;
            self.components.insert(c.clone(), Some(gs));
            result.push(c.clone());
        }
        Ok(result)
    }

    pub fn optimize_impl(&mut self, disk_based: bool) -> Result<()> {
        self.ensure_loaded_all()?;

        if self.disk_based != disk_based {
            self.disk_based = disk_based;

            // Change the node annotation implementation
            let mut new_node_annos: Box<dyn NodeAnnotationStorage> = if disk_based {
                Box::new(crate::annostorage::ondisk::AnnoStorageImpl::new(None)?)
            } else {
                Box::new(crate::annostorage::inmemory::AnnoStorageImpl::<NodeID>::new())
            };

            // Copy all annotations for all nodes
            info!("copying node annotations");
            for m in self
                .node_annos
                .exact_anno_search(Some(ANNIS_NS), NODE_TYPE, ValueSearch::Any)
            {
                let m = m?;
                for anno in self.node_annos.get_annotations_for_item(&m.node)? {
                    new_node_annos.insert(m.node, anno)?;
                }
            }
            self.node_annos = new_node_annos;
        }

        info!("re-calculating all statistics");
        self.calculate_all_statistics()?;

        for c in self.get_all_components(None, None) {
            // Perform the optimization if necessary
            info!("optimizing implementation for component {}", &c);
            self.optimize_gs_impl(&c)?;
        }
        if let Some(location) = &self.location {
            info!("saving corpus to disk");
            self.internal_save_with_backup(location)?;
        }
        Ok(())
    }

    pub fn optimize_gs_impl(&mut self, c: &Component<CT>) -> Result<()> {
        if let Some(gs) = self.get_graphstorage(c) {
            if let Some(stats) = gs.get_statistics() {
                let opt_info = registry::get_optimal_impl_heuristic(self, stats);

                // convert if necessary
                if opt_info.id != gs.serialization_id() {
                    let mut new_gs = registry::create_from_info(&opt_info)?;
                    let converted = if let Some(new_gs_mut) = Arc::get_mut(&mut new_gs) {
                        info!(
                            "converting component {} to implementation {}",
                            c, opt_info.id,
                        );
                        new_gs_mut.copy(self.get_node_annos(), gs.as_ref())?;
                        true
                    } else {
                        false
                    };
                    if converted {
                        // insert into components map
                        info!(
                            "finished conversion of component {} to implementation {}",
                            c, opt_info.id,
                        );
                        self.components.insert(c.clone(), Some(new_gs.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a read-only graph storage copy for the given component `c`.
    pub fn get_graphstorage(&self, c: &Component<CT>) -> Option<Arc<dyn GraphStorage>> {
        // get and return the reference to the entry if loaded
        let entry: &Arc<dyn GraphStorage> = self.components.get(c)?.as_ref()?;
        Some(entry.clone())
    }

    /// Get a read-only graph storage reference for the given component `c`.
    pub fn get_graphstorage_as_ref<'a>(
        &'a self,
        c: &Component<CT>,
    ) -> Option<&'a dyn GraphStorage> {
        // get and return the reference to the entry if loaded
        let entry: &Arc<dyn GraphStorage> = self.components.get(c)?.as_ref()?;
        Some(entry.as_ref())
    }

    /// Get a read-only reference to the node annotations of this graph
    pub fn get_node_annos(&self) -> &dyn NodeAnnotationStorage {
        self.node_annos.as_ref()
    }

    /// Get a mutable reference to the node annotations of this graph
    pub fn get_node_annos_mut(&mut self) -> &mut dyn NodeAnnotationStorage {
        self.node_annos.as_mut()
    }

    /// Returns all components of the graph given an optional type (`ctype`) and `name`.
    /// This allows to filter which components to receive.
    /// If you want to retrieve all components, use `None` as value for both arguments.
    pub fn get_all_components(&self, ctype: Option<CT>, name: Option<&str>) -> Vec<Component<CT>> {
        if let (Some(ctype), Some(name)) = (&ctype, name) {
            // lookup component from sorted map
            let mut result: Vec<_> = Vec::new();
            let ckey = Component::new(ctype.clone(), SmartString::default(), name.into());

            for (c, _) in self.components.range(ckey..) {
                if c.name != name || &c.get_type() != ctype {
                    break;
                }
                result.push(c.clone());
            }
            result
        } else if let Some(ctype) = &ctype {
            // lookup component from sorted map
            let mut result: Vec<_> = Vec::new();
            let ckey = Component::new(
                ctype.clone(),
                SmartString::default(),
                SmartString::default(),
            );

            for (c, _) in self.components.range(ckey..) {
                if &c.get_type() != ctype {
                    break;
                }
                result.push(c.clone());
            }
            result
        } else {
            // filter all entries
            let filtered_components = self
                .components
                .keys()
                .filter(move |c| {
                    if let Some(ctype) = ctype.clone() {
                        if ctype != c.get_type() {
                            return false;
                        }
                    }
                    if let Some(name) = name {
                        if name != c.name {
                            return false;
                        }
                    }
                    true
                })
                .cloned();
            filtered_components.collect()
        }
    }
}

#[cfg(test)]
mod tests;
