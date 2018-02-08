extern crate graphannis;

use graphannis::api::corpusstorage::CorpusStorage;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;

use std::collections::HashSet;

thread_local!{
   pub static CORPUS_STORAGE: RefCell<Option<CorpusStorage>> = {
         let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
            path
        } else {
            String::from("data")
        });

        // only execute the test if the directory exists
        let cs = if db_dir.exists() && db_dir.is_dir() {
            CorpusStorage::new(&db_dir).ok()
        } else {
            None
        };
       return RefCell::new(cs)
       };
}

struct SearchDef {
    pub aql: String,
    pub json: String,
    pub count: usize,
}

impl SearchDef {
    fn from_file(base: &Path) -> Option<SearchDef> {
        let mut p_aql = PathBuf::from(base);
        p_aql.set_extension("aql");

        let mut p_json = PathBuf::from(base);
        p_json.set_extension("json");

        let mut p_count = PathBuf::from(base);
        p_count.set_extension("count");

        let f_aql = File::open(p_aql);
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
                if let Ok(count) = count.trim().parse::<usize>() {
                    return Some(SearchDef {
                        aql: String::from(aql.trim()),
                        json: String::from(json.trim()),
                        count,
                    });
                }
            }
        }

        return None;
    }
}


fn get_query_dir() -> PathBuf {
    let query_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_QUERIES") {
        path
    } else {
        String::from("../queries")
    });
    query_dir
}

fn get_queries_from_folder(folder: &Path, panic_on_invalid : bool) -> Box<Iterator<Item = SearchDef>> {
    // get an iterator over all files in the folder
    if let Ok(it_folder) = folder.read_dir() {
        // filter by file type and read both the ".aql", ".json" and ".count" files
        let it = it_folder.filter_map(move |e| -> Option<SearchDef> {
            if let Ok(e) = e {
                let p = e.path();
                if p.exists() && p.is_file()
                    && p.extension() == Some(&std::ffi::OsString::from("aql"))
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

fn get_corpus_storage() -> Option<CorpusStorage> {
    let db_dir = PathBuf::from(if let Ok(path) = std::env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("../data")
    });

    // only execute the test if the directory exists
    let cs = if db_dir.exists() && db_dir.is_dir() {
        CorpusStorage::new(&db_dir).ok()
    } else {
        None
    };
    
    return cs;
}

fn search_test_base(corpus : &str, query_set : &str, panic_on_invalid : bool) {
    let cs = get_corpus_storage();

    if let Some(cs) = cs {
        let corpora : HashSet<String> = cs.list().into_iter().collect();
        // ignore of corpus does not exist
        if corpora.contains(corpus) {
            let mut d = get_query_dir();
            d.push(query_set);
            for def in get_queries_from_folder(&d, panic_on_invalid) {
                let count = cs.count(corpus, &def.json).unwrap_or(0);
                assert_eq!(
                    def.count, count,
                    "Query '{}' on corpus {} should have had count {} but was {}.",
                    def.aql, corpus, def.count, count
                );
                    
            }
        }
    };
}

#[test]
fn count_gum() {
    search_test_base("GUM", "SearchTestGUM", true);
}

#[test]
fn count_pcc2() {
    search_test_base("pcc2", "SearchTestPcc2", true);
}

#[test]
fn count_parlament() {
    search_test_base("parlament", "SearchTestParlament", true);
}

#[test]
fn count_tiger() {
    search_test_base("tiger2", "SearchTestTiger", true);
}

