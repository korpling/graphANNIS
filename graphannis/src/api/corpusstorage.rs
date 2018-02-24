//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use {Component, Match, StringID};
use parser::jsonqueryparser;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashSet};
use graphdb;
use graphdb::{GraphDB};
use std;
use plan;
use plan::ExecutionPlan;
use query::disjunction::Disjunction;

use std::iter::FromIterator;

//use {Annotation, Match, NodeID, StringID, AnnoKey};

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
                let error_msg : Vec<String> = error_vec.into_iter().map(|x| format!("{:?}", x)).collect();
                Error::ImpossibleSearch(error_msg.join("\n"))
            },
        }
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(e: std::ffi::OsString) -> Error {
        Error::StringConvert(e)
    }
}



pub struct CorpusStorage {
    db_dir: PathBuf,
/*    max_allowed_cache_size: Option<usize>, */
    corpus_cache: RwLock<BTreeMap<String, Arc<RwLock<CacheEntry>>>>,
}


struct PreparationResult<'a> {
    query: Disjunction<'a>,
    db_entry : Arc<RwLock<CacheEntry>>,
}

fn get_read_or_error<'a>(lock : &'a RwLockReadGuard<CacheEntry>) -> Result<&'a GraphDB, Error> {
    if let &CacheEntry::Loaded(ref db) = &**lock {            
        return Ok(db);
    } else {
        return Err(Error::LoadingFailed);
    }
}

fn get_write_or_error<'a>(lock : &'a mut RwLockWriteGuard<CacheEntry>) -> Result<&'a mut GraphDB, Error> {
    if let &mut CacheEntry::Loaded(ref mut db) = &mut **lock {            
        return Ok(db);
    } else {
        return Err(Error::LoadingFailed);
    }
}

impl CorpusStorage {
    pub fn new(
        db_dir: &Path,
/*        max_allowed_cache_size: Option<usize>, */
    ) -> Result<CorpusStorage, Error> {
        let cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
/*            max_allowed_cache_size, */
            corpus_cache: RwLock::new(BTreeMap::new()),
        };

        Ok(cs)
    }

    fn list_from_disk(&self) -> Result<Vec<String>,Error> {
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

    pub fn list(&self) -> Vec<String> {
        let result: Vec<String> = self.list_from_disk().unwrap_or_default();
        return result;
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
        let db_path: PathBuf = [self.db_dir.to_string_lossy().as_ref(), &corpus_name]
                    .iter()
                    .collect();
        
        if !db_path.is_dir() {
            return Err(Error::NoSuchCorpus);
        }

        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;
            
        let entry = cache.entry(corpus_name.clone()).or_insert_with(|| {
            Arc::new(RwLock::new(CacheEntry::NotLoaded))
        });
        
        return Ok(entry.clone());
    }

     fn get_loaded_entry(&self, corpus_name : &str) -> Result<Arc<RwLock<CacheEntry>>, Error> {

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

        // get write-lock and load entry
        let db_path: PathBuf = [self.db_dir.to_string_lossy().as_ref(), &corpus_name]
                    .iter()
                    .collect();
        
        if !db_path.is_dir() {
            return Err(Error::NoSuchCorpus);
        }

        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;
        let mut db = GraphDB::new();

        db.load_from(&db_path, false)?;
        
        let entry = Arc::new(RwLock::new(CacheEntry::Loaded(db)));
        cache.insert(String::from(corpus_name), entry.clone());
        return Ok(entry);
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
            // TODO: remove the folder from disk
        }

        if let Err(e) = std::fs::create_dir_all(&db_path) {
            error!(
                "Can't create directory {}: {:?}",
                db_path.to_string_lossy(),
                e
            );
        }

        // always calculate statistics

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
    }

    fn prepare_query<'a>(
        &self,
        corpus_name: &str,
        query_as_json: &'a str,
    ) -> Result<PreparationResult<'a>, Error> {

       
        let db_entry = self.get_loaded_entry(corpus_name)?;
        
        // make sure the database is loaded with all necessary components
        let (q, missing_components) = {

            let lock = db_entry.read().unwrap();
            let db = get_read_or_error(&lock)?;
            let q = jsonqueryparser::parse(query_as_json, db).ok_or(Error::ParserError)?;
            let necessary_components = q.necessary_components();

            let mut missing: HashSet<Component> = HashSet::from_iter(necessary_components.iter().cloned());

            // remove all that are already loaded
            for c in necessary_components.iter() {
                if db.get_graphstorage(c).is_some() {
                    missing.remove(c);
                }
            }
            let missing : Vec<Component> = missing.into_iter().collect();
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

        return Ok(PreparationResult {
            query: q,
            db_entry,
        });
    }

    pub fn get_string(&self, corpus_name : &str, str_id : StringID) -> Result<String, Error> {
        let db_entry = self.get_loaded_entry(corpus_name)?;

        // accuire read-only lock and get string
        let lock = db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let result = db.strings.str(str_id).cloned().ok_or(Error::ImpossibleSearch(format!("string with ID {} does not exist", str_id)))?;
        return Ok(result);
    }

    pub fn plan(&self, corpus_name: &str, query_as_json: &str) -> Result<String, Error> {
        let prep = self.prepare_query(corpus_name, query_as_json)?;
        

        // accuire read-only lock and plan
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(prep.query, &db)?;

        return Ok(format!("{}", plan));
     }

    pub fn preload(&self, corpus_name: &str) -> Result<(), Error> {

        let db_entry = self.get_loaded_entry(corpus_name)?;
        let mut lock = db_entry.write().unwrap();
        let db = get_write_or_error(&mut lock)?;
        db.ensure_loaded_all()?;
        return Ok(());
    }

     pub fn update_statistics(&self, corpus_name : &str) -> Result<(), Error> {
        let db_entry = self.get_loaded_entry(corpus_name)?;
         let mut lock = db_entry.write().unwrap();
         let db : &mut GraphDB = get_write_or_error(&mut lock)?;

        db.node_annos.calculate_statistics(&db.strings);
        for c in db.get_all_components(None, None).into_iter() {
            db.calculate_component_statistics(c)?;
        }

        // TODO: persist changes

        Ok(())
    }

    pub fn count(&self, corpus_name: &str, query_as_json: &str) -> Result<usize, Error> {

        let prep = self.prepare_query(corpus_name, query_as_json)?;
        

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;
        let plan = ExecutionPlan::from_disjunction(prep.query, &db)?;

        return Ok(plan.count());
        

    }

    pub fn find(&self, corpus_name: &str, query_as_json: &str, offset : usize, limit : usize) -> Result<Vec<String>, Error> {

        let prep = self.prepare_query(corpus_name, query_as_json)?;
        

        // accuire read-only lock and execute query
        let lock = prep.db_entry.read().unwrap();
        let db = get_read_or_error(&lock)?;

        let plan = ExecutionPlan::from_disjunction(prep.query, &db)?;

        let it : Vec<String> = plan.skip(offset).take(limit).map(|m : Vec<Match>| {
            let mut match_desc : Vec<String> = Vec::new();
            for singlematch in m.iter() {
                let mut node_desc = String::from("salt:/");
                if let Some(name_id) = db.node_annos.get(&singlematch.node, &db.get_node_name_key()) {
                    if let Some(name) = db.strings.str(name_id.clone()) {
                        node_desc.push_str(name);
                    }
                }
                match_desc.push(node_desc);
            }
            let mut result = String::new();
            result.push_str(&match_desc.join(" "));
            return result;
        }).collect();

        return Ok(it);
        

    }
}
