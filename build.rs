use lalrpop;
use csv;
use std::env;
use std::path::PathBuf;

/// Take the CSV file with the queries and add a test case for each query whose
/// corpora exist
fn create_search_tests() -> Option<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("searchtest.rs");
    let mut f = std::fs::File::create(&destination).unwrap();

    let query_file = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("queries/tests.csv")
    });
    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("data")
    });

    if query_file.is_file() {
        let mut reader = csv::Reader::from_path(query_file).ok()?;
        for row in reader.records() {
            use std::io::Write;
            let row = row.ok()?;
            let name = row.get(0)?;
            let aql = row.get(1)?;
            let corpus = row.get(2)?;
            let count = row.get(3)?.trim().parse::<i64>().ok()?;
            let corpus_dir = db_dir.join(corpus);
            if corpus_dir.is_dir() {
                // output test case if corpus exists locally
                write!(f,
"
#[ignore]
#[test]
#[allow(non_snake_case)]
fn search_{corpus}_{name}_serial() {{
    let aql = r#\"\"\"{aql}\"\"\"#;
    CORPUS_STORAGE.with(|cs| {{
        if let Some(ref cs) = *cs.borrow() {{
            let search_count = cs.count(\"{corpus}\", aql, QueryLanguage::AQL).unwrap_or(0);
            assert_eq!(
                {count}, search_count,
                \"Query '{{}}' ({name}) on corpus {corpus} should have had count {count} but was {{}}.\", 
                aql, search_count
            );
        }}
    }});
}}

", 
                name=name, corpus=corpus, aql=aql, count=count).ok()?;
                write!(f,
"
#[ignore]
#[test]
#[allow(non_snake_case)]
fn search_{corpus}_{name}_parallel() {{
    let aql = r#\"\"\"{aql}\"\"\"#;
    CORPUS_STORAGE_PARALLEL.with(|cs| {{
        if let Some(ref cs) = *cs.borrow() {{
            let search_count = cs.count(\"{corpus}\", aql, QueryLanguage::AQL).unwrap_or(0);
            assert_eq!(
                {count}, search_count,
                \"Query '{{}}' ({name}) on corpus {corpus} should have had count {count} but was {{}}.\", 
                aql, search_count
            );
        }}
    }});
}}

", 
                name=name, corpus=corpus, aql=aql, count=count).ok()?;
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
