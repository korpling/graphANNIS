use graphdb::GraphDB;
use annis::{AnnoKey, Annotation};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use std::num::ParseIntError;
use std::collections::BTreeMap;
use std;
use csv;

pub struct RelANNISLoader;

#[derive(Debug)]
pub enum RelANNISError {
    IOError(std::io::Error),
    CSVError(csv::Error),
    MissingColumn,
    InvalidDataType,
    ToplevelCorpusNameNotFound,
    DirectoryNotFound,
    Other,
}

type Result<T> = std::result::Result<T, RelANNISError>;

impl From<ParseIntError> for RelANNISError {
    fn from(_: ParseIntError) -> RelANNISError {
        RelANNISError::InvalidDataType
    }
}

impl From<csv::Error> for RelANNISError {
    fn from(e: csv::Error) -> RelANNISError {
        RelANNISError::CSVError(e)
    }
}

impl From<std::io::Error> for RelANNISError {
    fn from(e: std::io::Error) -> RelANNISError {
        RelANNISError::IOError(e)
    }
}

fn load_corpus_tab(path : &PathBuf, 
    corpus_by_preorder : &mut BTreeMap<u32, u32>,
    corpus_id_to_name : &mut BTreeMap<u32, String>,
    is_annis_33 : bool) -> Result<String> {

    let mut corpus_tab_path = PathBuf::from(path);
    corpus_tab_path.push(if is_annis_33 {"corpus.annis"} else {"corpus.tab"});
    
    let mut toplevel_corpus_name : Option<String> = None;

    let mut corpus_tab_csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(corpus_tab_path.as_path())?;
        
    for result in corpus_tab_csv.records() {
        let line = result?;

        let id = line.get(0).ok_or(RelANNISError::MissingColumn)?.parse::<u32>()?;
        let name = line.get(1).ok_or(RelANNISError::MissingColumn)?;
        let type_str = line.get(2).ok_or(RelANNISError::MissingColumn)?;
        let pre_order = line.get(4).ok_or(RelANNISError::MissingColumn)?.parse::<u32>()?;

        corpus_id_to_name.insert(id, String::from(name));
        if type_str == "CORPUS" && pre_order == 0 {
            toplevel_corpus_name = Some(String::from(name));
            corpus_by_preorder.insert(pre_order, id);
        } else if type_str == "DOCUMENT" {
            // TODO: do not only add documents but also sub-corpora
            corpus_by_preorder.insert(pre_order, id);
        }

    }

    toplevel_corpus_name.ok_or(RelANNISError::ToplevelCorpusNameNotFound)
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

    return Err(RelANNISError::DirectoryNotFound);
}

