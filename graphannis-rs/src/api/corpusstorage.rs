//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use {Component};
use parser::jsonqueryparser;
use std::sync::{Arc, RwLock};
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


struct DBLoader {
    db: Option<GraphDB>,
    db_path: PathBuf,
}

impl DBLoader {
    fn needs_loading(&self, components: &Vec<Component>) -> bool {
        
        if let Some(db) = self.db.as_ref() {
            let mut missing: HashSet<Component> = HashSet::from_iter(components.iter().cloned());

            // remove all that are already loaded
            for c in components.iter() {
                if db.get_graphstorage(c).is_some() {
                    missing.remove(c);
                }
            }

            return missing.len() > 0;
        } else {
            return true;
        }
    }

    /// Get the database without loading components of it.
    fn get<'a>(&'a self) -> Option<&'a GraphDB> {
        return self.db.as_ref();
    }


    /// Get the database and load components (and itself) when necessary
    fn get_with_components_loaded<'a, I>(&'a mut self, components: I) -> Option<&'a GraphDB>
    where
        I: Iterator<Item = &'a Component>,
    {
        if self.db.is_none() {
            let mut loaded_db = GraphDB::new();
            // TODO: what if loading fails?
            loaded_db.load_from(&self.db_path, false).ok()?;
            self.db = Some(loaded_db);
        }
        {
            let mut_db: &mut GraphDB = self.db.as_mut().unwrap();
            for c in components {
                // TODO: what if loading fails?
                mut_db.ensure_loaded(c).ok()?;
            }
        }
        return self.db.as_ref();
    }
    // TODO: add callback
}

#[derive(Debug)]
pub enum Error {
    IOerror(std::io::Error),
    DBError(graphdb::Error),
    LoadingFailed,
    ImpossibleSearch,
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
            plan::Error::ImpossibleSearch => Error::ImpossibleSearch,
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

    corpus_cache: RwLock<BTreeMap<String, Arc<RwLock<DBLoader>>>>,
}


struct PreparationResult<'a> {
    query: Disjunction<'a>,
    db_loader : Arc<RwLock<DBLoader>>,
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

        cs.load_available_from_disk()?;

        Ok(cs)
    }

    fn load_available_from_disk(&self) -> Result<(), Error> {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        for c_dir in self.db_dir.read_dir()? {
            let c_dir = c_dir?;
            let ftype = c_dir.file_type()?;
            if ftype.is_dir() {
                let corpus_name = c_dir.file_name().into_string()?;
                cache.insert(
                    corpus_name.clone(),
                    Arc::new(RwLock::new(DBLoader {
                        db: None,
                        db_path: c_dir.path(),
                    })),
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

/*
    fn get_or_create_loader(&self, corpus_name: &str) -> Arc<RwLock<DBLoader>> {
        let mut cache_lock = self.corpus_cache.write().unwrap();
        let cache = &mut *cache_lock;

        let corpus_name = corpus_name.to_string();
        let entry = cache.entry(corpus_name.clone()).or_insert_with(|| {
            // Create a new empty DB loader and put it into the cache. This will not load
            // the database itself, this can be done with the resulting object from the caller.
            let db_path: PathBuf = [self.db_dir.to_string_lossy().as_ref(), &corpus_name]
                .iter()
                .collect();
            Arc::new(RwLock::new(DBLoader { db: None, db_path }))
        });

        return entry.clone();
    }
*/
    fn get_loader(&self, corpus_name: &str) -> Option<Arc<RwLock<DBLoader>>> {

        let corpus_name = corpus_name.to_string();

        let cache_lock = self.corpus_cache.read().unwrap();
        let cache = &*cache_lock;

        let entry = cache.get(&corpus_name);
        return entry.cloned();
    }


    /// Import a corpus in relANNIS format from an external location into this corpus storage
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
            Arc::new(RwLock::new(DBLoader {
                db: Some(db),
                db_path: db_path,
            })),
        );
    }

    fn prepare_query<'a>(
        &self,
        corpus_name: &str,
        query_as_json: &'a str,
    ) -> Result<PreparationResult<'a>, Error> {

        let db_loader = self.get_loader(corpus_name).ok_or(Error::NoSuchCorpus)?;

        // make sure the database is loaded at all
        let needs_loading_base = {
            let lock = db_loader.read().unwrap();
            (&*lock).get().is_none()
        };
        if needs_loading_base {
            let mut lock = db_loader.write().unwrap();
            (&mut *lock).get_with_components_loaded(std::iter::empty());
        };

        // make sure the database is loaded with all necessary components
        let (q, needs_loading, necessary_components) = {
            let lock = db_loader.read().unwrap();
            
            let db = (&*lock).get().ok_or(Error::LoadingFailed)?;
            let q = jsonqueryparser::parse(query_as_json, db).ok_or(Error::ParserError)?;
            let necessary_components = q.necessary_components();
            let needs_loading = (&*lock).needs_loading(&necessary_components);

            (q, needs_loading, necessary_components)
        };
        if needs_loading {
            // load the needed components
            let mut lock = db_loader.write().unwrap();
            (&mut *lock).get_with_components_loaded(necessary_components.iter());
        };

        return Ok(PreparationResult {
            query: q,
            db_loader: db_loader,
        });
    }

    pub fn count(&self, corpus_name: &str, query_as_json: &str) -> Result<usize, Error> {

        let prep = self.prepare_query(corpus_name, query_as_json)?;
        

        // accuire read-only lock and execute query
        let lock = prep.db_loader.read().unwrap();
        let db: &GraphDB = (&*lock).get().unwrap();

        let plan = ExecutionPlan::from_disjunction(prep.query, db)?;

        return Ok(plan.count());
        

    }
}
