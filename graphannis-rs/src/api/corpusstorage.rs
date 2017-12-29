//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use std::sync::{Arc, RwLock, Mutex};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use graphdb::GraphDB;
use graphdb;
use relannis;
use std;
use query::conjunction::Conjunction;

//use {Annotation, Match, NodeID, StringID, AnnoKey};


enum LoadStatus {
    NotLoaded{corpus_name : String, db_path : PathBuf},
    NodesLoaded(Arc<GraphDB>),
    FullyLoaded(Arc<GraphDB>),
}

struct DBLoader {
    db : Option<GraphDB>,
    db_path: PathBuf,
}

impl DBLoader {
    fn get<'a>(&'a mut self) -> &'a GraphDB {
        if self.db.is_none() {
            let mut loaded_db = GraphDB::new();
            // TODO: what if loading fails?
            loaded_db.load_from(&self.db_path, false);
            self.db = Some(loaded_db);
        }
        return  &self.db.as_ref().unwrap();
    }
    // TODO: add callback
}

#[derive(Debug)]
pub enum Error {
    IOerror(std::io::Error),
    DBError(graphdb::Error),
    StringConvert(std::ffi::OsString),
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

impl From<std::ffi::OsString> for Error {
    fn from(e: std::ffi::OsString) -> Error {
        Error::StringConvert(e)
    }
}



pub struct CorpusStorage {
    db_dir: PathBuf,
    max_allowed_cache_size: Option<usize>,

    corpus_cache: RwLock<BTreeMap<String, Arc<Mutex<DBLoader>>>>,
}



impl CorpusStorage {
    pub fn new(
        db_dir: &Path,
        max_allowed_cache_size: Option<usize>,
    ) -> Result<CorpusStorage, Error> {
        let mut cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            max_allowed_cache_size,
            corpus_cache: RwLock::new(BTreeMap::new()),
        };

        cs.load_available_from_disk()?;

        Ok(cs)
    }

    fn load_available_from_disk(&mut self) -> Result<(), Error> {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache: &mut BTreeMap<String, Arc<Mutex<DBLoader>>> = &mut *cache_lock;

        for c_dir in self.db_dir.read_dir()? {
            let c_dir = c_dir?;
            let ftype = c_dir.file_type()?;
            if ftype.is_dir() {
                let corpus_name  = c_dir.file_name().into_string()?;
                cache.insert(
                    corpus_name.clone(),
                    Arc::new(Mutex::new(DBLoader{db: None, db_path: c_dir.path()})),
                );
            }
        }

        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        if let Ok(cache_lock) = self.corpus_cache.read() {
            let cache = &*cache_lock;
            result = cache.keys().cloned().collect();
        }

        return result;
    }


    fn get_loader(&mut self, corpus_name: &str) -> Arc<Mutex<DBLoader>> {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache: & mut BTreeMap<String, Arc<Mutex<DBLoader>>> = &mut *cache_lock;

        let corpus_name = corpus_name.to_string();
        let entry = cache.entry(corpus_name.clone()).or_insert_with(|| {
            // Create a new LoadStatus and put it into the cache. This will not load
            // the database itself, this can be done with the resulting object from the caller.
            let db_path: PathBuf = [self.db_dir.to_string_lossy().as_ref(), &corpus_name]
                .iter()
                .collect();
            Arc::new(Mutex::new(DBLoader{db: None, db_path}))
        });

        return entry.clone();
    }


    /// Import a corpus in relANNIS format from an external location into this corpus storage
    pub fn import(&mut self, corpus_name: &str, mut db: GraphDB) {
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
        let cache: &mut BTreeMap<String, Arc<Mutex<DBLoader>>> = &mut *cache_lock;

        // remove any possible old corpus
        let old_entry = cache.remove(corpus_name);
        if let Some(old_db) = old_entry {
            // TODO: remove the folder from disk
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
            Arc::new(Mutex::new(DBLoader{db: Some(db), db_path: db_path}))
        );


    }

    pub fn count(&mut self, corpus_name: &str, query_as_json: &str) {
        let db_loader = self.get_loader(corpus_name);

//        let db = self.load_corpus(db_loader);
        // TODO: actually parse the JSON and create query

        let q = Conjunction::new();
    }
}
