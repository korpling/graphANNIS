//! An API for managing corpora stored in a common location on the file system.
//! It is transactional and thread-safe.

use std::sync::{Arc,RwLock};
use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use graphdb::GraphDB;
//use {Annotation, Match, NodeID, StringID, AnnoKey};

#[derive(Clone)]
enum LoadStatus {
    NotLoaded(PathBuf),
    NodesLoaded(Arc<GraphDB>),
    FullyLoaded(Arc<GraphDB>),
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
    pub fn new(db_dir : &Path, max_allowed_cache_size : Option<usize>) -> CorpusStorage {

        CorpusStorage {
            db_dir: PathBuf::from(db_dir),
            max_allowed_cache_size,
            corpus_cache: RwLock::new(BTreeMap::new()),
        }
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
    pub fn import(&mut self, path_to_corpus : &Path, new_corpus_name : &str) {
        let corpus = self.get_corpus_from_cache(new_corpus_name);
        let corpus = load_corpus(corpus);
        // TODO: load the corpus data from the external location
        
        // make sure the corpus is properly saved at least once (so it is in a consistent state)
        corpus.persist();
    }
}
