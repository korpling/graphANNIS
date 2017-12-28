//! An API for managing a so-called corpus-storage, which is a folder on the file system that contains multiple corpora.
//! This API is transactional and allows multiple reads for the same corpus at the same time.

use std::path::PathBuf;

pub struct CorpusStorage {
    db_dir : PathBuf,
}

impl CorpusStorage {
    pub fn new(db_dir : PathBuf) -> CorpusStorage {

        CorpusStorage {
            db_dir,
        }
    }
}