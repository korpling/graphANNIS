pub mod memory_estimation;
pub mod quicksort;

use csv;
use regex_syntax;
use serde_derive;
use std;
use std::path::Path;

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
#[derive(Debug, Deserialize)]
pub struct SearchDef {
    pub aql: String,
    pub count: u64,
    pub name: String,
    pub corpus: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SearchDefRaw {
    pub aql: String,
    pub count: u64,
    pub name: String,
    pub corpus: String,
}

impl From<SearchDefRaw> for SearchDef {
    fn from(orig: SearchDefRaw) -> Self {
        // Copy all information but split the corpus names to a vector
        SearchDef {
            aql: orig.aql,
            count: orig.count,
            name: orig.name,
            corpus: orig.corpus.split(",").map(|s| s.to_string()).collect(),
        }
    }
}

/// Returns a vector over all query definitions defined in a CSV file.
/// - `file` - The CSV file path.
/// - `panic_on_invalid` - If true, an invalid query definition will trigger a panic, otherwise it will be ignored.
/// Can be used if this query is called in a test case to fail the test.
pub fn get_queries_from_csv(file: &Path, panic_on_invalid: bool) -> Vec<SearchDef> {
    if let Ok(mut reader) = csv::Reader::from_path(file) {
        if panic_on_invalid {
            let it = reader.deserialize().map(|row| -> SearchDef {
                let raw: SearchDefRaw = row.unwrap();
                raw.into()
            });
            it.collect()
        } else {
            let it = reader
                .deserialize()
                .filter_map(|row| -> Option<SearchDef> { row.ok().into() });
            it.collect()
        }
    } else {
        vec![]
    }
}