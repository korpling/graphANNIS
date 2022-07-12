pub mod mapslice;
pub mod quicksort;

use graphannis_core::serializer::KeyVec;

use crate::errors::{GraphAnnisError, Result};

use std::{
    path::Path,
    time::{Duration, Instant},
};

pub fn contains_regex_metacharacters(pattern: &str) -> bool {
    for c in pattern.chars() {
        if regex_syntax::is_meta_character(c) {
            return true;
        }
    }
    false
}

/// Creates a byte array key from a vector of strings.
///
/// The strings are terminated with `\0`.
pub fn create_str_vec_key(val: &[&str]) -> KeyVec {
    let mut result: KeyVec = KeyVec::default();
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
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Clone, Copy)]
pub struct TimeoutCheck {
    start_time: Instant,
    timeout: Option<Duration>,
}

impl TimeoutCheck {
    pub fn new(timeout: Option<Duration>) -> TimeoutCheck {
        TimeoutCheck {
            start_time: Instant::now(),
            timeout,
        }
    }

    /// Check if too much time was used and return an error if this is the case.
    pub fn check(&self) -> Result<()> {
        if let Some(timeout) = self.timeout {
            let elapsed = self.start_time.elapsed();
            if elapsed > timeout {
                debug!(
                    "Timeout reached after {} ms (configured for {} ms)",
                    elapsed.as_millis(),
                    timeout.as_millis()
                );
                return Err(GraphAnnisError::Timeout);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_names_from_match() {
        assert_eq!(
            vec!["corpus1/doc1#n1".to_string()],
            node_names_from_match("ns::name::corpus1/doc1#n1")
        );
        assert_eq!(
            vec!["corpus1/doc1#n1".to_string()],
            node_names_from_match("name::corpus1/doc1#n1")
        );
        assert_eq!(
            vec!["corpus1/doc1#n1".to_string()],
            node_names_from_match("corpus1/doc1#n1")
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
