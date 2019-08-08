use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use graphannis::graph::AnnotationStorage;
use graphannis::util;
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let matches = cs
        .find(
            "tutorial",
            "tok . tok",
            QueryLanguage::AQL,
            0,
            100,
            ResultOrder::Normal,
        )
        .unwrap();
    for m in matches {
        println!("{}", m);
        // convert the match string to a list of node IDs
        let node_names = util::node_names_from_match(&m);
        let g = cs.subgraph("tutorial", node_names, 2, 2).unwrap();
        // find all nodes of type "node" (regular annotation nodes)
        let node_search = g.exact_anno_search(
            Some("annis".to_string()),
            "node_type".to_string(),
            Some("node".to_string()).into(),
        );
        println!("Number of nodes in subgraph: {}", node_search.count());
    }
}
