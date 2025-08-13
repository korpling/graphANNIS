use graphannis::CorpusStorage;
use graphannis::corpusstorage::{QueryLanguage, ResultOrder, SearchQuery};
use graphannis::util;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let search_query = SearchQuery {
        corpus_names: &["tutorial"],
        query: "tok . tok",
        query_language: QueryLanguage::AQL,
        timeout: None,
    };

    let matches = cs
        .find(search_query, 0, Some(100), ResultOrder::Normal)
        .unwrap();
    for m in matches {
        println!("{}", m);
        // convert the match string to a list of node IDs
        let node_names = util::node_names_from_match(&m);
        let g = cs.subgraph("tutorial", node_names, 2, 2, None).unwrap();
        // find all nodes of type "node" (regular annotation nodes)
        let node_search =
            g.get_node_annos()
                .exact_anno_search(Some("annis"), "node_type", Some("node").into());
        println!("Number of nodes in subgraph: {}", node_search.count());
    }
}
