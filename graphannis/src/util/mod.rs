pub mod memory_estimation;
pub mod sort_matches;
pub mod token_helper;

use graphdb::GraphDB;
use regex_syntax;
use {AnnoKey, Annotation};

use std;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn regex_full_match(pattern: &str) -> String {
    let mut full_match_pattern = String::new();
    full_match_pattern.push_str(r"\A(");
    full_match_pattern.push_str(pattern);
    full_match_pattern.push_str(r")\z");

    full_match_pattern
}

pub fn contains_regex_metacharacters(pattern: &str) -> bool {
    for c in pattern.chars() {
        if regex_syntax::is_meta_character(c) {
            return true;
        }
    }
    return false;
}

pub fn check_annotation_key_equal(a: &Annotation, b: &Annotation) -> bool {
    // compare by name (non lexical but just by the ID)
    if a.key.name != 0 && b.key.name != 0 && a.key.name != b.key.name {
        return false;
    }

    // if equal, compare by namespace (non lexical but just by the ID)
    if a.key.ns != 0 && b.key.ns != 0 && a.key.ns != b.key.ns {
        return false;
    }

    // they are equal
    return true;
}

pub fn check_annotation_equal(a: &Annotation, b: &Annotation) -> bool {
    // compare by name (non lexical but just by the ID)
    if a.key.name != 0 && b.key.name != 0 && a.key.name != b.key.name {
        return false;
    }

    // if equal, compare by namespace (non lexical but just by the ID)
    if a.key.ns != 0 && b.key.ns != 0 && a.key.ns != b.key.ns {
        return false;
    }

    // if still equal compare by value (non lexical but just by the ID)
    if a.val != 0 && b.val != 0 && a.val != b.val {
        return false;
    }

    // they are equal
    return true;
}

/// Takes a node name/ID and extracts both the document path as array and the node name itself.
/// 
/// Complete node names/IDs have the form `toplevel-corpus/sub-corpus/.../document#node-name`,
/// mimicking simple URIs (without a scheme) and a limited set of allowed characters as corpus/document names.
/// The part before the fragment is returned as vector, the latter one as string.
///
/// # Examples
/// ```
/// extern crate graphannis;
/// use graphannis::util;
///
/// let full_node_name = "toplevel/subcorpus1/subcorpus2/doc1#mynode";
/// let (path, name) = util::extract_node_path(full_node_name);
///
/// assert_eq!(name, "mynode");
/// assert_eq!(path, vec!["toplevel", "subcorpus1", "subcorpus2", "doc1"]);
/// 
/// let full_doc_name = "toplevel/subcorpus1/subcorpus2/doc1";
/// let (path, name) = util::extract_node_path(full_doc_name);
/// 
/// assert_eq!(name, "");
/// assert_eq!(path, vec!["toplevel", "subcorpus1", "subcorpus2", "doc1"]);
/// ```
pub fn extract_node_path(full_node_name: &str) -> (Vec<&str>, &str) {
    // separate path and name first
    let hash_pos = full_node_name.rfind('#');

    let path_str : &str = &full_node_name[0..hash_pos.unwrap_or(full_node_name.len())];
    let mut path: Vec<&str> = Vec::with_capacity(4);
    path.extend(path_str
         .split('/')
         .filter(|s| !s.is_empty())
    );

    if let Some(hash_pos) = hash_pos {
        return (path, &full_node_name[hash_pos+1..]);
    } else {
        return (path, "");
    }
    
}

pub struct SearchDef {
    pub aql: String,
    pub json: String,
    pub count: u64,
    pub name: String,
}

impl SearchDef {
    pub fn from_file(base: &Path) -> Option<SearchDef> {
        let mut p_aql = PathBuf::from(base);
        p_aql.set_extension("aql");

        let mut p_json = PathBuf::from(base);
        p_json.set_extension("json");

        let mut p_count = PathBuf::from(base);
        p_count.set_extension("count");

        let f_aql = File::open(p_aql.clone());
        let f_json = File::open(p_json);
        let f_count = File::open(p_count);

        if let (Ok(mut f_aql), Ok(mut f_json), Ok(mut f_count)) = (f_aql, f_json, f_count) {
            let mut aql = String::new();
            let mut json = String::new();
            let mut count = String::new();

            if let (Ok(_), Ok(_), Ok(_)) = (
                f_aql.read_to_string(&mut aql),
                f_json.read_to_string(&mut json),
                f_count.read_to_string(&mut count),
            ) {
                // try to parse the count value
                if let Ok(count) = count.trim().parse::<u64>() {
                    let unknown_name = OsString::from("<unknown>");
                    let name: &OsStr = p_aql.file_stem().unwrap_or(&unknown_name);
                    return Some(SearchDef {
                        aql: String::from(aql.trim()),
                        json: String::from(json.trim()),
                        count,
                        name: String::from(name.to_string_lossy()),
                    });
                }
            }
        }

        return None;
    }
}

pub fn get_queries_from_folder(
    folder: &Path,
    panic_on_invalid: bool,
) -> Box<Iterator<Item = SearchDef>> {
    // get an iterator over all files in the folder
    if let Ok(it_folder) = folder.read_dir() {
        // filter by file type and read both the ".aql", ".json" and ".count" files
        let it = it_folder.filter_map(move |e| -> Option<SearchDef> {
            if let Ok(e) = e {
                let p = e.path();
                if p.exists() && p.is_file() && p.extension() == Some(&OsString::from("aql")) {
                    let r = SearchDef::from_file(&p);
                    if panic_on_invalid {
                        let r = r.expect(&format!(
                            "Search definition for query {} is incomplete",
                            p.to_string_lossy()
                        ));
                        return Some(r);
                    } else {
                        return r;
                    }
                }
            }

            return None;
        });

        return Box::from(it);
    }

    return Box::new(std::iter::empty());
}

pub fn qname_to_anno_key(qname: &str, db: &GraphDB) -> Option<AnnoKey> {
    // split qname
    let qname: Vec<&str> = qname.splitn(2, "::").collect();
    if qname.len() == 1 {
        return Some(AnnoKey {
            ns: db.strings.find_id("")?.clone(),
            name: db.strings.find_id(qname[0])?.clone(),
        });
    } else if qname.len() == 2 {
        return Some(AnnoKey {
            ns: db.strings.find_id(qname[0])?.clone(),
            name: db.strings.find_id(qname[1])?.clone(),
        });
    }
    None
}
