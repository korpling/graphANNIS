//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use graphdb::{ANNIS_NS, NODE_TYPE};
use {AnnoKey, Annotation, Component, ComponentType, CountExtra, Edge, Match, NodeID, StringID};
use parser::jsonqueryparser;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::path::{Path, PathBuf};
use std::collections::{BTreeSet, HashSet};
use graphdb;
use operator;
use graphdb::GraphDB;
use std;
use plan;
use types;
use plan::ExecutionPlan;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;
use exec::nodesearch::NodeSearchSpec;
use heapsize::HeapSizeOf;
use std::iter::FromIterator;
use linked_hash_map::LinkedHashMap;
use api::update::GraphUpdate;

enum CacheEntry {
    Loaded(GraphDB),
    NotLoaded,
}

#[derive(Debug)]
pub enum Error {
    IOerror(std::io::Error),
    DBError(graphdb::Error),
    LoadingFailed,
    ImpossibleSearch(String),
    NoSuchCorpus,
    QueryCreationError(plan::Error),
    StringConvert(std::ffi::OsString),
    ParserError,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOerror(e)
    }
}

impl From<graphdb::Error> for Error {
    fn from(e: graphdb::Error) -> Error {
        Error::DBError(e)
    }
}

impl From<plan::Error> for Error {
    fn from(e: plan::Error) -> Error {
        match e {
            plan::Error::ImpossibleSearch(error_vec) => {
                let error_msg: Vec<String> =
                    error_vec.into_iter().map(|x| format!("{:?}", x)).collect();
                Error::ImpossibleSearch(error_msg.join("\n"))
            }
        }
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(e: std::ffi::OsString) -> Error {
        Error::StringConvert(e)
    }
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

pub struct CorpusStorage {
    db_dir: PathBuf,
    max_allowed_cache_size: Option<usize>,
    corpus_cache: RwLock<LinkedHashMap<String, Arc<RwLock<CacheEntry>>>>,
}

struct PreparationResult<'a> {
    query: Disjunction<'a>,
    db_entry: Arc<RwLock<CacheEntry>>,
}

fn get_read_or_error<'a>(lock: &'a RwLockReadGuard<CacheEntry>) -> Result<&'a GraphDB, Error> {
    if let &CacheEntry::Loaded(ref db) = &**lock {
        return Ok(db);
    } else {
        return Err(Error::LoadingFailed);
    }
}

fn get_write_or_error<'a>(
    lock: &'a mut RwLockWriteGuard<CacheEntry>,
) -> Result<&'a mut GraphDB, Error> {
    if let &mut CacheEntry::Loaded(ref mut db) = &mut **lock {
        return Ok(db);
    } else {
        return Err(Error::LoadingFailed);
    }
}

fn check_cache_size_and_remove(
    max_cache_size: Option<usize>,
    cache: &mut LinkedHashMap<String, Arc<RwLock<CacheEntry>>>,
) {
    // only prune corpora from the cache if max. size was set
    if let Some(max_cache_size) = max_cache_size {
        // check size of each corpus
        let mut size_sum: usize = 0;
        let mut db_sizes: LinkedHashMap<String, usize> = LinkedHashMap::new();
        for (corpus, db_entry) in cache.iter() {
            let lock = db_entry.read().unwrap();
            if let &CacheEntry::Loaded(ref db) = &*lock {
                let s = db.heap_size_of_children();
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
) -> Result<GraphDB, Error> {
    // accuire read-only lock and create query that finds the context nodes
    let lock = db_entry.read().unwrap();
    let orig_db = get_read_or_error(&lock)?;

    let plan = ExecutionPlan::from_disjunction(&query, &orig_db)?;

    trace!("executing subgraph query\n{}", plan);

    let all_components = orig_db.get_all_components(None, None);

    // We have to keep our own unique set because the query will return "duplicates" whenever the other parts of the
    // match vector differ.
    let mut match_result: BTreeSet<Match> = BTreeSet::new();

    let mut result = GraphDB::new();

    // create the subgraph description
    for r in plan {
        for i in match_idx.iter().cloned() {
            if i < r.len() {
                let m: &Match = &r[i];
                if !match_result.contains(m) {
                    match_result.insert(m.clone());
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
    for a in orig_db.node_annos.get_all(&id) {
        if let (Some(ns), Some(name), Some(val)) = (
            orig_db.strings.str(a.key.ns),
            orig_db.strings.str(a.key.name),
            orig_db.strings.str(a.val),
        ) {
            let new_anno = Annotation {
                key: AnnoKey {
                    ns: db.strings.add(ns),
                    name: db.strings.add(name),
                },
                val: db.strings.add(val),
            };
            db.node_annos.insert(id, new_anno);
        }
    }

    trace!(
        "adding node \"{}\" to subgraph",
        db.strings
            .str(
                db.node_annos
                    .get(&id, &db.get_node_name_key())
                    .cloned()
                    .unwrap_or(0)
            )
            .unwrap_or(&String::from(""))
    );
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
                                ns: db.strings.add(ns),
                                name: db.strings.add(name),
                            },
                            val: db.strings.add(val),
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

impl CorpusStorage {
    pub fn new(
        db_dir: &Path,
        max_allowed_cache_size: Option<usize>,
    ) -> Result<CorpusStorage, Error> {
        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            max_allowed_cache_size,
            corpus_cache: RwLock::new(LinkedHashMap::new()),
        };

        Ok(cs)
    }

    pub fn new_auto_cache_size(db_dir: &Path) -> Result<CorpusStorage, Error> {
        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            max_allowed_cache_size: Some(1024 * 1024 * 1024), // 1 GB
            corpus_cache: RwLock::new(LinkedHashMap::new()),
        };

        Ok(cs)
    }

    fn list_from_disk(&self) -> Result<Vec<String>, Error> {
        let mut corpora: Vec<String> = Vec::new();
        for c_dir in self.db_dir.read_dir()? {
            let c_dir = c_dir?;
            let ftype = c_dir.file_type()?;
            if ftype.is_dir() {
                let corpus_name = c_dir.file_name().into_string()?;
                corpora.push(corpus_name.clone());
            }
        }
        Ok(corpora)
    }

    pub fn list(&self) -> Result<Vec<CorpusInfo>, Error> {
        let names: Vec<String> = self.list_from_disk().unwrap_or_default();
        let mut result: Vec<CorpusInfo> = vec![];

        for n in names {
            let cache_entry = self.get_entry(&n)?;
            let lock = cache_entry.read().unwrap();
            let corpus_info: CorpusInfo = match &*lock {
                &CacheEntry::Loaded(ref db) => {
                    // check if all components are loaded
                    let heap_size = db.heap_size_of_children();
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

    fn get_entry(&self, corpus_name: &str) -> Result<Arc<RwLock<CacheEntry>>, Error> {
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

    fn get_loaded_entry(
        &self,
        corpus_name: &str,
        create_if_missing: bool,
    ) -> Result<Arc<RwLock<CacheEntry>>, Error> {
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
        }

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
                return Err(Error::NoSuchCorpus);
            }
        };

        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        let mut db = GraphDB::new();
        if create_corpus {
            db.save_to(&db_path)?;
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

    fn get_fully_loaded_entry(&self, corpus_name: &str) -> Result<Arc<RwLock<CacheEntry>>, Error> {
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
    pub fn delete(&self, corpus_name: &str) {
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
    }

    fn prepare_query<'a>(
        &self,
        corpus_name: &str,
        query_as_json: &'a str,
    ) -> Result<PreparationResult<'a>, Error> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;

        // make sure the database is loaded with all necessary components
        let (q, missing_components) = {
            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;
            let q = jsonqueryparser::parse(query_as_json, db).ok_or(Error::ParserError)?;
            let necessary_components = q.necessary_components();

            let mut missing: HashSet<Component> =
                HashSet::from_iter(necessary_components.iter().cloned());

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

    pub fn get_string(&self, corpus_name: &str, str_id: StringID) -> Result<String, Error> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;

        // accuire read-only lock and get string
        let lock = db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let result = db.strings
            .str(str_id)
            .cloned()
            .ok_or(Error::ImpossibleSearch(format!(
                "string with ID {} does not exist",
                str_id
            )))?;
        return Ok(result);
    }

    pub fn plan(&self, corpus_name: &str, query_as_json: &str) -> Result<String, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;

        // accuire read-only lock and plan
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db)?;

        return Ok(format!("{}", plan));
    }

    pub fn preload(&self, corpus_name: &str) -> Result<(), Error> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let mut lock = db_entry.write().unwrap();
        let db = get_write_or_error(&mut lock)?;
        db.ensure_loaded_all()?;
        return Ok(());
    }

    pub fn update_statistics(&self, corpus_name: &str) -> Result<(), Error> {
        let db_entry = self.get_loaded_entry(corpus_name, false)?;
        let mut lock = db_entry.write().unwrap();
        let db: &mut GraphDB = get_write_or_error(&mut lock)?;

        db.node_annos.calculate_statistics(&db.strings);
        for c in db.get_all_components(None, None).into_iter() {
            db.calculate_component_statistics(&c)?;
        }

        // TODO: persist changes

        Ok(())
    }

    pub fn count(&self, corpus_name: &str, query_as_json: &str) -> Result<u64, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db)?;

        return Ok(plan.count() as u64);
    }

    pub fn count_extra(&self, corpus_name: &str, query_as_json: &str) -> Result<CountExtra, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db: &GraphDB = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(&prep.query, &db)?;

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
    ) -> Result<Vec<String>, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;

        let plan = ExecutionPlan::from_disjunction(&prep.query, &db)?;

        let it: Vec<String> = plan.skip(offset)
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

                    if let Some(name_id) = db.node_annos
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
            })
            .collect();

        return Ok(it);
    }

    pub fn subgraph(
        &self,
        corpus_name: &str,
        node_ids: Vec<String>,
        ctx_left: usize,
        ctx_right: usize,
    ) -> Result<GraphDB, Error> {
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

            // left context
            {
                let mut q_left: Conjunction = Conjunction::new();
                let n_idx = q_left.add_node(NodeSearchSpec::ExactValue {
                    ns: Some(graphdb::ANNIS_NS.to_string()),
                    name: graphdb::NODE_NAME.to_string(),
                    val: Some(source_node_id.to_string()),
                    is_meta: false,
                });
                let tok_covered_idx = q_left.add_node(NodeSearchSpec::AnyToken);
                let tok_precedence_idx = q_left.add_node(NodeSearchSpec::AnyToken);
                let any_node_idx = q_left.add_node(NodeSearchSpec::AnyNode);

                q_left.add_operator(Box::new(operator::OverlapSpec {}), n_idx, tok_covered_idx);
                q_left.add_operator(
                    Box::new(operator::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_left,
                    }),
                    tok_precedence_idx,
                    tok_covered_idx,
                );
                q_left.add_operator(
                    Box::new(operator::OverlapSpec {}),
                    any_node_idx,
                    tok_precedence_idx,
                );

                query.alternatives.push(q_left);
            }

            // right context
            {
                let mut q_right: Conjunction = Conjunction::new();
                let n_idx = q_right.add_node(NodeSearchSpec::ExactValue {
                    ns: Some(graphdb::ANNIS_NS.to_string()),
                    name: graphdb::NODE_NAME.to_string(),
                    val: Some(source_node_id.to_string()),
                    is_meta: false,
                });
                let tok_covered_idx = q_right.add_node(NodeSearchSpec::AnyToken);
                let tok_precedence_idx = q_right.add_node(NodeSearchSpec::AnyToken);
                let any_node_idx = q_right.add_node(NodeSearchSpec::AnyNode);

                q_right.add_operator(Box::new(operator::OverlapSpec {}), n_idx, tok_covered_idx);
                q_right.add_operator(
                    Box::new(operator::PrecedenceSpec {
                        segmentation: None,
                        min_dist: 0,
                        max_dist: ctx_right,
                    }),
                    tok_covered_idx,
                    tok_precedence_idx,
                );
                q_right.add_operator(
                    Box::new(operator::OverlapSpec {}),
                    any_node_idx,
                    tok_precedence_idx,
                );

                query.alternatives.push(q_right);
            }
        }

        return extract_subgraph_by_query(db_entry, query, vec![3]);
    }

    pub fn subgraph_for_query(
        &self,
        corpus_name: &str,
        query_as_json: &str,
    ) -> Result<GraphDB, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;

        let mut max_alt_size = 0;
        for alt in prep.query.alternatives.iter() {
            max_alt_size = std::cmp::max(max_alt_size, alt.num_of_nodes());
        }

        return extract_subgraph_by_query(
            prep.db_entry.clone(),
            prep.query,
            (0..max_alt_size).collect(),
        );
    }
    pub fn subcorpus_graph(
        &self,
        corpus_name: &str,
        corpus_ids: Vec<String>,
    ) -> Result<GraphDB, Error> {
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
            let corpus_idx = q.add_node(NodeSearchSpec::ExactValue {
                ns: Some(graphdb::ANNIS_NS.to_string()),
                name: graphdb::NODE_NAME.to_string(),
                val: Some(source_corpus_id.to_string()),
                is_meta: false,
            });
            let any_node_idx = q.add_node(NodeSearchSpec::AnyNode);
            q.add_operator(
                Box::new(operator::PartOfSubCorpusSpec::new(1, 1)),
                any_node_idx,
                corpus_idx,
            );
            query.alternatives.push(q);
        }

        return extract_subgraph_by_query(db_entry, query, vec![1]);
    }

    pub fn corpus_graph(&self, corpus_name: &str) -> Result<GraphDB, Error> {
        let db_entry = self.get_fully_loaded_entry(corpus_name)?;

        let mut query = Conjunction::new();

        query.add_node(NodeSearchSpec::new_exact(
            Some(ANNIS_NS),
            NODE_TYPE,
            Some("corpus"),
            false,
        ));

        return extract_subgraph_by_query(db_entry, query.into_disjunction(), vec![0]);
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

    pub fn apply_update(&self, corpus_name: &str, update: &mut GraphUpdate) -> Result<(), Error> {
        let db_entry = self.get_loaded_entry(corpus_name, true)?;
        {
            let mut lock = db_entry.write().unwrap();
            let db: &mut GraphDB = get_write_or_error(&mut lock)?;

            db.apply_update(update)?;
        }
        // start background thread to persists the results
        std::thread::spawn(move || {
            let lock = db_entry.read().unwrap();
            if let Ok(db) = get_read_or_error(&lock) {
                let db: &GraphDB = db;
                if let Err(e) = db.background_sync_wal_updates() {
                    error!("Can't sync changes in background thread: {:?}", e);
                }
            }
        });

        Ok(())
    }
}
