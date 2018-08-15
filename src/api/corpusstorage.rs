//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use annostorage::AnnoStorage;
use api::update::GraphUpdate;
use errors::*;
use exec::nodesearch::NodeSearchSpec;
use fs2::FileExt;
use graphdb;
use graphdb::GraphDB;
use graphdb::{ANNIS_NS, NODE_TYPE};
use linked_hash_map::LinkedHashMap;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use aql::operators;
use aql;
use plan::ExecutionPlan;
use query;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use std;
use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::fs::OpenOptions;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use types;
use util;
use util::memory_estimation;
use FrequencyTable;
use {AnnoKey, Annotation, Component, ComponentType, CountExtra, Edge, Match, NodeID, StringID};

use fxhash::FxHashMap;

use rand;
use rand::Rng;
use rayon::prelude::*;
use sys_info;

enum CacheEntry {
    Loaded(GraphDB),
    NotLoaded,
}

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
pub enum LoadStatus {
    NotLoaded,
    PartiallyLoaded(usize),
    FullyLoaded(usize),
}

#[derive(Ord, Eq, PartialOrd, PartialEq)]
pub struct CorpusInfo {
    pub name: String,
    pub load_status: LoadStatus,
    pub memory_size: usize,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum ResultOrder {
    Normal,
    Inverted,
    Random,
}

pub struct CorpusStorage {
    db_dir: PathBuf,
    lock_file: File,
    max_allowed_cache_size: Option<usize>,
    corpus_cache: RwLock<LinkedHashMap<String, Arc<RwLock<CacheEntry>>>>,
    pub query_config: query::Config,
}

struct PreparationResult<'a> {
    query: Disjunction<'a>,
    db_entry: Arc<RwLock<CacheEntry>>,
}

#[derive(Debug)]
pub struct FrequencyDefEntry {
    pub ns: Option<String>,
    pub name: String,
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

fn get_read_or_error<'a>(lock: &'a RwLockReadGuard<CacheEntry>) -> Result<&'a GraphDB> {
    if let &CacheEntry::Loaded(ref db) = &**lock {
        return Ok(db);
    } else {
        return Err(ErrorKind::LoadingDBFailed("".to_string()).into());
    }
}

fn get_write_or_error<'a>(lock: &'a mut RwLockWriteGuard<CacheEntry>) -> Result<&'a mut GraphDB> {
    if let &mut CacheEntry::Loaded(ref mut db) = &mut **lock {
        return Ok(db);
    } else {
        return Err("Could get loaded graph storage entry".into());
    }
}

fn check_cache_size_and_remove(
    max_cache_size: Option<usize>,
    cache: &mut LinkedHashMap<String, Arc<RwLock<CacheEntry>>>,
) {
    let mut mem_ops = MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

    // only prune corpora from the cache if max. size was set
    if let Some(max_cache_size) = max_cache_size {
        // check size of each corpus
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
        let mut num_of_loaded_corpora = db_sizes.len();

        // remove older entries (at the beginning) until cache size requirements are met,
        // but never remove the last loaded entry
        for (corpus_name, corpus_size) in db_sizes.iter() {
            if num_of_loaded_corpora > 1 && size_sum > max_cache_size {
                info!("Removing corpus {} from cache", corpus_name);
                cache.remove(corpus_name);
                size_sum -= corpus_size;
                num_of_loaded_corpora -= 1;
            } else {
                // nothing to do
                break;
            }
        }
    }
}

fn extract_subgraph_by_query(
    db_entry: Arc<RwLock<CacheEntry>>,
    query: Disjunction,
    match_idx: Vec<usize>,
    query_config: query::Config,
) -> Result<GraphDB> {
    // accuire read-only lock and create query that finds the context nodes
    let lock = db_entry.read().unwrap();
    let orig_db = get_read_or_error(&lock)?;

    let plan = ExecutionPlan::from_disjunction(&query, &orig_db, query_config).chain_err(|| "")?;

    debug!("executing subgraph query\n{}", plan);

    let all_components = orig_db.get_all_components(None, None);

    // We have to keep our own unique set because the query will return "duplicates" whenever the other parts of the
    // match vector differ.
    let mut match_result: BTreeSet<Match> = BTreeSet::new();

    let mut result = GraphDB::new();

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

fn create_subgraph_node(id: NodeID, db: &mut GraphDB, orig_db: &GraphDB) {
    // add all node labels with the same node ID
    let node_annos = Arc::make_mut(&mut db.node_annos);
    for a in orig_db.node_annos.get_all(&id) {
        if let (Some(ns), Some(name), Some(val)) = (
            orig_db.strings.str(a.key.ns),
            orig_db.strings.str(a.key.name),
            orig_db.strings.str(a.val),
        ) {
            let new_anno = Annotation {
                key: AnnoKey {
                    ns: Arc::make_mut(&mut db.strings).add(ns),
                    name: Arc::make_mut(&mut db.strings).add(name),
                },
                val: Arc::make_mut(&mut db.strings).add(val),
            };
            node_annos.insert(id, new_anno);
        }
    }
}
fn create_subgraph_edge(
    source_id: NodeID,
    db: &mut GraphDB,
    orig_db: &GraphDB,
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
                    .get_edge_annos(&types::Edge {
                        source: source_id,
                        target,
                    })
                    .into_iter()
                {
                    if let (Some(ns), Some(name), Some(val)) = (
                        orig_db.strings.str(a.key.ns),
                        orig_db.strings.str(a.key.name),
                        orig_db.strings.str(a.val),
                    ) {
                        let new_anno = Annotation {
                            key: AnnoKey {
                                ns: Arc::make_mut(&mut db.strings).add(ns),
                                name: Arc::make_mut(&mut db.strings).add(name),
                            },
                            val: Arc::make_mut(&mut db.strings).add(val),
                        };
                        if let Ok(new_gs) = db.get_or_create_writable(c.clone()) {
                            new_gs.add_edge_annotation(e.clone(), new_anno.clone());
                        }
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

impl CorpusStorage {
    pub fn new(
        db_dir: &Path,
        max_allowed_cache_size: Option<usize>,
        use_parallel_joins: bool,
    ) -> Result<CorpusStorage> {
        let query_config = query::Config { use_parallel_joins };

        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            lock_file: create_lockfile_for_directory(db_dir)?,
            max_allowed_cache_size,
            corpus_cache: RwLock::new(LinkedHashMap::new()),
            query_config,
        };

        Ok(cs)
    }

    pub fn new_auto_cache_size(db_dir: &Path, use_parallel_joins: bool) -> Result<CorpusStorage> {
        let query_config = query::Config { use_parallel_joins };

        // get the amount of available memory, use a quarter of it per default
        let cache_size: usize = if let Ok(mem) = sys_info::mem_info() {
            (((mem.avail as usize * 1024) as f64) / 4.0) as usize // mem.free is in KiB
        } else {
            // default to 1 GB
            1024 * 1024 * 1024
        };
        info!(
            "Using cache with size {:.*} MiB",
            2,
            cache_size as f64 / ((1024 * 1024) as f64)
        );

        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            lock_file: create_lockfile_for_directory(db_dir)?,
            max_allowed_cache_size: Some(cache_size), // 1 GB
            corpus_cache: RwLock::new(LinkedHashMap::new()),
            query_config: query_config,
        };

        Ok(cs)
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

    pub fn list(&self) -> Result<Vec<CorpusInfo>> {
        let names: Vec<String> = self.list_from_disk().unwrap_or_default();
        let mut result: Vec<CorpusInfo> = vec![];

        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

        for n in names {
            let cache_entry = self.get_entry(&n)?;
            let lock = cache_entry.read().unwrap();
            let corpus_info: CorpusInfo = match &*lock {
                &CacheEntry::Loaded(ref db) => {
                    // check if all components are loaded
                    let heap_size = db.size_of(&mut mem_ops);
                    let mut load_status = LoadStatus::FullyLoaded(heap_size);

                    for c in db.get_all_components(None, None) {
                        if !db.is_loaded(&c) {
                            load_status = LoadStatus::PartiallyLoaded(heap_size);
                            break;
                        }
                    }

                    CorpusInfo {
                        name: n.clone(),
                        load_status,
                        memory_size: 0,
                    }
                }
                &CacheEntry::NotLoaded => CorpusInfo {
                    name: n.clone(),
                    load_status: LoadStatus::NotLoaded,
                    memory_size: 0,
                },
            };
            result.push(corpus_info);
        }

        return Ok(result);
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

        let mut db = GraphDB::new();
        if create_corpus {
            db.persist_to(&db_path).chain_err(|| format!("Could not create corpus with name {}", corpus_name))?;
        } else {
            db.load_from(&db_path, false)?;
        }

        let entry = Arc::new(RwLock::new(CacheEntry::Loaded(db)));
        // first remove entry, than add it: this ensures it is at the end of the linked hash map
        cache.remove(corpus_name);
        cache.insert(String::from(corpus_name), entry.clone());

        check_cache_size_and_remove(self.max_allowed_cache_size, cache);

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

    /// Import a corpusfrom an external location into this corpus storage
    pub fn import(&self, corpus_name: &str, mut db: GraphDB) {
        let r = db.ensure_loaded_all();
        if let Err(e) = r {
            error!(
                "Some error occured when attempting to load components from disk: {:?}",
                e
            );
        }

        let mut db_path = PathBuf::from(&self.db_dir);
        db_path.push(corpus_name);

        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        // remove any possible old corpus
        let old_entry = cache.remove(corpus_name);
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
        let save_result = db.save_to(&db_path);
        if let Err(e) = save_result {
            error!(
                "Can't save corpus to {}: {:?}",
                db_path.to_string_lossy(),
                e
            );
        }

        // make it known to the cache
        cache.insert(
            String::from(corpus_name),
            Arc::new(RwLock::new(CacheEntry::Loaded(db))),
        );
        check_cache_size_and_remove(self.max_allowed_cache_size, cache);
    }

    /// delete a corpus
    pub fn delete(&self, corpus_name: &str) -> Result<()> {
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
                std::fs::remove_dir_all(db_path.clone()).chain_err(|| "Error when removing existing files")?
            }
        } else {
            return Err(ErrorKind::NoSuchCorpus(corpus_name.to_string()).into());
        }
        return Ok(());
    }

    fn prepare_query<'a>(
        &self,
        corpus_name: &str,
        query_as_aql: &'a str,
        additional_components: Vec<Component>,
    ) -> Result<PreparationResult<'a>> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;

        // make sure the database is loaded with all necessary components
        let (q, missing_components) = {
            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;
            let q = aql::parse(query_as_aql).chain_err(|| "Could not parse AQL")?;
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
            let mut lock = db_entry.write().unwrap();
            let db = get_write_or_error(&mut lock)?;
            for c in missing_components {
                db.ensure_loaded(&c)?;
            }
        };

        return Ok(PreparationResult { query: q, db_entry });
    }

    pub fn get_string(&self, corpus_name: &str, str_id: StringID) -> Result<String> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;

        // accuire read-only lock and get string
        let lock = db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let result = db
            .strings
            .str(str_id)
            .cloned()
            .ok_or(ErrorKind::NoSuchStringID(str_id))?;
        return Ok(result);
    }

    pub fn plan(&self, corpus_name: &str, query_as_json: &str) -> Result<String> {
        let prep = self.prepare_query(corpus_name, query_as_json, vec![])?;

        // accuire read-only lock and plan
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        return Ok(format!("{}", plan));
    }

    pub fn preload(&self, corpus_name: &str) -> Result<()> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let mut lock = db_entry.write().unwrap();
        let db = get_write_or_error(&mut lock)?;
        db.ensure_loaded_all()?;
        return Ok(());
    }

    pub fn update_statistics(&self, corpus_name: &str) -> Result<()> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let mut lock = db_entry.write().unwrap();
        let db: &mut GraphDB = get_write_or_error(&mut lock)?;

        Arc::make_mut(&mut db.node_annos).calculate_statistics(&db.strings);
        for c in db.get_all_components(None, None).into_iter() {
            db.calculate_component_statistics(&c)?;
        }

        // TODO: persist changes

        Ok(())
    }

    pub fn count(&self, corpus_name: &str, query_as_json: &str) -> Result<u64> {
        let prep = self.prepare_query(corpus_name, query_as_json, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        return Ok(plan.count() as u64);
    }

    pub fn count_extra(&self, corpus_name: &str, query_as_json: &str) -> Result<CountExtra> {
        let prep = self.prepare_query(corpus_name, query_as_json, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db: &GraphDB = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let mut known_documents = HashSet::new();

        let result = plan.fold((0, 0), move |acc: (u64, usize), m: Vec<Match>| {
            if !m.is_empty() {
                let m: &Match = &m[0];
                if let Some(node_name_id) = db.node_annos.get(&m.node, &db.get_node_name_key()) {
                    if let Some(node_name) = db.strings.str(node_name_id.clone()) {
                        let node_name: &str = node_name;
                        // extract the document path from the node name
                        let doc_path =
                            &node_name[0..node_name.rfind('#').unwrap_or(node_name.len())];
                        known_documents.insert(doc_path);
                    }
                }
            }
            (acc.0 + 1, known_documents.len())
        });

        return Ok(CountExtra {
            match_count: result.0,
            document_count: result.1 as u64,
        });
    }

    pub fn find(
        &self,
        corpus_name: &str,
        query_as_json: &str,
        offset: usize,
        limit: usize,
        order: ResultOrder,
    ) -> Result<Vec<String>> {
        let order_component = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: String::from(""),
        };
        let prep = self.prepare_query(corpus_name, query_as_json, vec![order_component])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;

        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let node_name_key = db.get_node_name_key();
        let mut node_to_path_cache = FxHashMap::default();
        let mut tmp_results: Vec<Vec<Match>> = Vec::with_capacity(1024);

        for mgroup in plan {
            // cache all paths of the matches
            for m in mgroup.iter() {
                if let Some(path_strid) = db.node_annos.get(&m.node, &node_name_key) {
                    if let Some(path) = db.strings.str(*path_strid) {
                        let path = util::extract_node_path(path);
                        node_to_path_cache.insert(m.node.clone(), path);
                    }
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
                    return util::sort_matches::compare_matchgroup_by_text_pos(
                        m1,
                        m2,
                        db,
                        &node_to_path_cache,
                    ).reverse();
                } else {
                    return util::sort_matches::compare_matchgroup_by_text_pos(
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

                        let anno_key: &AnnoKey = &singlematch.anno.key;
                        if let (Some(anno_ns), Some(anno_name)) =
                            (db.strings.str(anno_key.ns), db.strings.str(anno_key.name))
                        {
                            if anno_ns != "annis" {
                                if !anno_ns.is_empty() {
                                    node_desc.push_str(anno_ns);
                                    node_desc.push_str("::");
                                }
                                node_desc.push_str(anno_name);
                                node_desc.push_str("::");
                            }
                        }

                        if let Some(name_id) = db
                            .node_annos
                            .get(&singlematch.node, &db.get_node_name_key())
                        {
                            if let Some(name) = db.strings.str(name_id.clone()) {
                                node_desc.push_str("salt:/");
                                node_desc.push_str(name);
                            }
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

    pub fn subgraph(
        &self,
        corpus_name: &str,
        node_ids: Vec<String>,
        ctx_left: usize,
        ctx_right: usize,
    ) -> Result<GraphDB> {
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
                        ns: Some(graphdb::ANNIS_NS.to_string()),
                        name: graphdb::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);
                let tok_precedence_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                q_left.add_operator(Box::new(operators::OverlapSpec {}), &n_idx, &tok_covered_idx);
                q_left.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_left,
                    }),
                    &tok_precedence_idx,
                    &tok_covered_idx,
                );
                q_left.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &any_node_idx,
                    &tok_precedence_idx,
                );

                query.alternatives.push(q_left);
            }

            // left context (token onlys)
            {
                let mut q_left: Conjunction = Conjunction::new();

                let tok_precedence_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                let n_idx = q_left.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(graphdb::ANNIS_NS.to_string()),
                        name: graphdb::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_left.add_node(NodeSearchSpec::AnyToken, None);

                q_left.add_operator(Box::new(operators::OverlapSpec {}), &n_idx, &tok_covered_idx);
                q_left.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_left,
                    }),
                    &tok_precedence_idx,
                    &tok_covered_idx,
                );

                query.alternatives.push(q_left);
            }

            // right context (non-token)
            {
                let mut q_right: Conjunction = Conjunction::new();

                let any_node_idx = q_right.add_node(NodeSearchSpec::AnyNode, None);

                let n_idx = q_right.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(graphdb::ANNIS_NS.to_string()),
                        name: graphdb::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);
                let tok_precedence_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                q_right.add_operator(Box::new(operators::OverlapSpec {}), &n_idx, &tok_covered_idx);
                q_right.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_right,
                    }),
                    &tok_covered_idx,
                    &tok_precedence_idx,
                );
                q_right.add_operator(
                    Box::new(operators::OverlapSpec {}),
                    &any_node_idx,
                    &tok_precedence_idx,
                );

                query.alternatives.push(q_right);
            }

            // right context (token only)
            {
                let mut q_right: Conjunction = Conjunction::new();

                let tok_precedence_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                let n_idx = q_right.add_node(
                    NodeSearchSpec::ExactValue {
                        ns: Some(graphdb::ANNIS_NS.to_string()),
                        name: graphdb::NODE_NAME.to_string(),
                        val: Some(source_node_id.to_string()),
                        is_meta: false,
                    },
                    None,
                );
                let tok_covered_idx = q_right.add_node(NodeSearchSpec::AnyToken, None);

                q_right.add_operator(Box::new(operators::OverlapSpec {}), &n_idx, &tok_covered_idx);
                q_right.add_operator(
                    Box::new(operators::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_right,
                    }),
                    &tok_covered_idx,
                    &tok_precedence_idx,
                );

                query.alternatives.push(q_right);
            }
        }
        return extract_subgraph_by_query(db_entry, query, vec![0], self.query_config.clone());
    }

    pub fn subgraph_for_query(&self, corpus_name: &str, query_as_json: &str) -> Result<GraphDB> {
        let prep = self.prepare_query(corpus_name, query_as_json, vec![])?;

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
    pub fn subcorpus_graph(&self, corpus_name: &str, corpus_ids: Vec<String>) -> Result<GraphDB> {
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
                    ns: Some(graphdb::ANNIS_NS.to_string()),
                    name: graphdb::NODE_NAME.to_string(),
                    val: Some(source_corpus_id.to_string()),
                    is_meta: false,
                },
                None,
            );
            let any_node_idx = q.add_node(NodeSearchSpec::AnyNode, None);
            q.add_operator(
                Box::new(operators::PartOfSubCorpusSpec{min_dist: 1, max_dist:1}),
                &any_node_idx,
                &corpus_idx,
            );
            query.alternatives.push(q);
        }

        return extract_subgraph_by_query(db_entry, query, vec![1], self.query_config.clone());
    }

    pub fn corpus_graph(&self, corpus_name: &str) -> Result<GraphDB> {
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

    pub fn frequency(
        &self,
        corpus_name: &str,
        query_as_json: &str,
        definition: Vec<FrequencyDefEntry>,
    ) -> Result<FrequencyTable<String>> {
        let prep = self.prepare_query(corpus_name, query_as_json, vec![])?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db: &GraphDB = get_read_or_error(&lock)?;

        // get the matching annotation keys for each definition entry
        let mut annokeys: Vec<(usize, Vec<AnnoKey>)> = Vec::default();
        for def in definition.into_iter() {
            let ns_id: Option<&StringID> = if let Some(ns) = def.ns {
                db.strings.find_id(&ns)
            } else {
                None
            };
            let name_id = db
                .strings
                .find_id(&def.name)
                .ok_or(ErrorKind::NoSuchString(def.name.clone()))?;

            if let Some(node_ref) = prep.query.get_variable_pos(&def.node_ref) {
                if let Some(ns_id) = ns_id {
                    // add the single fully qualified annotation key
                    annokeys.push((
                        node_ref,
                        vec![AnnoKey {
                            ns: *ns_id,
                            name: *name_id,
                        }],
                    ));
                } else {
                    // add all matching annotation keys
                    annokeys.push((node_ref, db.node_annos.get_qnames(*name_id)));
                }
            }
        }

        let plan = ExecutionPlan::from_disjunction(&prep.query, &db, self.query_config.clone())?;

        let mut tuple_frequency: FxHashMap<Vec<StringID>, usize> = FxHashMap::default();

        for mgroup in plan {
            // for each match, extract the defined annotation (by its key) from the result node
            let mut tuple: Vec<StringID> = Vec::with_capacity(annokeys.len());
            for (node_ref, anno_keys) in annokeys.iter() {
                let mut tuple_val: StringID = 0;
                if *node_ref < mgroup.len() {
                    let m: &Match = &mgroup[*node_ref];
                    for k in anno_keys.iter() {
                        if let Some(val) = db.node_annos.get(&m.node, k) {
                            tuple_val = *val;
                        }
                    }
                }
                tuple.push(tuple_val);
            }
            // add the tuple to the frequency count
            let mut tuple_count: &mut usize = tuple_frequency.entry(tuple).or_insert(0);
            *tuple_count = *tuple_count + 1;
        }

        // output the frequency (needs collecting the actual string values)
        let mut result: FrequencyTable<String> = FrequencyTable::default();
        for (tuple_strid, count) in tuple_frequency.into_iter() {
            let mut tuple: Vec<String> = Vec::with_capacity(tuple_strid.len());
            for v in tuple_strid.into_iter() {
                tuple.push(db.strings.str(v).unwrap_or(&String::default()).clone());
            }
            result.push((tuple, count));
        }

        // sort the output (largest to smallest)
        result.sort_by(|a, b| a.1.cmp(&b.1).reverse());

        return Ok(result);
    }

    pub fn get_all_components(
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

    pub fn list_node_annotations(
        &self,
        corpus_name: &str,
        list_values: bool,
        only_most_frequent_values: bool,
    ) -> Vec<(String, String, String)> {
        let mut result = Vec::new();
        if let Ok(db_entry) = self.get_loaded_entry(corpus_name, false) {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                let node_annos: &AnnoStorage<NodeID> = &db.node_annos;
                for key in node_annos.get_all_keys() {
                    if let (Some(ns), Some(name)) =
                        (db.strings.str(key.ns), db.strings.str(key.name))
                    {
                        if list_values {
                            if only_most_frequent_values {
                                // get the first value
                                if let Some(val) = node_annos.get_all_values(key, true).next() {
                                    result.push((
                                        ns.clone(),
                                        name.clone(),
                                        db.strings.str(val).cloned().unwrap_or_default(),
                                    ));
                                }
                            } else {
                                // get all values
                                for val in node_annos.get_all_values(key, false) {
                                    result.push((
                                        ns.clone(),
                                        name.clone(),
                                        db.strings.str(val).cloned().unwrap_or_default(),
                                    ));
                                }
                            }
                        } else {
                            result.push((ns.clone(), name.clone(), String::new()));
                        }
                    }
                }
            }
        }

        return result;
    }

    pub fn list_edge_annotations(
        &self,
        corpus_name: &str,
        component: Component,
        list_values: bool,
        only_most_frequent_values: bool,
    ) -> Vec<(String, String, String)> {
        let mut result = Vec::new();
        if let Ok(db_entry) =
            self.get_loaded_entry_with_components(corpus_name, vec![component.clone()])
        {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                if let Some(gs) = db.get_graphstorage(&component) {
                    let edge_annos: &AnnoStorage<Edge> = gs.as_edgecontainer().get_anno_storage();
                    for key in edge_annos.get_all_keys() {
                        if let (Some(ns), Some(name)) =
                            (db.strings.str(key.ns), db.strings.str(key.name))
                        {
                            if list_values {
                                if only_most_frequent_values {
                                    // get the first value
                                    if let Some(val) = edge_annos.get_all_values(key, true).next() {
                                        result.push((
                                            ns.clone(),
                                            name.clone(),
                                            db.strings.str(val).cloned().unwrap_or_default(),
                                        ));
                                    }
                                } else {
                                    // get all values
                                    for val in edge_annos.get_all_values(key, false) {
                                        result.push((
                                            ns.clone(),
                                            name.clone(),
                                            db.strings.str(val).cloned().unwrap_or_default(),
                                        ));
                                    }
                                }
                            } else {
                                result.push((ns.clone(), name.clone(), String::new()));
                            }
                        }
                    }
                }
            }
        }

        return result;
    }

    pub fn apply_update(&self, corpus_name: &str, update: &mut GraphUpdate) -> Result<()> {
        let db_entry = self.get_loaded_entry(corpus_name, true).chain_err(|| format!("Could not get loaded entry for corpus {}", corpus_name))?;
        {
            let mut lock = db_entry.write().unwrap();
            let db: &mut GraphDB = get_write_or_error(&mut lock)?;

            db.apply_update(update)?;
        }
        // start background thread to persists the results
        std::thread::spawn(move || {
            trace!("Starting background thread to sync WAL updates");
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                let db: &GraphDB = db;
                if let Err(e) = db.background_sync_wal_updates() {
                    error!("Can't sync changes in background thread: {:?}", e);
                } else {
                    trace!("Finished background thread to sync WAL updates");
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use api::corpusstorage::CorpusStorage;
    use api::update::{GraphUpdate, UpdateEvent};

    #[test]
    fn delete() {
        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            let cs = CorpusStorage::new_auto_cache_size(tmp.path(), false).unwrap();
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
        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            {
                let cs = CorpusStorage::new_auto_cache_size(tmp.path(), false).unwrap();
                let mut g = GraphUpdate::new();
                g.add_event(UpdateEvent::AddNode {
                    node_name: "test".to_string(),
                    node_type: "node".to_string(),
                });

                cs.apply_update("testcorpus", &mut g).unwrap();
            }

            {
                let cs = CorpusStorage::new_auto_cache_size(tmp.path(), false).unwrap();
                let mut g = GraphUpdate::new();
                g.add_event(UpdateEvent::AddNode {
                    node_name: "test".to_string(),
                    node_type: "node".to_string(),
                });

                cs.apply_update("testcorpus", &mut g).unwrap();
            }
        }
    }
}

impl Drop for CorpusStorage {
    fn drop(&mut self) {
        if let Err(e) = self.lock_file.unlock() {
            warn!("Could not unlock CorpusStorage lock file: {:?}", e);
        }
    }
}
