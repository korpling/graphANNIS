//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use std::sync::{Arc,RwLock};
use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use graphdb::GraphDB;
use graphdb;
use relannis;
use std;
//use {Annotation, Match, NodeID, StringID, AnnoKey};

#[derive(Clone)]
enum LoadStatus {
    NotLoaded(PathBuf),
    NodesLoaded(Arc<GraphDB>),
    FullyLoaded(Arc<GraphDB>),
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
    db_dir : PathBuf,
    max_allowed_cache_size : Option<usize>,

    corpus_cache: RwLock<BTreeMap<String, LoadStatus>>,
}

fn load_corpus(status : LoadStatus) -> Arc<GraphDB> {
        let result = match status {
            LoadStatus::NotLoaded(location) => {
                let mut db = GraphDB::new();
                db.load_from(&location, false);
                Arc::new(db)
            },
            LoadStatus::NodesLoaded(corpus) | LoadStatus::FullyLoaded(corpus) => corpus, 
        };

        return result;
}



impl CorpusStorage {
    pub fn new(db_dir : &Path, max_allowed_cache_size : Option<usize>) -> Result<CorpusStorage, Error> {

        let mut cs = CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            max_allowed_cache_size,
            corpus_cache: RwLock::new(BTreeMap::new()),
        };

        cs.load_available_from_disk()?;

        Ok(cs)
    }

    fn load_available_from_disk(&mut self) -> Result<(),Error> {
        let mut cache_lock =  self.corpus_cache.write().unwrap();
        let cache : &mut BTreeMap<String, LoadStatus> = &mut *cache_lock;

        for c_dir in self.db_dir.read_dir()? {
            let c_dir = c_dir?;
            let ftype = c_dir.file_type()?;
            if ftype.is_dir()  {
                cache.insert(c_dir.file_name().into_string()?, LoadStatus::NotLoaded(c_dir.path()));
            }
        }

        Ok(())
    }

    fn get_corpus_from_cache(&mut self, corpus_name : &str) -> LoadStatus {
        let mut cache_lock =  self.corpus_cache.write().unwrap();
        
        let cache : &mut BTreeMap<String, LoadStatus> = &mut *cache_lock;
        
        let entry = cache.entry(String::from(corpus_name)).or_insert_with(|| {
            // Create a new LoadStatus and put it into the cache. This will not load
            // the database itself, this can be done with the resulting object from the caller.
            let db_path : PathBuf = [self.db_dir.to_string_lossy().as_ref(), corpus_name].iter().collect();
            LoadStatus::NotLoaded(db_path)
        });
        return entry.clone();
    }



    /// Import a corpus from an external location into this corpus storage
    pub fn import_from_dir(&mut self, new_corpus_name : &str, path_to_corpus : &Path) {
        let corpus = self.get_corpus_from_cache(new_corpus_name);
        let corpus = load_corpus(corpus);
        
        // TODO: load the corpus data from the external location      
//        corpus.load_from(path_to_corpus, false);

        // make sure the corpus is properly saved at least once (so it is in a consistent state)
        corpus.persist();
        unimplemented!();
    }

    /// Import a corpus in relANNIS format from an external location into this corpus storage
    pub fn import(&mut self, corpus_name : &str, mut db : GraphDB) {

        let r = db.ensure_loaded_all();
        
        let mut db_path = PathBuf::from(&self.db_dir);
        db_path.push(corpus_name);
        
        let mut cache_lock =  self.corpus_cache.write().unwrap();
        let cache : &mut BTreeMap<String, LoadStatus> = &mut *cache_lock;
        
        // remove any possible old corpus 
        let old_entry = cache.remove(corpus_name);
        if let Some(old_db) = old_entry {
            // TODO: remove the folder from disk
        }

        if let Err(e) = std::fs::create_dir_all(&db_path) {
             error!("Can't create directory {}: {:?}", db_path.to_string_lossy(), e);
        }        

        // save to its location
        let save_result = db.save_to(&db_path);
        if let Err(e) = save_result {
            error!("Can't save corpus to {}: {:?}", db_path.to_string_lossy(), e);
        }

        // make it known to the cache
        if let Err(e) = r {
            error!("Some error occured when attempting to load components from disk: {:?}", e);
            cache.insert(String::from(corpus_name), LoadStatus::NodesLoaded(Arc::new(db)));
        } else {
            cache.insert(String::from(corpus_name), LoadStatus::FullyLoaded(Arc::new(db)));
        }



    }
}
