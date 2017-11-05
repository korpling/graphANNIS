use graphdb::GraphDB;
use annis::{AnnoKey, Annotation};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::collections::BTreeMap;
use std;

pub struct RelANNISLoader;


type Result<T> = std::result::Result<T, std::io::Error>;


fn load_corpus_tab(path : &PathBuf, 
    corpus_by_preorder : &mut BTreeMap<u32, u32>,
    corpus_id_to_name : &mut BTreeMap<u32, String>,
    is_annis_33 : bool) -> Result<String> {

    let corpus_tab_path = PathBuf::from(path).push(if is_annis_33 {"corpus.annis"} else {"corpus.tab"});
    

    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Can't read corpus table"));
}

    

pub fn load(path : &str) -> Result<GraphDB>{
    
    // convert to path
    let mut path = PathBuf::from(path);
    if path.is_dir() && path.exists() {
        // check if this is the ANNIS 3.3 import format
        path.push("annis.version");
        let mut is_annis_33 = false;
        if path.exists() {
            let mut file = File::open(&path)?;
            let mut version_str = String::new();
            file.read_to_string(&mut version_str)?;
            
            let is_annis_33 = version_str == "3.3";
        }

        let mut db = GraphDB::new();

        let mut corpus_by_preorder = BTreeMap::new();
        let mut corpus_id_to_name = BTreeMap::new();
        let corpus_name = load_corpus_tab(&path, &mut corpus_by_preorder, &mut corpus_id_to_name, is_annis_33)?;

        return Ok(db);
    }


    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Could not find path {:?}", path.to_str())));
}

