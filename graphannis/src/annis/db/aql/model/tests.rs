use std::path::PathBuf;

use crate::{annis::db::aql::model::CorpusSizeStatistics, AnnotationGraph};
use assert_matches::assert_matches;

#[test]
fn global_stats_token_count() {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let data_dir = cargo_dir.join("tests/data");

    let mut graph = AnnotationGraph::with_default_graphstorages(false).unwrap();
    graph
        .load_from(&data_dir.join("sample-memory-based"), true)
        .unwrap();

    graph.calculate_all_statistics().unwrap();
    assert_eq!(true, graph.global_statistics.is_some());
    let global_stats = graph.global_statistics.unwrap();
    assert_eq!(true, global_stats.all_token_in_order_component);
    assert_matches!(global_stats.corpus_size, 
        CorpusSizeStatistics::Token { base_token_count, segmentation_count } 
        if base_token_count == 44 && segmentation_count.is_empty());
}
