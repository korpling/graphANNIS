pub mod memory_estimation;
pub mod quicksort;

use regex_syntax;
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
    false
}

pub fn split_qname(qname: &str) -> (Option<&str>, &str) {
    let sep_pos = qname.find("::.+");
    if let Some(sep_pos) = sep_pos {
        return (Some(&qname[..sep_pos]), &qname[sep_pos + 1..]);
    } else {
        return (None, qname);
    }
}

/// Defines a definition of a query including its number of expected results.
#[derive(Debug)]
pub struct SearchDef {
    pub aql: String,
    pub count: u64,
    pub name: String,
}

impl SearchDef {
    /// Create a definition of a query by a file name.
    pub fn from_file(base: &Path) -> Option<SearchDef> {
        let mut p_aql = PathBuf::from(base);
        p_aql.set_extension("aql");

        let mut p_count = PathBuf::from(base);
        p_count.set_extension("count");

        let f_aql = File::open(p_aql.clone());
        let f_count = File::open(p_count);

        if let (Ok(mut f_aql), Ok(mut f_count)) = (f_aql, f_count) {
            let mut aql = String::new();
            let mut count = String::new();

            if let (Ok(_), Ok(_)) = (
                f_aql.read_to_string(&mut aql),
                f_count.read_to_string(&mut count),
            ) {
                // try to parse the count value
                if let Ok(count) = count.trim().parse::<u64>() {
                    let unknown_name = OsString::from("<unknown>");
                    let name: &OsStr = p_aql.file_stem().unwrap_or(&unknown_name);
                    return Some(SearchDef {
                        aql: String::from(aql.trim()),
                        count,
                        name: String::from(name.to_string_lossy()),
                    });
                }
            }
        }

        None
    }
}

/// Returns an iterator over all query definitions of a folder.
/// - `folder` - The folder on the file system.
/// - `panic_on_invalid` - If true, an invalid query definition will trigger a panic, otherwise it will be ignored.
/// Can be used if this query is called in a test case to fail the test.
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
                        let r = r.unwrap_or_else(|| {
                            panic!(
                                "Search definition for query {} is incomplete",
                                p.to_string_lossy()
                            )
                        });
                        return Some(r);
                    } else {
                        return r;
                    }
                }
            }

            None
        });

        Box::from(it)
    } else {
        Box::new(std::iter::empty())
    }
}
