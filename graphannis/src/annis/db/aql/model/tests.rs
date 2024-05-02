use std::{fs::File, path::PathBuf};

use crate::{
    annis::db::aql::model::CorpusSize, AnnotationGraph
};
use assert_matches::assert_matches;

#[test]
fn global_stats_token_count() {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_file = File::open(&cargo_dir.join("tests/SegmentationWithGaps.graphml")).unwrap();
    let (graph, _config_str) : (AnnotationGraph, _) = graphannis_core::graph::serialization::graphml::import(
       input_file,
        false,
        |_status| {
        },
    ).unwrap();

    assert_eq!(true, graph.global_statistics.is_some());
    let global_stats = graph.global_statistics.unwrap();
    assert_eq!(true, global_stats.all_token_in_order_component);
    assert_matches!(global_stats.corpus_size, 
        CorpusSize::Token { base_token_count, segmentation_count } 
        if base_token_count == 16 
            && segmentation_count.len() == 2
            && *segmentation_count.get("diplomatic").unwrap() == 11 
            && *segmentation_count.get("norm").unwrap() == 13);
}
