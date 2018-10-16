use annis::db;
use annis::db::annostorage::AnnoStorage;
use annis::db::aql;
use annis::db::aql::operators;
use annis::db::exec::nodesearch::NodeSearchSpec;
use annis::db::plan::ExecutionPlan;
use annis::db::query;
use annis::db::query::conjunction::Conjunction;
use annis::db::query::disjunction::Disjunction;
use annis::db::relannis;
use annis::db::{AnnotationStorage, Graph, Match, ANNIS_NS, NODE_TYPE};
use annis::errors::ErrorKind;
use annis::errors::*;
use annis::types::AnnoKey;
use annis::types::{
    Annotation, Component, ComponentType, CountExtra, Edge, FrequencyTable, NodeID,
    QueryAttributeDescription,
};
use annis::util;
use annis::util::memory_estimation;
use fs2::FileExt;
use linked_hash_map::LinkedHashMap;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std;
use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use update::GraphUpdate;

use rustc_hash::FxHashMap;

use rand;
use rand::Rng;
use rayon::prelude::*;
use sys_info;

enum CacheEntry {
    Loaded(Graph),
    NotLoaded,
}

/// Indicates if the corpus is partially or fully loaded into the main memory cache.
#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
pub enum LoadStatus {
    /// Corpus is not loaded into main memory at all.
    NotLoaded,
    /// Corpus is partially loaded and is estimated to use the given amount of main memory in bytes.
    /// Partially means that the node annotations are and optionally some graph storages are loaded.
    PartiallyLoaded(usize),
    /// Corpus is fully loaded (node annotation information and all graph storages) and is estimated to use the given amount of main memory in bytes.
    FullyLoaded(usize),
}

/// Information about a single graph storage of the corpus.
#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub struct GraphStorageInfo {
    /// The component this graph storage belongs to.
    pub component: Component,
    /// Indicates if the graph storage is loaded or not.
    pub load_status: LoadStatus,
    /// Number of edge annotations in this graph storage.
    pub number_of_annotations: usize,
}

impl fmt::Display for GraphStorageInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Component {}: {} annnotations",
            self.component, self.number_of_annotations
        )?;
        match self.load_status {
            LoadStatus::NotLoaded => writeln!(f, "Not Loaded")?,
            LoadStatus::PartiallyLoaded(memory_size) => {
                writeln!(f, "Status: {:?}", "partially loaded")?;
                writeln!(
                    f,
                    "Memory: {:.2} MB",
                    memory_size as f64 / (1024 * 1024) as f64
                )?;
            }
            LoadStatus::FullyLoaded(memory_size) => {
                writeln!(f, "Status: {:?}", "fully loaded")?;
                writeln!(
                    f,
                    "Memory: {:.2} MB",
                    memory_size as f64 / (1024 * 1024) as f64
                )?;
            }
        };
        Ok(())
    }
}

/// Information about a corpus that is part of the corpus storage.
#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub struct CorpusInfo {
    /// Name of the corpus.
    pub name: String,
    /// Indicates if the corpus is partially or fully loaded.
    pub load_status: LoadStatus,
    /// A list of descriptions for the graph storages of this corpus.
    pub graphstorages: Vec<GraphStorageInfo>,
}

impl fmt::Display for CorpusInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.load_status {
            LoadStatus::NotLoaded => writeln!(f, "Not Loaded")?,
            LoadStatus::PartiallyLoaded(memory_size) => {
                writeln!(f, "Status: {:?}", "partially loaded")?;
                writeln!(
                    f,
                    "Total memory: {:.2} MB",
                    memory_size as f64 / (1024 * 1024) as f64
                )?;
            }
            LoadStatus::FullyLoaded(memory_size) => {
                writeln!(f, "Status: {:?}", "fully loaded")?;
                writeln!(
                    f,
                    "Total memory: {:.2} MB",
                    memory_size as f64 / (1024 * 1024) as f64
                )?;
            }
        };
        if !self.graphstorages.is_empty() {
            writeln!(f, "------------")?;
            for gs in self.graphstorages.iter() {
                write!(f, "{}", gs)?;
                writeln!(f, "------------")?;
            }
        }
        Ok(())
    }
}

/// Defines the order of results of a `find` query.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum ResultOrder {
    /// Order results by their document name and the the text position of the match.
    Normal,
    /// Inverted the order of `Normal`.
    Inverted,
    /// A random ordering which is **not stable**. Each new query will result in a different order.
    Random,
}

struct PreparationResult<'a> {
    query: Disjunction<'a>,
    db_entry: Arc<RwLock<CacheEntry>>,
}

/// Definition of a single attribute of a frequency query.
#[derive(Debug)]
pub struct FrequencyDefEntry {
    /// The namespace of the annotation from which the attribute value is generated.
    pub ns: Option<String>,
    /// The name of the annotation from which the attribute value is generated.
    pub name: String,
    /// The name of the query node from which the attribute value is generated.
    pub node_ref: String,
}

impl FromStr for FrequencyDefEntry {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<FrequencyDefEntry, Self::Err> {
        let splitted: Vec<&str> = s.splitn(2, ':').collect();
        if splitted.len() != 2 {
            return Err("Frequency definition must consists of two parts: \
                        the referenced node and the annotation name or \"tok\" separated by \":\""
                .into());
        }
        let node_ref = splitted[0];
        let anno_key = util::split_qname(splitted[1]);

        return Ok(FrequencyDefEntry {
            ns: anno_key.0.and_then(|ns| Some(String::from(ns))),
            name: String::from(anno_key.1),
            node_ref: String::from(node_ref),
        });
    }
}

/// An enum over all supported query languages of graphANNIS.
///
/// Currently, only the ANNIS Query Language (AQL) is supported, but this enum allows us to add e.g. a quirks mode for older query language versions
/// or completly new query languages.
#[repr(C)]
pub enum QueryLanguage {
    AQL,
}

/// An enum of all supported input formats of graphANNIS.
#[repr(C)]
pub enum ImportFormat {
    /// Legacy [relANNIS import file format](http://korpling.github.io/ANNIS/doc/dev-annisimportformat.html)
    RelANNIS,
}

/// Different strategies how it is decided when corpora need to be removed from the cache.
pub enum CacheStrategy {
    /// Fixed maximum size of the cache in bytes.
    /// Before and after a new entry is loaded, the cache is cleared to have at maximum this given size.
    /// The loaded entry is always added to the cache, even if the single corpus is larger than the maximum size.
    FixedMaxMemory(usize),
    /// Maximum percent of the current free space/memory available.
    /// E.g. if the percent is 25 and there is 4,5 GB of free memory not used by the cache itself, the cache will use at most 1,125 GB memory.
    /// Cache size is checked before and after a corpus is loaded.
    /// The loaded entry is always added to the cache, even if the single corpus is larger than the maximum size.
    PercentOfFreeMemory(f64),
    /// Stores at most one corpus in the cache.
    OnlyOneCorpus,
}

/// A thread-safe API for managing corpora stored in a common location on the file system.
///
/// Multiple corpora can be part of a corpus storage and they are identified by their unique name.
/// Corpora are loaded from disk into main memory on demand:
/// An internal main memory cache is used to avoid re-loading a recently queried corpus from disk again.
pub struct CorpusStorage {
    db_dir: PathBuf,
    lock_file: File,
    cache_strategy: CacheStrategy,
    corpus_cache: RwLock<LinkedHashMap<String, Arc<RwLock<CacheEntry>>>>,
    query_config: query::Config,
    active_background_workers: Arc<(Mutex<usize>, Condvar)>,
}

impl CorpusStorage {
    /// Create a new instance with a maximum size for the internal corpus cache.
    ///
    /// - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
    /// - `cache_strategy`: A strategy for clearing the cache.
    /// - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
    pub fn with_cache_strategy(
        db_dir: &Path,
        cache_strategy: CacheStrategy,
        use_parallel_joins: bool,
    ) -> Result<CorpusStorage> {
        let query_config = query::Config { use_parallel_joins };

        let active_background_workers = Arc::new((Mutex::new(0), Condvar::new()));
        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            lock_file: create_lockfile_for_directory(db_dir)?,
            cache_strategy,
            corpus_cache: RwLock::new(LinkedHashMap::new()),
            query_config,
            active_background_workers,
        };

        Ok(cs)
    }

    /// Create a new instance with a an automatic determined size of the internal corpus cache.
    ///
    /// Currently, set the maximum cache size to 25% of the available/free memory at construction time.
    /// This behavior chan change in the future.
    ///
    /// - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
    /// - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
    pub fn with_auto_cache_size(db_dir: &Path, use_parallel_joins: bool) -> Result<CorpusStorage> {
        let query_config = query::Config { use_parallel_joins };

        // get the amount of available memory, use a quarter of it per default
        let cache_strategy: CacheStrategy = CacheStrategy::PercentOfFreeMemory(25.0);

        let active_background_workers = Arc::new((Mutex::new(0), Condvar::new()));

        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            lock_file: create_lockfile_for_directory(db_dir)?,
            cache_strategy,
            corpus_cache: RwLock::new(LinkedHashMap::new()),
            query_config: query_config,
            active_background_workers,
        };

        Ok(cs)
    }

    /// List  all available corpora in the corpus storage.
    pub fn list(&self) -> Result<Vec<CorpusInfo>> {
        let names: Vec<String> = self.list_from_disk().unwrap_or_default();
        let mut result: Vec<CorpusInfo> = vec![];

        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

        for n in names {
            if let Ok(corpus_info) = self.create_corpus_info(&n, &mut mem_ops) {
                result.push(corpus_info);
            }
        }

        return Ok(result);
    }

    fn list_from_disk(&self) -> Result<Vec<String>> {
        let mut corpora: Vec<String> = Vec::new();
        for c_dir in self.db_dir.read_dir().chain_err(|| {
            format!(
                "Listing directories from {} failed",
                self.db_dir.to_string_lossy()
            )
        })? {
            let c_dir = c_dir.chain_err(|| {
                format!(
                    "Could not get directory entry of folder {}",
                    self.db_dir.to_string_lossy()
                )
            })?;
            let ftype = c_dir.file_type().chain_err(|| {
                format!(
                    "Could not determine file type for {}",
                    c_dir.path().to_string_lossy()
                )
            })?;
            if ftype.is_dir() {
                let corpus_name = c_dir.file_name().to_string_lossy().to_string();
                corpora.push(corpus_name.clone());
            }
        }
        Ok(corpora)
    }

    fn create_corpus_info(
        &self,
        corpus_name: &str,
        mem_ops: &mut MallocSizeOfOps,
    ) -> Result<CorpusInfo> {
        let cache_entry = self.get_entry(corpus_name)?;
        let lock = cache_entry.read().unwrap();
        let corpus_info: CorpusInfo = match &*lock {
            &CacheEntry::Loaded(ref db) => {
                // check if all components are loaded
                let heap_size = db.size_of(mem_ops);
                let mut load_status = LoadStatus::FullyLoaded(heap_size);

                let mut graphstorages = Vec::new();
                for c in db.get_all_components(None, None) {
                    if let Some(gs) = db.get_graphstorage_as_ref(&c) {
                        graphstorages.push(GraphStorageInfo {
                            component: c.clone(),
                            load_status: LoadStatus::FullyLoaded(gs.size_of(mem_ops)),
                            number_of_annotations: gs.get_anno_storage().number_of_annotations(),
                        });
                    } else {
                        load_status = LoadStatus::PartiallyLoaded(heap_size);
                        graphstorages.push(GraphStorageInfo {
                            component: c.clone(),
                            load_status: LoadStatus::NotLoaded,
                            number_of_annotations: 0,
                        })
                    }
                }

                CorpusInfo {
                    name: corpus_name.to_owned(),
                    load_status,
                    graphstorages,
                }
            }
            &CacheEntry::NotLoaded => CorpusInfo {
                name: corpus_name.to_owned(),
                load_status: LoadStatus::NotLoaded,
                graphstorages: vec![],
            },
        };
        Ok(corpus_info)
    }

    /// Return detailled information about a specific corpus with a given name (`corpus_name`).
    pub fn info(&self, corpus_name: &str) -> Result<CorpusInfo> {
        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);
        self.create_corpus_info(corpus_name, &mut mem_ops)
    }

    fn get_entry(&self, corpus_name: &str) -> Result<Arc<RwLock<CacheEntry>>> {
        let corpus_name = corpus_name.to_string();

        {
            // test with read-only access if corpus is contained in cache
            let cache_lock = self.corpus_cache.read().unwrap();
            let cache = &*cache_lock;
            if let Some(e) = cache.get(&corpus_name) {
                return Ok(e.clone());
            }
        }

        // if not yet available, change to write-lock and insert cache entry
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        let entry = cache
            .entry(corpus_name.clone())
            .or_insert_with(|| Arc::new(RwLock::new(CacheEntry::NotLoaded)));

        return Ok(entry.clone());
    }

    fn load_entry_with_lock(
        &self,
        cache_lock: &mut RwLockWriteGuard<LinkedHashMap<String, Arc<RwLock<CacheEntry>>>>,
        corpus_name: &str,
        create_if_missing: bool,
    ) -> Result<Arc<RwLock<CacheEntry>>> {
        let cache = &mut *cache_lock;

        // if not loaded yet, get write-lock and load entry
        let db_path: PathBuf = [self.db_dir.to_string_lossy().as_ref(), &corpus_name]
            .iter()
            .collect();

        let create_corpus = if db_path.is_dir() {
            false
        } else {
            if create_if_missing {
                true
            } else {
                return Err(ErrorKind::NoSuchCorpus(corpus_name.to_string()).into());
            }
        };

        // make sure the cache is not too large before adding the new corpus
        check_cache_size_and_remove_with_cache(cache, &self.cache_strategy, vec![]);

        let mut db = Graph::new();
        if create_corpus {
            db.persist_to(&db_path)
                .chain_err(|| format!("Could not create corpus with name {}", corpus_name))?;
        } else {
            db.load_from(&db_path, false)?;
        }

        let entry = Arc::new(RwLock::new(CacheEntry::Loaded(db)));
        // first remove entry, than add it: this ensures it is at the end of the linked hash map
        cache.remove(corpus_name);
        cache.insert(String::from(corpus_name), entry.clone());

        check_cache_size_and_remove_with_cache(cache, &self.cache_strategy, vec![corpus_name]);

        return Ok(entry);
    }

    fn get_loaded_entry(
        &self,
        corpus_name: &str,
        create_if_missing: bool,
    ) -> Result<Arc<RwLock<CacheEntry>>> {
        let cache_entry = self.get_entry(corpus_name)?;

        // check if basics (node annotation, strings) of the database are loaded
        let loaded = {
            let lock = cache_entry.read().unwrap();
            match &*lock {
                &CacheEntry::Loaded(_) => true,
                _ => false,
            }
        };

        if loaded {
            return Ok(cache_entry);
        } else {
            let mut cache_lock = self.corpus_cache.write().unwrap();
            return self.load_entry_with_lock(&mut cache_lock, corpus_name, create_if_missing);
        }
    }

    fn get_loaded_entry_with_components(
        &self,
        corpus_name: &str,
        components: Vec<Component>,
    ) -> Result<Arc<RwLock<CacheEntry>>> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let missing_components = {
            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;

            let mut missing: HashSet<Component> = HashSet::new();
            for c in components.into_iter() {
                if !db.is_loaded(&c) {
                    missing.insert(c);
                }
            }
            missing
        };
        if !missing_components.is_empty() {
            // load the needed components
            let mut lock = db_entry.write().unwrap();
            let db = get_write_or_error(&mut lock)?;
            for c in missing_components {
                db.ensure_loaded(&c)?;
            }
        };

        Ok(db_entry)
    }

    fn get_fully_loaded_entry(&self, corpus_name: &str) -> Result<Arc<RwLock<CacheEntry>>> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let missing_components = {
            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;

            let mut missing: HashSet<Component> = HashSet::new();
            for c in db.get_all_components(None, None).into_iter() {
                if !db.is_loaded(&c) {
                    missing.insert(c);
                }
            }
            missing
        };
        if !missing_components.is_empty() {
            // load the needed components
            let mut lock = db_entry.write().unwrap();
            let db = get_write_or_error(&mut lock)?;
            for c in missing_components {
                db.ensure_loaded(&c)?;
            }
        };

        Ok(db_entry)
    }

    /// Import a corpus from an external location on the file system into this corpus storage.
    ///
    /// - `path` - The location on the file system where the corpus data is located.
    /// - `format` - The format in which this corpus data is stored.
    /// - `corpus_name` - Optionally override the name of the new corpus for file formats that already provide a corpus name.
    pub fn import_from_fs(
        &self,
        path: &Path,
        format: ImportFormat,
        corpus_name: Option<String>,
    ) -> Result<String> {
        let (orig_name, mut graph) = match format {
            ImportFormat::RelANNIS => relannis::load(path)?,
        };

        let r = graph.ensure_loaded_all();
        if let Err(e) = r {
            error!(
                "Some error occured when attempting to load components from disk: {:?}",
                e
            );
        }

        let corpus_name = corpus_name.unwrap_or(orig_name);

        let mut db_path = PathBuf::from(&self.db_dir);
        db_path.push(corpus_name.clone());

        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        // make sure the cache is not too large before adding the new corpus
        check_cache_size_and_remove_with_cache(cache, &self.cache_strategy, vec![]);

        // remove any possible old corpus
        let old_entry = cache.remove(&corpus_name);
        if let Some(_) = old_entry {
            if let Err(e) = std::fs::remove_dir_all(db_path.clone()) {
                error!("Error when removing existing files {}", e);
            }
        }

        if let Err(e) = std::fs::create_dir_all(&db_path) {
            error!(
                "Can't create directory {}: {:?}",
                db_path.to_string_lossy(),
                e
            );
        }

        // save to its location
        let save_result = graph.save_to(&db_path);
        if let Err(e) = save_result {
            error!(
                "Can't save corpus to {}: {:?}",
                db_path.to_string_lossy(),
                e
            );
        }

        // make it known to the cache
        cache.insert(
            corpus_name.clone(),
            Arc::new(RwLock::new(CacheEntry::Loaded(graph))),
        );
        check_cache_size_and_remove_with_cache(cache, &self.cache_strategy, vec![&corpus_name]);

        Ok(corpus_name)
    }

    /// Delete a corpus from this corpus storage.
    /// Returns `true` if the corpus was sucessfully deleted and `false` if no such corpus existed.
    pub fn delete(&self, corpus_name: &str) -> Result<bool> {
        let mut db_path = PathBuf::from(&self.db_dir);
        db_path.push(corpus_name);

        let mut cache_lock = self.corpus_cache.write().unwrap();

        let cache = &mut *cache_lock;

        // remove any possible old corpus
        if let Some(db_entry) = cache.remove(corpus_name) {
            // aquire exclusive lock for this cache entry because
            // other queries or background writer might still have access it and need to finish first
            let mut _lock = db_entry.write().unwrap();

            if db_path.is_dir() && db_path.exists() {
                std::fs::remove_dir_all(db_path.clone())
                    .chain_err(|| "Error when removing existing files")?
            }

            return Ok(true);
        } else {
            return Ok(false);
        }
    }

    /// Apply a sequence of updates (`update` parameter) to this graph for a corpus given by the `corpus_name` parameter.
    ///
    /// It is ensured that the update process is atomic and that the changes are persisted to disk if the result is `Ok`.
    pub fn apply_update(&self, corpus_name: &str, update: &mut GraphUpdate) -> Result<()> {
        let db_entry = self
            .get_loaded_entry(corpus_name, true)
            .chain_err(|| format!("Could not get loaded entry for corpus {}", corpus_name))?;
        {
            let mut lock = db_entry.write().unwrap();
            let db: &mut Graph = get_write_or_error(&mut lock)?;

            db.apply_update(update)?;
        }
        // start background thread to persists the results

        let active_background_workers = self.active_background_workers.clone();
        {
            let &(ref lock, ref _cvar) = &*active_background_workers;
            let mut nr_active_background_workers = lock.lock().unwrap();
            *nr_active_background_workers = *nr_active_background_workers + 1;
        }
        thread::spawn(move || {
            trace!("Starting background thread to sync WAL updates");
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                let db: &Graph = db;
                if let Err(e) = db.background_sync_wal_updates() {
                    error!("Can't sync changes in background thread: {:?}", e);
                } else {
                    trace!("Finished background thread to sync WAL updates");
                }
            }
            let &(ref lock, ref cvar) = &*active_background_workers;
            let mut nr_active_background_workers = lock.lock().unwrap();
            *nr_active_background_workers = *nr_active_background_workers - 1;
            cvar.notify_all();
        });

        Ok(())
    }

    fn prepare_query<'a>(
        &self,
        corpus_name: &str,
        query: &'a str,
        query_language: QueryLanguage,
        additional_components: Vec<Component>,
    ) -> Result<PreparationResult<'a>> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;

        // make sure the database is loaded with all necessary components
        let (q, missing_components) = {
            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;

            let q = match query_language {
                QueryLanguage::AQL => aql::parse(query)?,
            };

            let necessary_components = q.necessary_components(db);

            let mut missing: HashSet<Component> =
                HashSet::from_iter(necessary_components.iter().cloned());

            // make sure the additional components are loaded
            missing.extend(additional_components.into_iter());

            // remove all that are already loaded
            for c in necessary_components.iter() {
                if db.get_graphstorage(c).is_some() {
                    missing.remove(c);
                }
            }
            let missing: Vec<Component> = missing.into_iter().collect();
            (q, missing)
        };

        if !missing_components.is_empty() {
            // load the needed components
            {
                let mut lock = db_entry.write().unwrap();
                let db = get_write_or_error(&mut lock)?;
                for c in missing_components {
                    db.ensure_loaded(&c)?;
                }
            }
            self.check_cache_size_and_remove(vec![corpus_name]);
        };

        return Ok(PreparationResult { query: q, db_entry });
    }

    /// Preloads all annotation and graph storages from the disk into a main memory cache.
    pub fn preload(&self, corpus_name: &str) -> Result<()> {
        {
            let db_entry = self.get_loaded_entry(corpus_name, false)?;
            let mut lock = db_entry.write().unwrap();
            let db = get_write_or_error(&mut lock)?;
            db.ensure_loaded_all()?;
        }
        self.check_cache_size_and_remove(vec![corpus_name]);
        return Ok(());
    }

    /// Unloads a corpus from the cache.
    pub fn unload(&self, corpus_name: &str) {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;
        cache.remove(corpus_name);
    }

    /// Update the graph and annotation statistics for a corpus given by `corpus_name`.
    pub fn update_statistics(&self, corpus_name: &str) -> Result<()> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let mut lock = db_entry.write().unwrap();
        let db: &mut Graph = get_write_or_error(&mut lock)?;

        Arc::make_mut(&mut db.node_annos).calculate_statistics();
        for c in db.get_all_components(None, None).into_iter() {
            db.calculate_component_statistics(&c)?;
        }

        // TODO: persist changes

        Ok(())
    }

    /// Parses a `query` and checks if it is valid.
    ///
    /// - `corpus_name` - The name of the corpus the query would be executed on (needed because missing annotation names can be a semantic parser error).
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    ///
    /// Returns `true` if valid and an error with the parser message if invalid.
    pub fn validate_query(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<bool> {
        let prep: PreparationResult =
            self.prepare_query(corpus_name, query, query_language, vec![])?;
        // also get the semantic errors by creating an execution plan on the actual Graph
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;
        return Ok(true);
    }

    /// Returns a string representation of the execution plan for a `query`.
    ///
    /// - `corpus_name` - The name of the corpus to execute the query on.
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    pub fn plan(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<String> {
        let prep = self.prepare_query(corpus_name, query, query_language, vec![])?;

        // accuire read-only lock and plan
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        return Ok(format!("{}", plan));
    }

    /// Count the number of results for a `query`.
    /// - `corpus_name` - The name of the corpus to execute the query on.
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    ///
    /// Returns the count as number.
    pub fn count(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<u64> {
        let prep = self.prepare_query(corpus_name, query, query_language, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        return Ok(plan.count() as u64);
    }

    /// Count the number of results for a `query` and return both the total number of matches and also the number of documents in the result set.
    ///
    /// - `corpus_name` - The name of the corpus to execute the query on.
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    pub fn count_extra(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<CountExtra> {
        let prep = self.prepare_query(corpus_name, query, query_language, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db: &Graph = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let mut known_documents = HashSet::new();

        let node_name_key_id = db
            .node_annos
            .get_key_id(&db.get_node_name_key())
            .ok_or("No internal ID for node names found")?;

        let result = plan.fold((0, 0), move |acc: (u64, usize), m: Vec<Match>| {
            if !m.is_empty() {
                let m: &Match = &m[0];
                if let Some(node_name) = db
                    .node_annos
                    .get_value_for_item_by_id(&m.node, node_name_key_id)
                {
                    let node_name: &str = node_name.as_ref();
                    // extract the document path from the node name
                    let doc_path = &node_name[0..node_name.rfind('#').unwrap_or(node_name.len())];
                    known_documents.insert(doc_path.to_owned());
                }
            }
            (acc.0 + 1, known_documents.len())
        });

        return Ok(CountExtra {
            match_count: result.0,
            document_count: result.1 as u64,
        });
    }

    /// Find all results for a `query` and return the match ID for each result.
    ///
    /// The query is paginated and an offset and limit can be specified.
    ///
    /// - `corpus_name` - The name of the corpus to execute the query on.
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    /// - `offset` - Skip the `n` first results, where `n` is the offset.
    /// - `limit` - Return at most `n` matches, where `n` is the limit.
    /// - `order` - Specify the order of the matches.
    ///
    /// Returns a vector of match IDs, where each match ID consists of the matched node annotation identifiers separated by spaces.
    /// You can use the [subgraph(...)](#method.subgraph) method to get the subgraph for a single match described by the node annnotation identifiers.
    pub fn find(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
        offset: usize,
        limit: usize,
        order: ResultOrder,
    ) -> Result<Vec<String>> {
        let order_component = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        };
        let prep = self.prepare_query(corpus_name, query, query_language, vec![order_component])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;

        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let node_name_key_id = db
            .node_annos
            .get_key_id(&db.get_node_name_key())
            .ok_or("No internal ID for node names found")?;
        let mut node_to_path_cache = FxHashMap::default();
        let mut tmp_results: Vec<Vec<Match>> = Vec::with_capacity(1024);

        for mgroup in plan {
            // cache all paths of the matches
            for m in mgroup.iter() {
                if let Some(path) = db
                    .node_annos
                    .get_value_for_item_by_id(&m.node, node_name_key_id)
                {
                    let path = util::extract_node_path(&path);
                    node_to_path_cache.insert(m.node.clone(), path);
                }
            }

            // add all matches to temporary vector
            tmp_results.push(mgroup);
        }

        // either sort or randomly shuffle results
        if order == ResultOrder::Random {
            let mut rng = rand::thread_rng();
            rng.shuffle(&mut tmp_results[..])
        } else {
            let order_func = |m1: &Vec<Match>, m2: &Vec<Match>| -> std::cmp::Ordering {
                if order == ResultOrder::Inverted {
                    return db::sort_matches::compare_matchgroup_by_text_pos(
                        m1,
                        m2,
                        db,
                        &node_to_path_cache,
                    ).reverse();
                } else {
                    return db::sort_matches::compare_matchgroup_by_text_pos(
                        m1,
                        m2,
                        db,
                        &node_to_path_cache,
                    );
                }
            };

            if self.query_config.use_parallel_joins {
                tmp_results.par_sort_unstable_by(order_func);
            } else {
                tmp_results.sort_unstable_by(order_func);
            }
        }

        let expected_size = std::cmp::min(tmp_results.len(), limit);

        let mut results: Vec<String> = Vec::with_capacity(expected_size);
        results.extend(
            tmp_results
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|m: Vec<Match>| {
                    let mut match_desc: Vec<String> = Vec::new();
                    for singlematch in m.iter() {
                        let mut node_desc = String::new();

                        if let Some(anno_key) = db.node_annos.get_key_value(singlematch.anno_key) {
                            if &anno_key.ns != "annis" {
                                if !anno_key.ns.is_empty() {
                                    node_desc.push_str(&anno_key.ns);
                                    node_desc.push_str("::");
                                }
                                node_desc.push_str(&anno_key.name);
                                node_desc.push_str("::");
                            }
                        }

                        if let Some(name) = db
                            .node_annos
                            .get_value_for_item_by_id(&singlematch.node, node_name_key_id)
                        {
                            node_desc.push_str("salt:/");
                            node_desc.push_str(name.as_ref());
                        }

                        match_desc.push(node_desc);
                    }
                    let mut result = String::new();
                    result.push_str(&match_desc.join(" "));
                    return result;
                }),
        );

        return Ok(results);
    }

    /// Return the copy of a subgraph which includes the given list of node annotation identifiers,
    /// the nodes that cover the same token as the given nodes and
    /// all nodes that cover the token which are part of the defined context.
    ///
    /// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
    /// - `node_ids` - A set of node annotation identifiers describing the subgraph.
    /// - `ctx_left` and `ctx_right` - Left and right context in token distance to be included in the subgraph.
    pub fn subgraph(
        &self,
        corpus_name: &str,
        node_ids: Vec<String>,
        ctx_left: usize,
        ctx_right: usize,
    ) -> Result<Graph> {
        let db_entry = self.get_fully_loaded_entry(corpus_name)?;

        let mut query = Disjunction {
            alternatives: vec![],
        };

        // find all nodes covering the same token
        for source_node_id in node_ids {
            let source_node_id: &str = if source_node_id.starts_with("salt:/") {
                // remove the "salt:/" prefix
                &source_node_id[6..]
            } else {
                &source_node_id
            };

            // left context (non-token)
            {
                let mut q_left: Conjunction = Conjunction::new();

                let any_node_idx = q_left.add_node(NodeSearchSpec::AnyNode, None);

                let n_idx = q_left.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(db::ANNIS_NS.to_string()),
                        name: db::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);
                let tok_precedence_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                q_left.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &n_idx,
                    &tok_covered_idx,
                )?;
                q_left.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_left,
                    }),
                    &tok_precedence_idx,
                    &tok_covered_idx,
                )?;
                q_left.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &any_node_idx,
                    &tok_precedence_idx,
                )?;

                query.alternatives.push(q_left);
            }

            // left context (token onlys)
            {
                let mut q_left: Conjunction = Conjunction::new();

                let tok_precedence_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                let n_idx = q_left.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(db::ANNIS_NS.to_string()),
                        name: db::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                q_left.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &n_idx,
                    &tok_covered_idx,
                )?;
                q_left.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_left,
                    }),
                    &tok_precedence_idx,
                    &tok_covered_idx,
                )?;

                query.alternatives.push(q_left);
            }

            // right context (non-token)
            {
                let mut q_right: Conjunction = Conjunction::new();

                let any_node_idx = q_right.add_node(NodeSearchSpec::AnyNode, None);

                let n_idx = q_right.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(db::ANNIS_NS.to_string()),
                        name: db::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);
                let tok_precedence_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                q_right.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &n_idx,
                    &tok_covered_idx,
                )?;
                q_right.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_right,
                    }),
                    &tok_covered_idx,
                    &tok_precedence_idx,
                )?;
                q_right.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &any_node_idx,
                    &tok_precedence_idx,
                )?;

                query.alternatives.push(q_right);
            }

            // right context (token only)
            {
                let mut q_right: Conjunction = Conjunction::new();

                let tok_precedence_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                let n_idx = q_right.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(db::ANNIS_NS.to_string()),
                        name: db::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                q_right.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &n_idx,
                    &tok_covered_idx,
                )?;
                q_right.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_right,
                    }),
                    &tok_covered_idx,
                    &tok_precedence_idx,
                )?;

                query.alternatives.push(q_right);
            }
        }
        return extract_subgraph_by_query(db_entry, query, vec![0], self.query_config.clone());
    }

    /// Return the copy of a subgraph which includes all nodes matched by the given `query`.
    ///
    /// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
    /// - `query` - The query which defines included nodes.
    /// - `query_language` - The query language of the query (e.g. AQL).
    pub fn subgraph_for_query(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<Graph> {
        let prep = self.prepare_query(corpus_name, query, query_language, vec![])?;

        let mut max_alt_size = 0;
        for alt in prep.query.alternatives.iter() {
            max_alt_size = std::cmp::max(max_alt_size, alt.num_of_nodes());
        }

        return extract_subgraph_by_query(
            prep.db_entry.clone(),
            prep.query,
            (0..max_alt_size).collect(),
            self.query_config.clone(),
        );
    }

    /// Return the copy of a subgraph which includes all nodes that belong to any of the given list of sub-corpus/document identifiers.
    ///
    /// - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
    /// - `corpus_ids` - A set of sub-corpus/document identifiers describing the subgraph.
    pub fn subcorpus_graph(&self, corpus_name: &str, corpus_ids: Vec<String>) -> Result<Graph> {
        let db_entry = self.get_fully_loaded_entry(corpus_name)?;

        let mut query = Disjunction {
            alternatives: vec![],
        };
        // find all nodes that a connected with the corpus IDs
        for source_corpus_id in corpus_ids {
            let source_corpus_id: &str = if source_corpus_id.starts_with("salt:/") {
                // remove the "salt:/" prefix
                &source_corpus_id[6..]
            } else {
                &source_corpus_id
            };
            let mut q = Conjunction::new();
            let corpus_idx = q.add_node(
                NodeSearchSpec::ExactValue {
                    ns: Some(db::ANNIS_NS.to_string()),
                    name: db::NODE_NAME.to_string(),
                    val: Some(source_corpus_id.to_string()),
                    is_meta: false,
                },
                None,
            );
            let any_node_idx = q.add_node(NodeSearchSpec::AnyNode, None);
            q.add_operator(
                Box::new(operators::PartOfSubCorpusSpec {
                    min_dist: 1,
                    max_dist: usize::max_value(),
                }),
                &any_node_idx,
                &corpus_idx,
            )?;
            query.alternatives.push(q);
        }

        return extract_subgraph_by_query(db_entry, query, vec![1], self.query_config.clone());
    }

    /// Return the copy of the graph of the corpus given by `corpus_name`.
    pub fn corpus_graph(&self, corpus_name: &str) -> Result<Graph> {
        let db_entry = self.get_fully_loaded_entry(corpus_name)?;

        let mut query = Conjunction::new();

        query.add_node(
            NodeSearchSpec::new_exact(Some(ANNIS_NS), NODE_TYPE, Some("corpus"), false),
            None,
        );

        return extract_subgraph_by_query(
            db_entry,
            query.into_disjunction(),
            vec![0],
            self.query_config.clone(),
        );
    }

    /// Execute a frequency query.
    ///
    /// - `corpus_name` - The name of the corpus to execute the query on.
    /// - `query` - The query as string.
    /// - `query_language` The query language of the query (e.g. AQL).
    /// - `definition` - A list of frequency query definitions.
    ///
    /// Returns a frequency table of strings.
    pub fn frequency(
        &self,
        corpus_name: &str,
        query: &str,
        query_language: QueryLanguage,
        definition: Vec<FrequencyDefEntry>,
    ) -> Result<FrequencyTable<String>> {
        let prep = self.prepare_query(corpus_name, query, query_language, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db: &Graph = get_read_or_error(&lock)?;

        // get the matching annotation keys for each definition entry
        let mut annokeys: Vec<(usize, Vec<AnnoKey>)> = Vec::default();
        for def in definition.into_iter() {
            if let Some(node_ref) = prep.query.get_variable_pos(&def.node_ref) {
                if let Some(ns) = def.ns {
                    // add the single fully qualified annotation key
                    annokeys.push((
                        node_ref,
                        vec![AnnoKey {
                            ns: ns.clone(),
                            name: def.name.clone(),
                        }],
                    ));
                } else {
                    // add all matching annotation keys
                    annokeys.push((node_ref, db.node_annos.get_qnames(&def.name)));
                }
            }
        }

        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let mut tuple_frequency: FxHashMap<Vec<String>, usize> = FxHashMap::default();

        for mgroup in plan {
            // for each match, extract the defined annotation (by its key) from the result node
            let mut tuple: Vec<String> = Vec::with_capacity(annokeys.len());
            for (node_ref, anno_keys) in annokeys.iter() {
                let mut tuple_val: String = String::default();
                if *node_ref < mgroup.len() {
                    let m: &Match = &mgroup[*node_ref];
                    for k in anno_keys.iter() {
                        if let Some(val) = db.node_annos.get_value_for_item(&m.node, k) {
                            tuple_val = val.to_owned();
                        }
                    }
                }
                tuple.push(tuple_val);
            }
            // add the tuple to the frequency count
            let mut tuple_count: &mut usize = tuple_frequency.entry(tuple).or_insert(0);
            *tuple_count = *tuple_count + 1;
        }

        // output the frequency
        let mut result: FrequencyTable<String> = FrequencyTable::default();
        for (tuple, count) in tuple_frequency.into_iter() {
            result.push((tuple, count));
        }

        // sort the output (largest to smallest)
        result.sort_by(|a, b| a.1.cmp(&b.1).reverse());

        return Ok(result);
    }

    /// Parses a `query`and return a list of descriptions for its nodes.
    ///
    /// - `query` - The query to be analyzed.
    /// - `query_language` - The query language of the query (e.g. AQL).
    pub fn node_descriptions(
        &self,
        query: &str,
        query_language: QueryLanguage,
    ) -> Result<Vec<QueryAttributeDescription>> {
        let mut result = Vec::new();
        // parse query
        let q: Disjunction = match query_language {
            QueryLanguage::AQL => aql::parse(query)?,
        };
        let mut component_nr = 0;
        for alt in q.alternatives.into_iter() {
            let alt: Conjunction = alt;
            for mut n in alt.get_node_descriptions().into_iter() {
                n.alternative = component_nr;
                result.push(n);
            }
            component_nr += 1;
        }

        return Ok(result);
    }

    /// Returns a list of all components of a corpus given by `corpus_name`.
    ///
    /// - `ctype` - Optionally filter by the component type.
    /// - `name` - Optionally filter by the component name.
    pub fn list_components(
        &self,
        corpus_name: &str,
        ctype: Option<ComponentType>,
        name: Option<&str>,
    ) -> Vec<Component> {
        if let Ok(db_entry) = self.get_loaded_entry(corpus_name, false) {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                return db.get_all_components(ctype, name);
            }
        }
        return vec![];
    }

    /// Returns a list of all node annotations of a corpus given by `corpus_name`.
    ///
    /// - `list_values` - If true include the possible values in the result.
    /// - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
    pub fn list_node_annotations(
        &self,
        corpus_name: &str,
        list_values: bool,
        only_most_frequent_values: bool,
    ) -> Vec<Annotation> {
        let mut result: Vec<Annotation> = Vec::new();
        if let Ok(db_entry) = self.get_loaded_entry(corpus_name, false) {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                let node_annos: &AnnoStorage<NodeID> = &db.node_annos;
                for key in node_annos.annotation_keys() {
                    if list_values {
                        if only_most_frequent_values {
                            // get the first value
                            if let Some(val) =
                                node_annos.get_all_values(&key, true).into_iter().next()
                            {
                                result.push(Annotation {
                                    key: key.clone(),
                                    val: val.to_owned(),
                                });
                            }
                        } else {
                            // get all values
                            for val in node_annos.get_all_values(&key, false).into_iter() {
                                result.push(Annotation {
                                    key: key.clone(),
                                    val: val.to_owned(),
                                });
                            }
                        }
                    } else {
                        result.push(Annotation {
                            key: key.clone(),
                            val: String::default(),
                        });
                    }
                }
            }
        }

        return result;
    }

    /// Returns a list of all node annotations of a corpus given by `corpus_name`.
    ///
    /// - `list_values` - If true include the possible values in the result.
    /// - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
    pub fn list_edge_annotations(
        &self,
        corpus_name: &str,
        component: Component,
        list_values: bool,
        only_most_frequent_values: bool,
    ) -> Vec<Annotation> {
        let mut result: Vec<Annotation> = Vec::new();
        if let Ok(db_entry) =
            self.get_loaded_entry_with_components(corpus_name, vec![component.clone()])
        {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                if let Some(gs) = db.get_graphstorage(&component) {
                    let edge_annos: &AnnotationStorage<Edge> =
                        gs.as_edgecontainer().get_anno_storage();
                    for key in edge_annos.annotation_keys() {
                        if list_values {
                            if only_most_frequent_values {
                                // get the first value
                                if let Some(val) =
                                    edge_annos.get_all_values(&key, true).into_iter().next()
                                {
                                    result.push(Annotation {
                                        key: key.clone(),
                                        val: val.to_owned(),
                                    });
                                }
                            } else {
                                // get all values
                                for val in edge_annos.get_all_values(&key, false).into_iter() {
                                    result.push(Annotation {
                                        key: key.clone(),
                                        val: val.to_owned(),
                                    });
                                }
                            }
                        } else {
                            result.push(Annotation {
                                key: key.clone(),
                                val: String::new(),
                            });
                        }
                    }
                }
            }
        }

        return result;
    }

    fn check_cache_size_and_remove(&self, keep: Vec<&str>) {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;
        check_cache_size_and_remove_with_cache(cache, &self.cache_strategy, keep);
    }
}

impl Drop for CorpusStorage {
    fn drop(&mut self) {
        // wait until all background workers are finished
        let &(ref lock, ref cvar) = &*self.active_background_workers;
        let mut nr_active_background_workers = lock.lock().unwrap();
        while *nr_active_background_workers > 0 {
            trace!(
                "Waiting for background thread to finish ({} worker(s) left)...",
                *nr_active_background_workers
            );
            nr_active_background_workers = cvar.wait(nr_active_background_workers).unwrap();
        }

        // unlock lock file
        if let Err(e) = self.lock_file.unlock() {
            warn!("Could not unlock CorpusStorage lock file: {:?}", e);
        } else {
            trace!("Unlocked CorpusStorage lock file");
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate log;
    extern crate simplelog;
    extern crate tempdir;

    use corpusstorage::QueryLanguage;
    use update::{GraphUpdate, UpdateEvent};
    use CorpusStorage;

    #[test]
    fn delete() {
        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
            // fully load a corpus
            let mut g = GraphUpdate::new();
            g.add_event(UpdateEvent::AddNode {
                node_name: "test".to_string(),
                node_type: "node".to_string(),
            });

            cs.apply_update("testcorpus", &mut g).unwrap();
            cs.preload("testcorpus").unwrap();
            cs.delete("testcorpus").unwrap();
        }
    }

    #[test]
    fn load_cs_twice() {
        // Init logger to get a trace of the actions that failed
        simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default())
            .unwrap();

        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            {
                let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
                let mut g = GraphUpdate::new();
                g.add_event(UpdateEvent::AddNode {
                    node_name: "test".to_string(),
                    node_type: "node".to_string(),
                });

                cs.apply_update("testcorpus", &mut g).unwrap();
            }

            {
                let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();
                let mut g = GraphUpdate::new();
                g.add_event(UpdateEvent::AddNode {
                    node_name: "test".to_string(),
                    node_type: "node".to_string(),
                });

                cs.apply_update("testcorpus", &mut g).unwrap();
            }
        }
    }

    #[test]
    fn apply_update_add_nodes() {
        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            let mut cs = CorpusStorage::with_auto_cache_size(tmp.path(), false).unwrap();

            let mut g = GraphUpdate::new();
            g.add_event(UpdateEvent::AddNode {
                node_name: "root".to_string(),
                node_type: "corpus".to_string(),
            });
            g.add_event(UpdateEvent::AddNode {
                node_name: "root/doc1".to_string(),
                node_type: "corpus".to_string(),
            });
            g.add_event(UpdateEvent::AddNode {
                node_name: "root/doc1#MyToken".to_string(),
                node_type: "node".to_string(),
            });
            g.add_event(UpdateEvent::AddNode {
                node_name: "root/doc2".to_string(),
                node_type: "corpus".to_string(),
            });
            g.add_event(UpdateEvent::AddNode {
                node_name: "root/doc2#MyToken".to_string(),
                node_type: "node".to_string(),
            });

            cs.apply_update("root", &mut g).unwrap();

            let node_count = cs.count("root", "node", QueryLanguage::AQL).unwrap();
            assert_eq!(2, node_count);
        }
    }

}

fn get_read_or_error<'a>(lock: &'a RwLockReadGuard<CacheEntry>) -> Result<&'a Graph> {
    if let &CacheEntry::Loaded(ref db) = &**lock {
        return Ok(db);
    } else {
        return Err(ErrorKind::LoadingDBFailed("".to_string()).into());
    }
}

fn get_write_or_error<'a>(lock: &'a mut RwLockWriteGuard<CacheEntry>) -> Result<&'a mut Graph> {
    if let &mut CacheEntry::Loaded(ref mut db) = &mut **lock {
        return Ok(db);
    } else {
        return Err("Could get loaded graph storage entry".into());
    }
}

fn check_cache_size_and_remove_with_cache(
    cache: &mut LinkedHashMap<String, Arc<RwLock<CacheEntry>>>,
    cache_strategy: &CacheStrategy,
    keep: Vec<&str>,
) {
    let mut mem_ops = MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

    let keep: HashSet<&str> = keep.into_iter().collect();

    // check size of each corpus and calculate the sum of used memory
    let mut size_sum: usize = 0;
    let mut db_sizes: LinkedHashMap<String, usize> = LinkedHashMap::new();
    for (corpus, db_entry) in cache.iter() {
        let lock = db_entry.read().unwrap();
        if let &CacheEntry::Loaded(ref db) = &*lock {
            let s = db.size_of(&mut mem_ops);
            size_sum += s;
            db_sizes.insert(corpus.clone(), s);
        }
    }

    let max_cache_size: usize = match cache_strategy {
        CacheStrategy::OnlyOneCorpus => 0,
        CacheStrategy::FixedMaxMemory(max_size) => *max_size,
        CacheStrategy::PercentOfFreeMemory(max_percent) => {
            // get the current free space in main memory
            if let Ok(mem) = sys_info::mem_info() {
                // the free memory
                let free_system_mem: usize = mem.avail as usize * 1024; // mem.free is in KiB
                                                                        // A part of the system memory is already used by the cache.
                                                                        // We want x percent of the overall available memory (thus not used by us), so add the cache size
                let available_memory: usize = free_system_mem + size_sum;
                ((available_memory as f64) * (max_percent / 100.0)) as usize
            } else {
                // fallback to include only the last loaded corpus if free memory size is unknown
                0
            }
        }
    };

    debug!(
        "Current cache size is {:.2} MB / max  {:.2} MB",
        (size_sum as f64) / (1024.0 * 1024.0),
        (max_cache_size as f64) / (1024.0 * 1024.0)
    );

    // remove older entries (at the beginning) until cache size requirements are met,
    // but never remove the last loaded entry
    for (corpus_name, corpus_size) in db_sizes.iter() {
        if size_sum > max_cache_size {
            if !keep.contains(corpus_name.as_str()) {
                info!("Removing corpus {} from cache", corpus_name);
                cache.remove(corpus_name);
                size_sum -= corpus_size;
                debug!(
                    "Current cache size is {:.2} MB / max  {:.2} MB",
                    (size_sum as f64) / (1024.0 * 1024.0),
                    (max_cache_size as f64) / (1024.0 * 1024.0)
                );
            }
        } else {
            // cache size is smaller, nothing to do
            break;
        }
    }
}

fn extract_subgraph_by_query(
    db_entry: Arc<RwLock<CacheEntry>>,
    query: Disjunction,
    match_idx: Vec<usize>,
    query_config: query::Config,
) -> Result<Graph> {
    // accuire read-only lock and create query that finds the context nodes
    let lock = db_entry.read().unwrap();
    let orig_db = get_read_or_error(&lock)?;

    let plan = ExecutionPlan::from_disjunction(&query, &orig_db, query_config).chain_err(|| "")?;

    debug!("executing subgraph query\n{}", plan);

    let all_components = orig_db.get_all_components(None, None);

    // We have to keep our own unique set because the query will return "duplicates" whenever the other parts of the
    // match vector differ.
    let mut match_result: BTreeSet<Match> = BTreeSet::new();

    let mut result = Graph::new();

    // create the subgraph description
    for r in plan {
        trace!("subgraph query found match {:?}", r);
        for i in match_idx.iter().cloned() {
            if i < r.len() {
                let m: &Match = &r[i];
                if !match_result.contains(m) {
                    match_result.insert(m.clone());
                    trace!("subgraph query extracted node {:?}", m.node);
                    create_subgraph_node(m.node, &mut result, orig_db);
                }
            }
        }
    }

    for m in match_result.iter() {
        create_subgraph_edge(m.node, &mut result, orig_db, &all_components);
    }

    return Ok(result);
}

fn create_subgraph_node(id: NodeID, db: &mut Graph, orig_db: &Graph) {
    // add all node labels with the same node ID
    let node_annos = Arc::make_mut(&mut db.node_annos);
    for a in orig_db.node_annos.get_annotations_for_item(&id).into_iter() {
        node_annos.insert(id, a);
    }
}
fn create_subgraph_edge(
    source_id: NodeID,
    db: &mut Graph,
    orig_db: &Graph,
    all_components: &Vec<Component>,
) {
    // find outgoing edges
    for c in all_components {
        if let Some(orig_gs) = orig_db.get_graphstorage(c) {
            for target in orig_gs.get_outgoing_edges(&source_id) {
                let e = Edge {
                    source: source_id,
                    target,
                };
                if let Ok(new_gs) = db.get_or_create_writable(c.clone()) {
                    new_gs.add_edge(e.clone());
                }

                for a in orig_gs
                    .get_anno_storage()
                    .get_annotations_for_item(&Edge {
                        source: source_id,
                        target,
                    }).into_iter()
                {
                    if let Ok(new_gs) = db.get_or_create_writable(c.clone()) {
                        new_gs.add_edge_annotation(e.clone(), a);
                    }
                }
            }
        }
    }
}

fn create_lockfile_for_directory(db_dir: &Path) -> Result<File> {
    std::fs::create_dir_all(&db_dir)
        .chain_err(|| format!("Could not create directory {}", db_dir.to_string_lossy()))?;
    let lock_file_path = db_dir.join("db.lock");
    // check if we can get the file lock
    let lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(lock_file_path.as_path())
        .chain_err(|| {
            format!(
                "Could not open or create lockfile {}",
                lock_file_path.to_string_lossy()
            )
        })?;
    lock_file.try_lock_exclusive().chain_err(|| {
        format!(
            "Could not accuire lock for directory {}",
            db_dir.to_string_lossy()
        )
    })?;

    return Ok(lock_file);
}
