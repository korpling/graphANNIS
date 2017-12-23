extern crate graphannis;

use std::env;
use std::path::PathBuf;
use graphannis::graphdb::GraphDB;
use graphannis::relannis;
use graphannis::{Annotation, Component, ComponentType, Edge};
use graphannis::plan::ExecutionPlan;
use graphannis::nodesearch::NodeSearch;
use graphannis::join::nestedloop::NestedLoop;
use graphannis::operator::precedence::{Precedence, PrecedenceSpec};

fn load_corpus(name: &str) -> Option<GraphDB> {
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
        let annos: Vec<Annotation> = db.node_annos.get_all(&0);

        assert_eq!(7, annos.len());

        assert_eq!("annis", db.strings.str(annos[0].key.ns).unwrap());
        assert_eq!("node_name", db.strings.str(annos[0].key.name).unwrap());
        assert_eq!("pcc2/4282#tok_13", db.strings.str(annos[0].val).unwrap());

        assert_eq!("annis", db.strings.str(annos[1].key.ns).unwrap());
        assert_eq!("tok", db.strings
        .str(annos[1].key.name).unwrap());
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
}

#[test]
fn edges() {
    if let Some(mut db) = load_corpus("pcc2") {
        // get some edges
        let edge = Edge {
            source: 371,
            target: 126,
        };

        let edge_components = db.get_direct_connected(&edge).unwrap();
        assert_eq!(4, edge_components.len());

        let edge_annos = db.get_graphstorage(&edge_components[1])
            .unwrap()
            .get_edge_annos(&edge);
        assert_eq!(1, edge_annos.len());
        assert_eq!("tiger", db.strings.str(edge_annos[0].key.ns).unwrap());
        assert_eq!("func", db.strings.str(edge_annos[0].key.name).unwrap());
        assert_eq!("OA", db.strings.str(edge_annos[0].val).unwrap());

        let edge_annos = db.get_graphstorage(&edge_components[2])
            .unwrap()
            .get_edge_annos(&edge);
        assert_eq!(1, edge_annos.len());
        assert_eq!("tiger", db.strings.str(edge_annos[0].key.ns).unwrap());
        assert_eq!("func", db.strings.str(edge_annos[0].key.name).unwrap());
        assert_eq!("OA", db.strings.str(edge_annos[0].val).unwrap());

        let edge_annos = db.get_graphstorage(&edge_components[0])
            .unwrap()
            .get_edge_annos(&edge);
        assert_eq!(0, edge_annos.len());
        let edge_annos = db.get_graphstorage(&edge_components[3])
            .unwrap()
            .get_edge_annos(&edge);
        assert_eq!(0, edge_annos.len());
    }
}

#[test]
fn manual_execution_plan() {
    if let Some(mut db) = load_corpus("pcc2") {
        let n1 = NodeSearch::new(
            db.node_annos.exact_anno_search(
                Some(db.strings.add("annis")),
                db.strings.add("node_type"),
                Some(db.strings.add("node")),
            ),
            None,
        );

        let n2 = NodeSearch::new(
            db.node_annos.exact_anno_search(
                Some(db.strings.add("annis")),
                db.strings.add("node_type"),
                Some(db.strings.add("node")),
            ),
            None,
        );

        let op = Precedence::new(
            &db,
            PrecedenceSpec {
                segmentation: None,
                min_dist: 1,
                max_dist: 1,
            },
        );

        let op = Box::new(op.unwrap());

        let n1 = Box::new(n1);
        let n2 = Box::new(n2);

        let join = NestedLoop::new(n1, n2, 0, 0, op);

        assert_eq!(2678, join.count());
    }
}

