pub mod token_helper;
pub mod memory_estimation;

use {Annotation, AnnoKey};
use graphdb::GraphDB;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std;

pub fn regex_full_match(pattern: &str) -> String {
    let mut full_match_pattern = String::new();
    full_match_pattern.push_str(r"\A(");
    full_match_pattern.push_str(pattern);
    full_match_pattern.push_str(r")\z");

    full_match_pattern
}

// pub fn contains_regex_metacharacters(pattern: &str) -> bool {

// }

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
                    let name : &OsStr = p_aql.file_stem().unwrap_or(&unknown_name);
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

pub fn get_queries_from_folder(folder: &Path, panic_on_invalid : bool) -> Box<Iterator<Item = SearchDef>> {
    // get an iterator over all files in the folder
    if let Ok(it_folder) = folder.read_dir() {
        // filter by file type and read both the ".aql", ".json" and ".count" files
        let it = it_folder.filter_map(move |e| -> Option<SearchDef> {
            if let Ok(e) = e {
                let p = e.path();
                if p.exists() && p.is_file()
                    && p.extension() == Some(&OsString::from("aql"))
                {
                    let r = SearchDef::from_file(&p);
                    if panic_on_invalid {
                        let r = r.expect(&format!("Search definition for query {} is incomplete", p.to_string_lossy()));
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

pub fn qname_to_anno_key(qname : &str, db : &GraphDB) -> Option<AnnoKey> {
    // split qname
    let qname : Vec<&str> = qname.splitn(2, "::").collect();
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


