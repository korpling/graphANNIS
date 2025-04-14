use tempfile::TempDir;

use crate::annis::types::TimelineStrategy;

use super::*;

fn create_temporary_corpus_dir_file(file_content: &str, file_path: &str) -> TempDir {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join(file_path);
    // The file path could contain directories that need to be created
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    };
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "{}", file_content).unwrap();
    f.flush().unwrap();
    parent
}

#[test]
fn test_escape_field() {
    assert_eq!(escape_field("ab\\$c"), "ab$c");
    assert_eq!(escape_field("ab\\\\cd\\\\"), "ab\\cd\\",);
    assert_eq!(escape_field("ab\\'cd\\te"), "ab'cd\te");
    assert_eq!(escape_field("a\\n"), "a\n");
}

#[test]
fn relannis33_missing_segmentation_span() {
    // Prepare all necessary information to parse the node file
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_path = cargo_dir.join("tests").join("MissingSegmentationCorpus");
    let mut u = GraphUpdate::default();
    let mut texts = DiskMap::default();
    texts
        .insert(
            TextKey {
                id: 0,
                corpus_ref: Some(0),
            },
            Text {
                name: "text".into(),
                val: "".into(),
            },
        )
        .unwrap();
    let mut corpus_by_id = BTreeMap::default();
    let document = CorpusTableEntry {
        pre: 1,
        post: 2,
        name: "document".into(),
        normalized_name: "document".into(),
    };
    let corpus = CorpusTableEntry {
        pre: 0,
        post: 3,
        name: "corpus".into(),
        normalized_name: "corpus".into(),
    };
    corpus_by_id.insert(0, document);
    corpus_by_id.insert(1, corpus);
    let mut corpus_by_preorder = BTreeMap::default();
    corpus_by_preorder.insert(0, 1);
    corpus_by_preorder.insert(1, 0);

    let corpus_table = ParsedCorpusTable {
        toplevel_corpus_name: "MissingSegmentationCorpus".into(),
        corpus_by_preorder,
        corpus_by_id,
    };

    // Load the problematic node file, which should not result in an error
    let result = load_node_tab(
        &input_path,
        &mut u,
        &mut texts,
        &corpus_table,
        true,
        &|_| {},
    )
    .unwrap();

    // Check that the node was added to the missing segmentation span map
    assert_eq!(true, result.missing_seg_span.contains_key(&680).unwrap());
}

/// Regression Test for https://github.com/korpling/graphANNIS/issues/222
#[test]
fn trim_resolver_mappings() {
    let parent = create_temporary_corpus_dir_file(
        "example	NULL	layer	node	htmldoc	edition	hidden	0	hide_tok:true;annos:abc, def   ; config: edition",
        "resolver_vis_map.annis",
    );

    // Parse the resolver entry
    let mut config = CorpusConfiguration::default();
    load_resolver_vis_map(parent.path(), &mut config, true, &|_| {}).unwrap();
    // 6 default rules and an additional rule from the file
    assert_eq!(7, config.visualizers.len());

    assert_eq!("edition", config.visualizers[1].display_name.as_str(),);
    assert_eq!(3, config.visualizers[1].mappings.len());
    assert_eq!(
        Some(&"true".to_string()),
        config.visualizers[1].mappings.get("hide_tok")
    );
    assert_eq!(
        Some(&"abc, def".to_string()),
        config.visualizers[1].mappings.get("annos")
    );
    assert_eq!(
        Some(&"edition".to_string()),
        config.visualizers[1].mappings.get("config")
    );
}

#[test]
fn missing_visibility_column_in_resolver() {
    let parent = create_temporary_corpus_dir_file(
        r#"somecorpus	NULL	syntax	node	tree	syntax (tree)	1	test: true; anothertest:false
somecorpus	NULL	NULL	NULL	discourse	document (text)	2	NULL"#,
        "resolver_vis_map.tab",
    );
    let mut config = CorpusConfiguration::default();
    load_resolver_vis_map(parent.path(), &mut config, false, &|_| {}).unwrap();

    assert_eq!(8, config.visualizers.len());

    let syntax_vis = &config.visualizers[1];
    assert_eq!("syntax (tree)", syntax_vis.display_name);
    assert_eq!(VisualizerVisibility::Hidden, syntax_vis.visibility);
    assert_eq!(2, syntax_vis.mappings.len());
    assert_eq!("true", syntax_vis.mappings.get("test").unwrap());
    assert_eq!("false", syntax_vis.mappings.get("anothertest").unwrap());

    let doc_vis = &config.visualizers[2];
    assert_eq!("document (text)", doc_vis.display_name);
    assert_eq!(VisualizerVisibility::Hidden, doc_vis.visibility);
    assert_eq!(0, doc_vis.mappings.len());
}

#[test]
fn parse_virtual_tokenization_mapping() {
    let parent = create_temporary_corpus_dir_file(
        "virtual_tokenization_mapping=anno1=norm,anno2=norm,anotherspan=dipl,testspan=clean",
        "ExtData/corpus.properties",
    );

    // Parse the corpus configuration
    let mut config = CorpusConfiguration::default();
    load_corpus_properties(parent.path(), &mut config, &|_| {}).unwrap();

    match config.view.timeline_strategy {
        TimelineStrategy::Explicit => {
            panic!("virtual_tokenization_strategy was None, should have been Mapping")
        }
        TimelineStrategy::ImplicitFromNamespace => {
            panic!("virtual_tokenization_strategy was FromNamespace, should have been Mapping")
        }
        TimelineStrategy::ImplicitFromMapping { mappings } => {
            // Check that all entries exist
            assert_eq!(4, mappings.len());
            assert_eq!(Some(&"norm".into()), mappings.get("anno1"));
            assert_eq!(Some(&"norm".into()), mappings.get("anno2"));
            assert_eq!(Some(&"dipl".into()), mappings.get("anotherspan"));
            assert_eq!(Some(&"clean".into()), mappings.get("testspan"));
        }
    }
}
#[test]
fn parse_virtual_tokenization_from_namespace() {
    // Basic case: not set
    let parent = create_temporary_corpus_dir_file("", "ExtData/corpus_config.properties");
    let mut config = CorpusConfiguration::default();
    load_corpus_properties(parent.path(), &mut config, &|_| {}).unwrap();
    assert_eq!(TimelineStrategy::Explicit, config.view.timeline_strategy);

    // Set to "false"
    let parent = create_temporary_corpus_dir_file(
        "virtual_tokenization_from_namespace=false",
        "ExtData/corpus.properties",
    );
    let mut config = CorpusConfiguration::default();
    load_corpus_properties(parent.path(), &mut config, &|_| {}).unwrap();
    assert_eq!(TimelineStrategy::Explicit, config.view.timeline_strategy);

    // Set to invalid value
    let parent = create_temporary_corpus_dir_file(
        "virtual_tokenization_from_namespace=sdsg",
        "ExtData/corpus.properties",
    );
    let mut config = CorpusConfiguration::default();
    load_corpus_properties(parent.path(), &mut config, &|_| {}).unwrap();
    assert_eq!(TimelineStrategy::Explicit, config.view.timeline_strategy);

    // Set to "true"
    let parent = create_temporary_corpus_dir_file(
        "virtual_tokenization_from_namespace=true",
        "ExtData/corpus.properties",
    );
    let mut config = CorpusConfiguration::default();
    load_corpus_properties(parent.path(), &mut config, &|_| {}).unwrap();
    assert_eq!(
        TimelineStrategy::ImplicitFromNamespace,
        config.view.timeline_strategy
    );
}
