use file_diff::diff;

use regex::Regex;
use std::path::PathBuf;

/// Take the CSV file with the queries and add a test case for each query whose
/// corpora exist
fn create_search_tests() -> Option<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("searchtest.rs");
    let destintation_tmp = std::path::Path::new(&out_dir).join("searchtest.rs.tmp");
    let mut f = std::fs::File::create(&destintation_tmp).unwrap();

    let query_file = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("tests/searchtest_queries.csv")
    });
    println!("cargo:rerun-if-changed={}", query_file.to_string_lossy());
    let invalid_chars = Regex::new(r"[^A-Za-z0-9_]").unwrap();

    if query_file.is_file() {
        let mut reader = csv::Reader::from_path(query_file).ok()?;
        for row in reader.records() {
            use std::io::Write;
            let row = row.ok()?;
            let name = row.get(0)?;
            let name_escaped = invalid_chars.replace_all(name, "_");
            let aql = row.get(1)?;
            let corpus = row.get(2)?;
            let corpus_escaped = invalid_chars.replace_all(corpus, "_");
            let count = row.get(3)?.trim().parse::<i64>().ok()?;
            // output test case
            write!(
                f,
                "
#[ignore]
#[test]
#[allow(non_snake_case)]
fn search_{corpus_escaped}_{name_escaped}() {{
let aql = r###\"{aql}\"###;
if let Some(cs) = CORPUS_STORAGE.as_ref() {{
    let search_count = {{ 
        let search_query = graphannis::corpusstorage::SearchQuery {{
            query: aql,
            corpus_names: &[\"{corpus}\"],
            query_language: QueryLanguage::AQL,
            timeout: None,
        }};
        cs.count(search_query).unwrap_or(0)
    }};
    assert_eq!(
        {count}, search_count,
        \"Query '{{}}' ({name}) on corpus {corpus} should have had count {count} but was {{}}.\", 
        aql, search_count
    );
}}
}}

",
                name = name,
                name_escaped = name_escaped,
                corpus = corpus,
                corpus_escaped = corpus_escaped,
                aql = aql,
                count = count
            )
            .ok()?;
        }
    }

    // if the new temporary file is different to the existing one, replace them
    if !diff(
        &destination.to_string_lossy(),
        &destintation_tmp.to_string_lossy(),
    ) {
        std::fs::copy(destintation_tmp, destination).ok()?;
    }

    Some(())
}

fn create_parser() {
    println!("cargo:rerun-if-changed=src/annis/db/aql/parser.lalrpop");
    lalrpop::process_root().unwrap();
}

fn main() {
    create_parser();
    create_search_tests().unwrap();
}
