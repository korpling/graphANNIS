extern crate graphannis;

use std::env;
use std::path::PathBuf;
use graphannis::graphdb::GraphDB;
use graphannis::relannis;

fn load_corpus(name : &str) -> Option<GraphDB> {
    let mut data_dir = PathBuf::from(if let Ok(path) = env::var("ANNIS4_TEST_DATA") {
        path
    } else {
        String::from("../data")
    });
    data_dir.push("../relannis/");
    data_dir.push(name);
    
    // only execute the test if the directory exists
    if data_dir.exists() && data_dir.is_dir() {
        return Some(relannis::load(data_dir.to_str().unwrap()).unwrap());
    } else {
        return None;
    }
}

#[test]
fn node_annos() {
    if let Some(db) = load_corpus("pcc2") {

    }
}