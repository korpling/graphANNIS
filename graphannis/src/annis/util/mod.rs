pub mod quicksort;

use csv;
use regex_syntax;
use std;
use std::path::Path;

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
        (Some(&qname[..sep_pos]), &qname[sep_pos + 1..])
    } else {
        (None, qname)
    }
}

/// Creates a byte array key from a vector of strings.
///
/// The strings are terminated with `\0`.
pub fn create_str_vec_key(val: &[&str]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::default();
    for v in val {
        // append null-terminated string to result
        for b in v.as_bytes() {
            result.push(*b)
        }
        result.push(0);
    }
    result
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
            corpus: orig.corpus.split(',').map(|s| s.to_string()).collect(),
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
                .filter_map(|row| -> Option<SearchDef> { row.ok() });
            it.collect()
        }
    } else {
        vec![]
    }
}

/// Takes a match identifier (which includes the matched annotation name) and returns the node name.
pub fn node_names_from_match(match_line: &str) -> Vec<String> {
    let mut result = Vec::default();

    for m in match_line.split_whitespace() {
        let elements: Vec<&str> = m.splitn(3, "::").collect();
        if let Some(last_element) = elements.last() {
            result.push(last_element.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_names_from_match() {
        assert_eq!(
            vec!["salt://corpus1/doc1#n1".to_string()],
            node_names_from_match("ns::name::salt://corpus1/doc1#n1")
        );
        assert_eq!(
            vec!["salt://corpus1/doc1#n1".to_string()],
            node_names_from_match("name::salt://corpus1/doc1#n1")
        );
        assert_eq!(
            vec!["salt://corpus1/doc1#n1".to_string()],
            node_names_from_match("salt://corpus1/doc1#n1")
        );

        assert_eq!(
            vec![
                "n1".to_string(),
                "n2".to_string(),
                "n3".to_string(),
                "n4".to_string()
            ],
            node_names_from_match("annis::test::n1 n2 test2::n3 n4")
        );
    }
}
