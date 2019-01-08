use lalrpop;
use csv;
use std::env;
use std::path::PathBuf;
use regex::Regex;
use std::ops::Deref;

/// Take the CSV file with the queries and add a test case for each query whose
/// corpora exist
fn create_search_tests() -> Option<()> {
    
    println!("rerun-if-env-changed=ANNIS4_TEST_QUERIES");
    println!("rerun-if-env-changed=ANNIS4_TEST_DATA");
    
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("searchtest.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    let query_file = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("tests/searchtest_queries.csv")
    });
    println!("cargo:rerun-if-changed={}", query_file.to_string_lossy());


    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("data")
    });
    println!("cargo:rerun-if-changed={}", db_dir.to_string_lossy());

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
            let corpus_dir = db_dir.join(corpus.deref());
            if corpus_dir.is_dir() {
                // output test case if corpus exists locally
                write!(f,
"
#[ignore]
#[test]
#[allow(non_snake_case)]
fn search_{corpus_escaped}_{name_escaped}() {{
    let aql = r###\"{aql}\"###;
    if let Some(cs_mutex) = CORPUS_STORAGE.as_ref() {{
        let search_count = {{ 
            let cs = cs_mutex.lock().unwrap();
            cs.count(\"{corpus}\", aql, QueryLanguage::AQL).unwrap_or(0)
        }};
        assert_eq!(
            {count}, search_count,
            \"Query '{{}}' ({name}) on corpus {corpus} should have had count {count} but was {{}}.\", 
            aql, search_count
        );
    }}
}}

", 
                name=name, name_escaped=name_escaped, corpus=corpus, corpus_escaped=corpus_escaped, aql=aql, count=count).ok()?;
            }
        }
    }

    Some(())
}

fn create_parser() {
    lalrpop::Configuration::new()
        .set_in_dir("src/annis/db/aql/")
        .set_out_dir(env::var("OUT_DIR").unwrap())
        .process()
        .unwrap();
}

fn main() {
    create_parser();
    create_search_tests().unwrap();
}
