use crate::annis::db::graphstorage::adjacencylist::AdjacencyListStorage;
use crate::annis::db::graphstorage::registry;
use crate::annis::db::graphstorage::union::UnionEdgeContainer;
use crate::annis::db::graphstorage::EdgeContainer;
use crate::annis::db::graphstorage::{GraphStorage, WriteableGraphStorage};
use crate::annis::db::update::{GraphUpdate, UpdateEvent};
use crate::annis::dfs::CycleSafeDFS;
use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, Annotation, Component, ComponentType, Edge, NodeID};
use crate::annis::util::disk_collections::{DiskMap, EvictionStrategy};
use crate::malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::iter::FromIterator;
use std::ops::Bound::Included;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;
use tempfile;

pub mod annostorage;
pub mod aql;
pub mod corpusstorage;
#[cfg(test)]
pub mod example_generator;
pub mod exec;
pub mod graphstorage;
mod plan;
pub mod query;
pub mod relannis;
pub mod sort_matches;
pub mod token_helper;
pub mod update;

pub use annostorage::AnnotationStorage;

pub const ANNIS_NS: &str = "annis";
pub const NODE_NAME: &str = "node_name";
pub const TOK: &str = "tok";
pub const NODE_TYPE: &str = "node_type";

lazy_static! {
    pub static ref DEFAULT_ANNO_KEY: Arc<AnnoKey> = Arc::from(AnnoKey::default());
    pub static ref NODE_NAME_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.to_owned(),
        name: NODE_NAME.to_owned(),
    });
    pub static ref TOKEN_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
            ns: ANNIS_NS.to_owned(),
            name: TOK.to_owned(),
    });
    /// Return an annotation key which is used for the special `annis::node_type` annotation which every node must have to mark its existance.
    pub static ref NODE_TYPE_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.to_owned(),
        name: NODE_TYPE.to_owned(),
    });
}

/// A match is the result of a query on an annotation storage.
#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Match {
    node: NodeID,
    /// The qualified annotation name.
    anno_key: Arc<AnnoKey>,
}

impl Match {
    /// Get the node identifier this match refers to.
    pub fn get_node(&self) -> NodeID {
        self.node
    }

    /// Extract the annotation for this match . The annotation value
    /// is retrieved from the `graph` given as argument.
    pub fn extract_annotation(&self, graph: &Graph) -> Option<Annotation> {
        let val = graph
            .node_annos
            .get_value_for_item(&self.node, &self.anno_key)?
            .to_owned();
        Some(Annotation {
            key: self.anno_key.as_ref().clone(),
            val: val.to_string(),
        })
    }

    /// Returns true if this match is different to all the other matches given as argument.
    ///
    /// A single match is different if the node ID or the annotation key are different.
    pub fn different_to_all(&self, other: &[Match]) -> bool {
        for o in other.iter() {
            if self.node == o.node && self.anno_key == o.anno_key {
                return false;
            }
        }
        true
    }

    /// Returns true if this match is different to the other match given as argument.
    ///
    /// A single match is different if the node ID or the annotation key are different.
    pub fn different_to(&self, other: &Match) -> bool {
        self.node != other.node || self.anno_key != other.anno_key
    }
}

impl Into<Match> for (Edge, Arc<AnnoKey>) {
    fn into(self) -> Match {
        Match {
            node: self.0.source,
            anno_key: self.1,
        }
    }
}

impl Into<Match> for (NodeID, Arc<AnnoKey>) {
    fn into(self) -> Match {
        Match {
            node: self.0,
            anno_key: self.1,
        }
    }
}

#[derive(Clone)]
pub enum ValueSearch<T> {
    Any,
    Some(T),
    NotSome(T),
}

impl<T> From<Option<T>> for ValueSearch<T> {
    fn from(orig: Option<T>) -> ValueSearch<T> {
        match orig {
            None => ValueSearch::Any,
            Some(v) => ValueSearch::Some(v),
        }
    }
}

impl<T> ValueSearch<T> {
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ValueSearch<U> {
        match self {
            ValueSearch::Any => ValueSearch::Any,
            ValueSearch::Some(v) => ValueSearch::Some(f(v)),
            ValueSearch::NotSome(v) => ValueSearch::NotSome(f(v)),
        }
    }

    #[inline]
    pub fn as_ref(&self) -> ValueSearch<&T> {
        match *self {
            ValueSearch::Any => ValueSearch::Any,
            ValueSearch::Some(ref v) => ValueSearch::Some(v),
            ValueSearch::NotSome(ref v) => ValueSearch::NotSome(v),
        }
    }
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
    node_annos: Box<dyn AnnotationStorage<NodeID>>,

    location: Option<PathBuf>,

    components: BTreeMap<Component, Option<Arc<dyn GraphStorage>>>,
    current_change_id: u64,

    background_persistance: Arc<Mutex<()>>,

    cached_size: Mutex<Option<usize>>,

    disk_based: bool,
}

impl MallocSizeOf for Graph {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut size = self.node_annos.size_of(ops);

        for c in self.components.keys() {
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

        size
    }
}

fn load_component_from_disk(component_path: Option<PathBuf>) -> Result<Arc<dyn GraphStorage>> {
    let cpath = component_path.ok_or("Can't load component with empty path")?;

    // load component into memory
    let impl_path = PathBuf::from(&cpath).join("impl.cfg");
    let mut f_impl = std::fs::File::open(impl_path)?;
    let mut impl_name = String::new();
    f_impl.read_to_string(&mut impl_name)?;


    let gs = registry::deserialize(&impl_name, &cpath)?;

    Ok(gs)
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
    p
}

impl Graph {
    /// Create a new and empty instance without any location on the disk.
    fn new(disk_based: bool) -> Result<Graph> {
        let node_annos: Box<dyn AnnotationStorage<NodeID>> = if disk_based {
            Box::new(annostorage::ondisk::AnnoStorageImpl::new(None)?)
        } else {
            Box::new(annostorage::inmemory::AnnoStorageImpl::<NodeID>::new())
        };

        Ok(Graph {
            node_annos,
            components: BTreeMap::new(),

            location: None,

            current_change_id: 0,

            background_persistance: Arc::new(Mutex::new(())),
            cached_size: Mutex::new(None),

            disk_based,
        })
    }

    /// Create a new instance without any location on the disk but with the default graph storage components
    /// (Coverage, Order, LeftToken, RightToken, PartOf).
    fn with_default_graphstorages(disk_based: bool) -> Result<Graph> {
        let mut db = Graph::new(disk_based)?;
        db.get_or_create_writable(&Component {
            ctype: ComponentType::Coverage,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        })?;
        db.get_or_create_writable(&Component {
            ctype: ComponentType::Ordering,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        })?;
        db.get_or_create_writable(&Component {
            ctype: ComponentType::LeftToken,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        })?;
        db.get_or_create_writable(&Component {
            ctype: ComponentType::RightToken,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        })?;
        db.get_or_create_writable(&Component {
            ctype: ComponentType::PartOf,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        })?;

        Ok(db)
    }

    fn set_location(&mut self, location: &Path) -> Result<()> {
        self.location = Some(PathBuf::from(location));

        Ok(())
    }

    /// Clear the graph content.
    /// This removes all node annotations, edges and knowledge about components.
    fn clear(&mut self) {
        self.reset_cached_size();
        self.node_annos = Box::new(annostorage::inmemory::AnnoStorageImpl::new());
        self.components.clear();
    }

    /// Load the graph from an external location.
    /// This sets the location of this instance to the given location.
    ///
    /// * `location` - The path on the disk
    /// * `preload` - If `true`, all components are loaded from disk into main memory.
    fn load_from(&mut self, location: &Path, preload: bool) -> Result<()> {
        info!("Loading corpus from {}", location.to_string_lossy());
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

        let ondisk_subdirectory = dir2load.join(annostorage::ondisk::SUBFOLDER_NAME);
        if ondisk_subdirectory.exists() && ondisk_subdirectory.is_dir() {
            self.disk_based = true;
            // directly load the on disk storage from the given folder to avoid having a temporary directory
            let node_annos_tmp =
                annostorage::ondisk::AnnoStorageImpl::new(Some(ondisk_subdirectory))?;
            self.node_annos = Box::new(node_annos_tmp);
        } else {
            // assume a main memory implementation
            self.disk_based = false;
            let mut node_annos_tmp: annostorage::inmemory::AnnoStorageImpl<NodeID> =
                annostorage::inmemory::AnnoStorageImpl::new();
            node_annos_tmp.load_annotations_from(&dir2load)?;
            self.node_annos = Box::new(node_annos_tmp);
        }

        let log_path = dir2load.join("update_log.bin");

        let logfile_exists = log_path.exists() && log_path.is_file();

        self.find_components_from_disk(&dir2load)?;

        // If backup is active or a write log exists, always  a pre-load to get the complete corpus.
        if preload | logfile_exists | backup_was_loaded {
            self.ensure_loaded_all()?;
        }

        if logfile_exists {
            // apply any outstanding log file updates
            let log_reader = std::fs::File::open(&log_path)?;
            let mut update = bincode::deserialize_from(log_reader)?;
            self.apply_update_in_memory(&mut update, |_| {})?;
        } else {
            self.current_change_id = 0;
        }

        if backup_was_loaded {
            // save the current corpus under the actual location
            self.save_to(&location.join("current"))?;
            // rename backup folder (renaming is atomic and deleting could leave an incomplete backup folder on disk)
            let tmp_dir = tempfile::Builder::new()
                .prefix("temporary-graphannis-backup")
                .tempdir_in(location)?;
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

        self.node_annos.save_annotations_to(&location)?;

        for (c, e) in &self.components {
            if let Some(ref data) = *e {
                let dir = PathBuf::from(&location).join(component_to_relative_path(c));
                std::fs::create_dir_all(&dir)?;
                
                let impl_name = registry::serialize(&data, &dir)?;

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
        self.internal_save(&location.join("current"))
    }

    /// Save the current database at a new `location` and remember it as new internal location.
    fn persist_to(&mut self, location: &Path) -> Result<()> {
        self.set_location(location)?;
        self.internal_save(&location.join("current"))
    }

    fn get_cached_node_id_from_name(
        &self,
        node_name: String,
        cache: &mut DiskMap<String, Option<NodeID>>,
    ) -> Result<Option<NodeID>> {
        if let Some(id) = cache.try_get(&node_name)? {
            Ok(id)
        } else {
            let id = self.get_node_id_from_name(&node_name);
            cache.insert(node_name, id.clone())?;
            Ok(id)
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn apply_update_in_memory<F>(&mut self, u: &mut GraphUpdate, progress_callback: F) -> Result<()>
    where
        F: Fn(&str) -> (),
    {
        self.reset_cached_size();

        let mut invalid_nodes: DiskMap<NodeID, bool> =
            DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;
        // Calculating the invalid nodes adds additional computational overhead. If there are no nodes yet in the graph,
        // we already know that all new nodes are invalid and don't need calculate the invalid ones.
        let calculate_invalid_nodes = !self.node_annos.is_empty();

        let all_components = self.get_all_components(None, None);

        let mut text_coverage_components = FxHashSet::default();
        text_coverage_components
            .extend(self.get_all_components(Some(ComponentType::Dominance), Some("")));
        text_coverage_components
            .extend(self.get_all_components(Some(ComponentType::Coverage), None));

        // Cache the expensive mapping of node names to IDs
        let mut node_ids: DiskMap<String, Option<NodeID>> =
            DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;
        // Iterate once over all changes in the same order as the updates have been added
        let mut nr_updates = 0;
        for (id, change) in u.iter()? {
            trace!("applying event {:?}", &change);
            match change {
                UpdateEvent::AddNode {
                    node_name,
                    node_type,
                } => {
                    let existing_node_id =
                        self.get_cached_node_id_from_name(node_name.clone(), &mut node_ids)?;
                    // only add node if it does not exist yet
                    if existing_node_id.is_none() {
                        let new_node_id: NodeID =
                            if let Some(id) = self.node_annos.get_largest_item() {
                                id + 1
                            } else {
                                0
                            };

                        let new_anno_name = Annotation {
                            key: NODE_NAME_KEY.as_ref().clone(),
                            val: node_name.to_string(),
                        };
                        let new_anno_type = Annotation {
                            key: NODE_TYPE_KEY.as_ref().clone(),
                            val: node_type.to_string(),
                        };

                        // add the new node (with minimum labels)
                        self.node_annos.insert(new_node_id, new_anno_name)?;
                        self.node_annos.insert(new_node_id, new_anno_type)?;

                        // update the internal cache
                        node_ids.insert(node_name, Some(new_node_id))?;

                        if !calculate_invalid_nodes {
                            // All new added nodes need to be marked as invalid
                            invalid_nodes.insert(new_node_id, true)?;
                        }
                    }
                }
                UpdateEvent::DeleteNode { node_name } => {
                    if let Some(existing_node_id) =
                        self.get_cached_node_id_from_name(node_name, &mut node_ids)?
                    {
                        if !calculate_invalid_nodes
                            && invalid_nodes.get(&existing_node_id).is_none()
                        {
                            self.calculate_invalidated_nodes_by_coverage(
                                existing_node_id,
                                &text_coverage_components,
                                &mut invalid_nodes,
                            )?;
                        }

                        // delete all annotations
                        {
                            for a in self.node_annos.get_annotations_for_item(&existing_node_id) {
                                self.node_annos
                                    .remove_annotation_for_item(&existing_node_id, &a.key)?;
                            }
                        }
                        // delete all edges pointing to this node either as source or target
                        for c in all_components.iter() {
                            if let Ok(gs) = self.get_or_create_writable(c) {
                                gs.delete_node(existing_node_id)?;
                            }
                        }
                    }
                }
                UpdateEvent::AddNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                    anno_value,
                } => {
                    if let Some(existing_node_id) =
                        self.get_cached_node_id_from_name(node_name, &mut node_ids)?
                    {
                        let anno = Annotation {
                            key: AnnoKey {
                                ns: anno_ns.to_string(),
                                name: anno_name.to_string(),
                            },
                            val: anno_value.to_string(),
                        };
                        self.node_annos.insert(existing_node_id, anno)?;
                    }
                }
                UpdateEvent::DeleteNodeLabel {
                    node_name,
                    anno_ns,
                    anno_name,
                } => {
                    if let Some(existing_node_id) =
                        self.get_cached_node_id_from_name(node_name, &mut node_ids)?
                    {
                        let key = AnnoKey {
                            ns: anno_ns.to_string(),
                            name: anno_name.to_string(),
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
                    let source = self.get_cached_node_id_from_name(source_node, &mut node_ids)?;
                    let target = self.get_cached_node_id_from_name(target_node, &mut node_ids)?;
                    // only add edge if both nodes already exist
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer: layer.to_string(),
                                name: component_name.to_string(),
                            };
                            let gs = self.get_or_create_writable(&c)?;
                            gs.add_edge(Edge { source, target })?;

                            if (c.ctype == ComponentType::Dominance
                                || c.ctype == ComponentType::Coverage)
                                && c.name.is_empty()
                            {
                                // might be a new text coverage component
                                text_coverage_components.insert(c.clone());
                            }

                            if calculate_invalid_nodes {
                                if c.ctype == ComponentType::Coverage
                                    || c.ctype == ComponentType::Dominance
                                    || c.ctype == ComponentType::Ordering
                                    || c.ctype == ComponentType::LeftToken
                                    || c.ctype == ComponentType::RightToken
                                {
                                    self.calculate_invalidated_nodes_by_coverage(
                                        source,
                                        &text_coverage_components,
                                        &mut invalid_nodes,
                                    )?;
                                }

                                if c.ctype == ComponentType::Ordering {
                                    self.calculate_invalidated_nodes_by_coverage(
                                        target,
                                        &text_coverage_components,
                                        &mut invalid_nodes,
                                    )?;
                                }
                            }
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
                    let source = self.get_cached_node_id_from_name(source_node, &mut node_ids)?;
                    let target = self.get_cached_node_id_from_name(target_node, &mut node_ids)?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer: layer.to_string(),
                                name: component_name.to_string(),
                            };

                            if calculate_invalid_nodes {
                                if c.ctype == ComponentType::Coverage
                                    || c.ctype == ComponentType::Dominance
                                    || c.ctype == ComponentType::Ordering
                                    || c.ctype == ComponentType::LeftToken
                                    || c.ctype == ComponentType::RightToken
                                {
                                    self.calculate_invalidated_nodes_by_coverage(
                                        source,
                                        &text_coverage_components,
                                        &mut invalid_nodes,
                                    )?;
                                }

                                if c.ctype == ComponentType::Ordering {
                                    self.calculate_invalidated_nodes_by_coverage(
                                        target,
                                        &text_coverage_components,
                                        &mut invalid_nodes,
                                    )?;
                                }
                            }

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
                    let source = self.get_cached_node_id_from_name(source_node, &mut node_ids)?;
                    let target = self.get_cached_node_id_from_name(target_node, &mut node_ids)?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer: layer.to_string(),
                                name: component_name.to_string(),
                            };
                            let gs = self.get_or_create_writable(&c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(source, target, 1, Included(1)) {
                                let anno = Annotation {
                                    key: AnnoKey {
                                        ns: anno_ns.to_string(),
                                        name: anno_name.to_string(),
                                    },
                                    val: anno_value.to_string(),
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
                    let source = self.get_cached_node_id_from_name(source_node, &mut node_ids)?;
                    let target = self.get_cached_node_id_from_name(target_node, &mut node_ids)?;
                    if let (Some(source), Some(target)) = (source, target) {
                        if let Ok(ctype) = ComponentType::from_str(&component_type) {
                            let c = Component {
                                ctype,
                                layer: layer.to_string(),
                                name: component_name.to_string(),
                            };
                            let gs = self.get_or_create_writable(&c)?;
                            // only add label if the edge already exists
                            let e = Edge { source, target };
                            if gs.is_connected(source, target, 1, Included(1)) {
                                let key = AnnoKey {
                                    ns: anno_ns.to_string(),
                                    name: anno_name.to_string(),
                                };
                                gs.delete_edge_annotation(&e, &key)?;
                            }
                        }
                    }
                }
            } // end match update entry type
            self.current_change_id = id;

            nr_updates += 1;
            if nr_updates % 100_000 == 0 {
                progress_callback(&format!("applied {} atomic updates", nr_updates));
            }
        } // end for each consistent update entry

        // Re-index the inherited coverage component.
        // To make this operation fast, we need to optimize the order component first
        let order_component = Component {
            ctype: ComponentType::Ordering,
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        };
        let order_stats_exist = self
            .get_graphstorage(&order_component)
            .map(|gs_order| gs_order.get_statistics().is_some())
            .unwrap_or(false);
        if !order_stats_exist {
            self.calculate_component_statistics(&order_component)?;
        }
        self.optimize_impl(&order_component)?;
        if let Some(gs_order) = self.get_graphstorage(&order_component) {
            progress_callback("re-indexing the inherited coverage edges");

            invalid_nodes.compact()?;
            self.reindex_inherited_coverage(invalid_nodes, gs_order)?;
        }

        Ok(())
    }

    fn calculate_invalidated_nodes_by_coverage(
        &self,
        node: NodeID,
        text_coverage_components: &FxHashSet<Component>,
        invalid_nodes: &mut DiskMap<NodeID, bool>,
    ) -> Result<()> {
        let containers: Vec<&dyn EdgeContainer> = text_coverage_components
            .iter()
            .filter_map(|c| self.get_graphstorage_as_ref(c))
            .map(|gs| gs.as_edgecontainer())
            .collect();

        let union = UnionEdgeContainer::new(containers);

        let dfs = CycleSafeDFS::new_inverse(&union, node, 0, usize::max_value());
        for step in dfs {
            invalid_nodes.insert(step.node, true)?;
        }

        Ok(())
    }

    fn reindex_inherited_coverage(
        &mut self,
        invalid_nodes: DiskMap<NodeID, bool>,
        gs_order: Arc<dyn GraphStorage>,
    ) -> Result<()> {
        {
            // remove existing left/right token edges for the invalidated nodes
            let gs_left = self.get_or_create_writable(&Component {
                ctype: ComponentType::LeftToken,
                name: "".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;

            for (n, _) in invalid_nodes.iter() {
                gs_left.delete_node(n)?;
            }

            let gs_right = self.get_or_create_writable(&Component {
                ctype: ComponentType::RightToken,
                name: "".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;

            for (n, _) in invalid_nodes.iter() {
                gs_right.delete_node(n)?;
            }

            let gs_cov = self.get_or_create_writable(&Component {
                ctype: ComponentType::Coverage,
                name: "inherited-coverage".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;
            for (n, _) in invalid_nodes.iter() {
                gs_cov.delete_node(n)?;
            }
        }

        let all_cov_components = self.get_all_components(Some(ComponentType::Coverage), None);
        let all_dom_gs: Vec<Arc<dyn GraphStorage>> = self
            .get_all_components(Some(ComponentType::Dominance), Some(""))
            .into_iter()
            .filter_map(|c| self.get_graphstorage(&c))
            .collect();
        {
            // go over each node and calculate the left-most and right-most token

            let all_cov_gs: Vec<Arc<dyn GraphStorage>> = all_cov_components
                .iter()
                .filter_map(|c| self.get_graphstorage(c))
                .collect();

            for (n, _) in invalid_nodes.iter() {
                self.calculate_token_alignment(
                    n,
                    ComponentType::LeftToken,
                    gs_order.as_ref(),
                    &all_cov_gs,
                    &all_dom_gs,
                )?;
                self.calculate_token_alignment(
                    n,
                    ComponentType::RightToken,
                    gs_order.as_ref(),
                    &all_cov_gs,
                    &all_dom_gs,
                )?;
            }
        }

        for (n, _) in invalid_nodes.iter() {
            self.calculate_inherited_coverage_edges(n, &all_cov_components, &all_dom_gs)?;
        }

        Ok(())
    }

    fn calculate_inherited_coverage_edges(
        &mut self,
        n: NodeID,
        all_cov_components: &[Component],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> Result<FxHashSet<NodeID>> {
        let mut covered_token = FxHashSet::default();
        for c in all_cov_components.iter() {
            if let Some(gs) = self.get_graphstorage_as_ref(c) {
                covered_token.extend(gs.find_connected(n, 1, std::ops::Bound::Included(1)));
            }
        }

        if covered_token.is_empty() {
            if self.node_annos.get_value_for_item(&n, &TOKEN_KEY).is_some() {
                covered_token.insert(n);
            } else {
                // recursivly get the covered token from all children connected by a dominance relation
                for dom_gs in all_dom_gs {
                    for out in dom_gs.get_outgoing_edges(n) {
                        covered_token.extend(self.calculate_inherited_coverage_edges(
                            out,
                            all_cov_components,
                            all_dom_gs,
                        )?);
                    }
                }
            }
        }

        if let Ok(gs_cov) = self.get_or_create_writable(&Component {
            ctype: ComponentType::Coverage,
            name: "inherited-coverage".to_owned(),
            layer: ANNIS_NS.to_owned(),
        }) {
            for t in covered_token.iter() {
                gs_cov.add_edge(Edge {
                    source: n,
                    target: *t,
                })?;
            }
        }

        Ok(covered_token)
    }

    fn calculate_token_alignment(
        &mut self,
        n: NodeID,
        ctype: ComponentType,
        gs_order: &dyn GraphStorage,
        all_cov_gs: &[Arc<dyn GraphStorage>],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> Result<Option<NodeID>> {
        let alignment_component = Component {
            ctype: ctype.clone(),
            name: "".to_owned(),
            layer: ANNIS_NS.to_owned(),
        };

        // if this is a token, return the token itself
        if self.node_annos.get_value_for_item(&n, &TOKEN_KEY).is_some() {
            // also check if this is an actually token and not only a segmentation
            let mut is_token = true;
            for gs_coverage in all_cov_gs.iter() {
                if gs_coverage.get_outgoing_edges(n).next().is_some() {
                    is_token = false;
                    break;
                }
            }
            if is_token {
                return Ok(Some(n));
            }
        }

        // if the node already has a left/right token, just return this value
        if let Some(alignment_gs) = self.get_graphstorage_as_ref(&alignment_component) {
            let existing = alignment_gs.get_outgoing_edges(n).next();
            if let Some(existing) = existing {
                return Ok(Some(existing));
            }
        }

        // recursively get all candidate token by iterating over text-coverage edges
        let mut candidates = FxHashSet::default();

        for gs_for_component in all_dom_gs.iter().chain(all_cov_gs.iter()) {
            for target in gs_for_component.get_outgoing_edges(n) {
                if let Some(candidate_for_target) = self.calculate_token_alignment(
                    target,
                    ctype.clone(),
                    gs_order,
                    all_cov_gs,
                    all_dom_gs,
                )? {
                    candidates.insert(candidate_for_target);
                } else {
                    return Ok(None);
                }
            }
        }

        // order the candidate token by their position in the order chain
        let mut candidates = Vec::from_iter(candidates.into_iter());
        candidates.sort_unstable_by(move |a, b| {
            if a == b {
                return std::cmp::Ordering::Equal;
            }
            if gs_order.is_connected(*a, *b, 1, std::ops::Bound::Unbounded) {
                return std::cmp::Ordering::Less;
            } else if gs_order.is_connected(*b, *a, 1, std::ops::Bound::Unbounded) {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Equal
        });

        // add edge to left/right most candidate token
        let t = if ctype == ComponentType::RightToken {
            candidates.last()
        } else {
            candidates.first()
        };
        if let Some(t) = t {
            let gs = self.get_or_create_writable(&alignment_component)?;
            let e = Edge {
                source: n,
                target: *t,
            };
            gs.add_edge(e)?;

            Ok(Some(*t))
        } else {
            Ok(None)
        }
    }

    /// Apply a sequence of updates (`u` parameter) to this graph.
    /// If the graph has a location on the disk, the changes are persisted.
    fn apply_update<F>(&mut self, u: &mut GraphUpdate, progress_callback: F) -> Result<()>
    where
        F: Fn(&str) -> (),
    {
        progress_callback("applying list of atomic updates");

        // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
        self.ensure_loaded_all()?;

        let result = self.apply_update_in_memory(u, &progress_callback);

        progress_callback("memory updates completed, persisting updates to disk");

        if let Some(location) = self.location.clone() {
            trace!("output location for persisting updates is {:?}", location);
            if result.is_ok() {
                let current_path = location.join("current");
                // make sure the output path exits
                std::fs::create_dir_all(&current_path)?;

                // If successfull write log
                let log_path = location.join("update_log.bin");

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
            // Acquire lock, so that only one thread can write background data at the same time
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
        self.reset_cached_size();

        // move the old entry into the ownership of this function
        let entry = self.components.remove(c);
        // component exists?
        if let Some(gs_opt) = entry {
            let mut loaded_comp: Arc<dyn GraphStorage> = if let Some(gs_opt) = gs_opt {
                gs_opt
            } else {
                load_component_from_disk(self.component_path(c))?
            };

            // copy to writable implementation if needed
            let is_writable = {
                Arc::get_mut(&mut loaded_comp)
                    .ok_or_else(|| format!("Could not get mutable reference for component {}", c))?
                    .as_writeable()
                    .is_some()
            };

            let loaded_comp = if is_writable {
                loaded_comp
            } else {
                let mut gs_copy: AdjacencyListStorage = registry::create_writeable()?;
                gs_copy.copy(&self, loaded_comp.as_ref())?;
                Arc::from(gs_copy)
            };

            // (re-)insert the component into map again
            self.components.insert(c.clone(), Some(loaded_comp));
        }
        Ok(())
    }

    fn calculate_component_statistics(&mut self, c: &Component) -> Result<()> {
        self.reset_cached_size();

        let mut result: Result<()> = Ok(());
        let mut entry = self
            .components
            .remove(c)
            .ok_or_else(|| format!("Component {} is missing", c.clone()))?;
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
        result
    }

    fn get_or_create_writable(&mut self, c: &Component) -> Result<&mut dyn WriteableGraphStorage> {
        self.reset_cached_size();

        if self.components.contains_key(c) {
            // make sure the component is actually writable and loaded
            self.insert_or_copy_writeable(c)?;
        } else {
            let w = registry::create_writeable()?;

            self.components.insert(c.clone(), Some(Arc::from(w)));
        }

        // get and return the reference to the entry
        let entry: &mut Arc<dyn GraphStorage> = self
            .components
            .get_mut(c)
            .ok_or_else(|| format!("Could not get mutable reference for component {}", c))?
            .as_mut()
            .ok_or_else(|| {
                format!(
                    "Could not get mutable reference to optional value for component {}",
                    c
                )
            })?;
        let gs_mut_ref: &mut dyn GraphStorage = Arc::get_mut(entry)
            .ok_or_else(|| format!("Could not get mutable reference for component {}", c))?;
        Ok(gs_mut_ref.as_writeable().ok_or("Invalid type")?)
    }

    fn is_loaded(&self, c: &Component) -> bool {
        let entry: Option<&Option<Arc<dyn GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if gs_opt.is_some() {
                return true;
            }
        }
        false
    }

    fn ensure_loaded_all(&mut self) -> Result<()> {
        let mut components_to_load: Vec<Component> = Vec::with_capacity(self.components.len());

        // colllect all missing components
        for (c, gs) in &self.components {
            if gs.is_none() {
                components_to_load.push(c.clone());
            }
        }

        self.reset_cached_size();

        // load missing components in parallel
        let loaded_components: Vec<(Component, Result<Arc<dyn GraphStorage>>)> = components_to_load
            .into_par_iter()
            .map(|c| {
                info!("Loading component {} from disk", c);
                let cpath = self.component_path(&c);
                let loaded_component = load_component_from_disk(cpath);
                (c, loaded_component)
            })
            .collect();

        // insert all the loaded components
        for (c, gs) in loaded_components {
            let gs = gs?;
            self.components.insert(c, Some(gs));
        }
        Ok(())
    }

    fn ensure_loaded(&mut self, c: &Component) -> Result<()> {
        // get and return the reference to the entry if loaded
        let entry: Option<Option<Arc<dyn GraphStorage>>> = self.components.remove(c);
        if let Some(gs_opt) = entry {
            let loaded: Arc<dyn GraphStorage> = if let Some(gs_opt) = gs_opt {
                gs_opt
            } else {
                self.reset_cached_size();
                info!("Loading component {} from disk", c);
                load_component_from_disk(self.component_path(c))?
            };

            self.components.insert(c.clone(), Some(loaded));
        }
        Ok(())
    }

    fn optimize_impl(&mut self, c: &Component) -> Result<()> {
        if let Some(gs) = self.get_graphstorage(c) {
            if let Some(stats) = gs.get_statistics() {
                let opt_info = registry::get_optimal_impl_heuristic(self, stats);

                // convert if necessary
                if opt_info.id != gs.serialization_id() {
                    let mut new_gs = registry::create_from_info(&opt_info);
                    let converted = if let Some(new_gs_mut) = Arc::get_mut(&mut new_gs) {
                        new_gs_mut.copy(self, gs.as_ref())?;
                        true
                    } else {
                        false
                    };
                    if converted {
                        self.reset_cached_size();
                        // insert into components map
                        info!(
                            "converted component {} to implementation {}",
                            c, opt_info.id,
                        );
                        self.components.insert(c.clone(), Some(new_gs.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    fn get_node_id_from_name(&self, node_name: &str) -> Option<NodeID> {
        let mut all_nodes_with_anno = self.node_annos.exact_anno_search(
            Some(&ANNIS_NS.to_owned()),
            &NODE_NAME.to_owned(),
            Some(node_name).into(),
        );
        if let Some(m) = all_nodes_with_anno.next() {
            return Some(m.node);
        }
        None
    }

    /// Get a read-only graph storage reference for the given component `c`.
    pub fn get_graphstorage(&self, c: &Component) -> Option<Arc<dyn GraphStorage>> {
        // get and return the reference to the entry if loaded
        let entry: Option<&Option<Arc<dyn GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if let Some(ref impl_type) = *gs_opt {
                return Some(impl_type.clone());
            }
        }
        None
    }

    /// Get a read-only reference to the node annotations of this graph
    pub fn get_node_annos(&self) -> &dyn AnnotationStorage<NodeID> {
        self.node_annos.as_ref()
    }

    fn get_graphstorage_as_ref<'a>(&'a self, c: &Component) -> Option<&'a dyn GraphStorage> {
        // get and return the reference to the entry if loaded
        let entry: Option<&Option<Arc<dyn GraphStorage>>> = self.components.get(c);
        if let Some(gs_opt) = entry {
            if let Some(ref impl_type) = *gs_opt {
                return Some(impl_type.as_ref());
            }
        }
        None
    }

    /// Returns all components of the graph given an optional type (`ctype`) and `name`.
    /// This allows to filter which components to receive.
    /// If you want to retrieve all components, use `None` as value for both arguments.
    pub fn get_all_components(
        &self,
        ctype: Option<ComponentType>,
        name: Option<&str>,
    ) -> Vec<Component> {
        if let (Some(ctype), Some(name)) = (&ctype, name) {
            // lookup component from sorted map
            let mut result: Vec<Component> = Vec::new();
            let ckey = Component {
                ctype: ctype.clone(),
                name: String::from(name),
                layer: String::default(),
            };

            for (c, _) in self.components.range(ckey..) {
                if c.name != name || c.ctype != *ctype {
                    break;
                }
                result.push(c.clone());
            }
            result
        } else if let Some(ctype) = &ctype {
            // lookup component from sorted map
            let mut result: Vec<Component> = Vec::new();
            let ckey = Component {
                ctype: ctype.clone(),
                name: String::default(),
                layer: String::default(),
            };

            for (c, _) in self.components.range(ckey..) {
                if c.ctype != *ctype {
                    break;
                }
                result.push(c.clone());
            }
            result
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
                        true
                    });
            filtered_components.collect()
        }
    }

    fn size_of_cached(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut lock = self.cached_size.lock().unwrap();
        let cached_size: &mut Option<usize> = &mut *lock;
        if let Some(cached) = cached_size {
            return *cached;
        }
        let calculated_size = self.size_of(ops);
        *cached_size = Some(calculated_size);
        calculated_size
    }

    fn reset_cached_size(&self) {
        let mut lock = self.cached_size.lock().unwrap();
        let cached_size: &mut Option<usize> = &mut *lock;
        *cached_size = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annis::types::{AnnoKey, Annotation, ComponentType, Edge};

    #[test]
    fn create_writeable_gs() {
        let mut db = Graph::new(false).unwrap();

        let anno_key = AnnoKey {
            ns: "test".to_owned(),
            name: "edge_anno".to_owned(),
        };
        let anno_val = "testValue".to_owned();

        let gs: &mut dyn WriteableGraphStorage = db
            .get_or_create_writable(&Component {
                ctype: ComponentType::Pointing,
                layer: String::from("test"),
                name: String::from("dep"),
            })
            .unwrap();

        gs.add_edge(Edge {
            source: 0,
            target: 1,
        })
        .unwrap();

        gs.add_edge_annotation(
            Edge {
                source: 0,
                target: 1,
            },
            Annotation {
                key: anno_key,
                val: anno_val,
            },
        )
        .unwrap();
    }
}
