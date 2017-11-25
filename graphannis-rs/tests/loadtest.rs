extern crate graphannis;

use std::env;
use std::path::PathBuf;
use graphannis::graphdb::GraphDB;
use graphannis::relannis;
use graphannis::{Annotation, Component, ComponentType, Edge};

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
        let annos : Vec<Annotation> = db.node_annos.get_all(&0);

        assert_eq!(7, annos.len());
        
        assert_eq!("annis", db.strings.str(annos[0].key.ns).unwrap());
        assert_eq!("node_name", db.strings.str(annos[0].key.name).unwrap());
        assert_eq!("pcc2/4282#tok_13", db.strings.str(annos[0].val).unwrap());

        assert_eq!("annis", db.strings.str(annos[1].key.ns).unwrap());
        assert_eq!("tok", db.strings.str(annos[1].key.name).unwrap());
        assert_eq!("so", db.strings.str(annos[1].val).unwrap());

        assert_eq!("annis", db.strings.str(annos[2].key.ns).unwrap());
        assert_eq!("node_type", db.strings.str(annos[2].key.name).unwrap());
        assert_eq!("node", db.strings.str(annos[2].val).unwrap());

        assert_eq!("annis", db.strings.str(annos[3].key.ns).unwrap());
        assert_eq!("layer", db.strings.str(annos[3].key.name).unwrap());
        assert_eq!("token_merged", db.strings.str(annos[3].val).unwrap());

        assert_eq!("tiger", db.strings.str(annos[4].key.ns).unwrap());
        assert_eq!("lemma", db.strings.str(annos[4].key.name).unwrap());
        assert_eq!("so", db.strings.str(annos[4].val).unwrap());

        assert_eq!("tiger", db.strings.str(annos[5].key.ns).unwrap());
        assert_eq!("morph", db.strings.str(annos[5].key.name).unwrap());
        assert_eq!("--", db.strings.str(annos[5].val).unwrap());

        assert_eq!("tiger", db.strings.str(annos[6].key.ns).unwrap());
        assert_eq!("pos", db.strings.str(annos[6].key.name).unwrap());
        assert_eq!("ADV", db.strings.str(annos[6].val).unwrap());
    }

    #[test]
    fn edges() {
        if let Some(db) = load_corpus("pcc2") {
            // get some edges
            let gs_cov = db.get_graphstorage(Component{ctype: ComponentType::Coverage, layer: String::from("annis"), name:String::from("")}).unwrap();
            let out = gs_cov.get_outgoing_edges(&0);
            // TODO: test edges
        }
    }
}