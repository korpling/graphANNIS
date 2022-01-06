use super::aql::model::{AnnotationComponentType, TOK_WHITESPACE_AFTER, TOK_WHITESPACE_BEFORE};
use crate::annis::db::corpusstorage::SALT_URI_ENCODE_SET;
use crate::annis::errors::*;
use crate::annis::util::create_str_vec_key;
use crate::update::{GraphUpdate, UpdateEvent};
use crate::{
    annis::{
        db::aql::model::TOK,
        types::{
            CorpusConfiguration, ExampleQuery, VisualizerRule, VisualizerRuleElement,
            VisualizerVisibility,
        },
    },
    corpusstorage::QueryLanguage,
    AnnotationGraph,
};
use graphannis_core::serializer::KeyVec;
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_NS},
    serializer::KeySerializer,
    types::{AnnoKey, Component, Edge, NodeID},
    util::disk_collections::DiskMap,
};
use itertools::Itertools;
use percent_encoding::utf8_percent_encode;
use smartstring::alias::String;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Bound;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref INVALID_STRING: SmartString<LazyCompact> =
        SmartString::<LazyCompact>::from(std::char::MAX.to_string());
    static ref DEFAULT_VISUALIZER_RULES: Vec<(i64, bool, VisualizerRule)> = vec![
        (
            -1,
            true,
            VisualizerRule {
                display_name: "kwic".to_string(),
                vis_type: "kwic".to_string(),
                element: None,
                layer: None,
                visibility: VisualizerVisibility::Permanent,
                mappings: BTreeMap::default(),
            }
        ),
        (
            101,
            true,
            VisualizerRule {
                display_name: "tree".to_string(),
                vis_type: "tree".to_string(),
                element: Some(VisualizerRuleElement::Node),
                layer: Some("tiger".to_string()),
                visibility: VisualizerVisibility::Hidden,
                mappings: BTreeMap::default(),
            }
        ),
        (
            102,
            true,
            VisualizerRule {
                display_name: "exmaralda".to_string(),
                vis_type: "grid".to_string(),
                element: Some(VisualizerRuleElement::Node),
                layer: Some("exmaralda".to_string()),
                visibility: VisualizerVisibility::Hidden,
                mappings: BTreeMap::default(),
            }
        ),
        (
            103,
            true,
            VisualizerRule {
                display_name: "mmax".to_string(),
                vis_type: "grid".to_string(),
                element: Some(VisualizerRuleElement::Node),
                layer: Some("mmax".to_string()),
                visibility: VisualizerVisibility::Hidden,
                mappings: BTreeMap::default(),
            }
        ),
        (
            104,
            true,
            VisualizerRule {
                display_name: "coref".to_string(),
                vis_type: "discourse".to_string(),
                element: Some(VisualizerRuleElement::Edge),
                layer: Some("mmax".to_string()),
                visibility: VisualizerVisibility::Hidden,
                mappings: BTreeMap::default(),
            }
        ),
        (
            105,
            true,
            VisualizerRule {
                display_name: "urml".to_string(),
                vis_type: "grid".to_string(),
                element: Some(VisualizerRuleElement::Node),
                layer: Some("urml".to_string()),
                visibility: VisualizerVisibility::Hidden,
                mappings: BTreeMap::default(),
            }
        ),
    ];
}

#[derive(
    Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize, MallocSizeOf,
)]
pub struct TextProperty {
    segmentation: String,
    corpus_id: u32,
    text_id: u32,
    val: u32,
}

impl KeySerializer for TextProperty {
    fn create_key(&self) -> KeyVec {
        let mut result = KeyVec::new();
        result.extend(create_str_vec_key(&[&self.segmentation]));
        result.extend(self.corpus_id.to_be_bytes());
        result.extend(self.text_id.to_be_bytes());
        result.extend(self.val.to_be_bytes());
        result
    }

    fn parse_key(key: &[u8]) -> Self {
        let id_size = std::mem::size_of::<u32>();
        let mut id_offset = key.len() - id_size * 3;
        let key_as_string: std::string::String = std::string::String::from_utf8_lossy(key).into();
        let segmentation_vector: Vec<_> = key_as_string.split_terminator('\0').collect();

        let corpus_id = u32::from_be_bytes(
            key[id_offset..(id_offset + id_size)]
                .try_into()
                .expect("TextProperty deserialization key was too small"),
        );
        id_offset += id_size;

        let text_id = u32::from_be_bytes(
            key[id_offset..(id_offset + id_size)]
                .try_into()
                .expect("TextProperty deserialization key was too small"),
        );
        id_offset += id_size;

        let val = u32::from_be_bytes(
            key[id_offset..(id_offset + id_size)]
                .try_into()
                .expect("TextProperty deserialization key was too small"),
        );

        let segmentation = if segmentation_vector.is_empty() {
            String::from("")
        } else {
            segmentation_vector[0].into()
        };

        TextProperty {
            segmentation,
            corpus_id,
            text_id,
            val,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, MallocSizeOf)]
struct TextKey {
    id: u32,
    corpus_ref: Option<u32>,
}

impl KeySerializer for TextKey {
    fn create_key(&self) -> KeyVec {
        let mut result = KeyVec::new();
        result.extend(self.id.to_be_bytes());
        if let Some(corpus_ref) = self.corpus_ref {
            result.extend(corpus_ref.to_be_bytes());
        }
        result
    }

    fn parse_key(key: &[u8]) -> Self {
        let id_size = std::mem::size_of::<u32>();
        let id = u32::from_be_bytes(
            key[0..id_size]
                .try_into()
                .expect("TextKey deserialization key was too small"),
        );
        let corpus_ref = if key.len() == id_size * 2 {
            Some(u32::from_be_bytes(
                key[id_size..]
                    .try_into()
                    .expect("TextKey deserialization key was too small"),
            ))
        } else {
            None
        };

        TextKey { id, corpus_ref }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, MallocSizeOf)]
struct NodeByTextEntry {
    text_id: u32,
    corpus_ref: u32,
    node_id: NodeID,
}

impl KeySerializer for NodeByTextEntry {
    fn create_key(&self) -> KeyVec {
        let mut result = KeyVec::new();
        result.extend(self.text_id.to_be_bytes());
        result.extend(self.corpus_ref.to_be_bytes());
        result.extend(self.node_id.to_be_bytes());
        result
    }

    fn parse_key(key: &[u8]) -> Self {
        let u32_size = std::mem::size_of::<u32>();
        let text_id = u32::from_be_bytes(
            key[0..u32_size]
                .try_into()
                .expect("NodeByTextEntry deserialization key was too small (text_id)"),
        );
        let mut offset = u32_size;

        let corpus_ref = u32::from_be_bytes(
            key[offset..(offset + u32_size)]
                .try_into()
                .expect("NodeByTextEntry deserialization key was too small (corpus_ref)"),
        );
        offset += u32_size;

        let node_id = NodeID::from_be_bytes(
            key[offset..]
                .try_into()
                .expect("NodeByTextEntry deserialization key was too small (node_id)"),
        );

        NodeByTextEntry {
            text_id,
            corpus_ref,
            node_id,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, MallocSizeOf)]
struct Text {
    name: String,
    val: String,
}

struct CorpusTableEntry {
    pre: u32,
    post: u32,
    name: String,
    normalized_name: String,
}

struct ParsedCorpusTable {
    toplevel_corpus_name: String,
    corpus_by_preorder: BTreeMap<u32, u32>,
    corpus_by_id: BTreeMap<u32, CorpusTableEntry>,
}

struct TextPosTable {
    token_by_left_textpos: DiskMap<TextProperty, NodeID>,
    token_by_right_textpos: DiskMap<TextProperty, NodeID>,
    // maps a token index to an node ID
    token_by_index: DiskMap<TextProperty, NodeID>,
    // maps a token node id to the token index
    token_to_index: DiskMap<NodeID, TextProperty>,
    //// Map as node to it's "left" value.
    //// This is used for alignment and can be the token or character index.
    node_to_left: DiskMap<NodeID, TextProperty>,
    /// Map as node to it's "right" value.
    //// This is used for alignment and can be the token or character index.
    node_to_right: DiskMap<NodeID, TextProperty>,
    /// Map a node to its left character index.
    node_to_left_char: DiskMap<NodeID, TextProperty>,
    /// Map a node to its right character index.
    node_to_right_char: DiskMap<NodeID, TextProperty>,
}

struct LoadRankResult {
    components_by_pre: DiskMap<u32, Component<AnnotationComponentType>>,
    edges_by_pre: DiskMap<u32, Edge>,
    text_coverage_edges: DiskMap<Edge, bool>,
    /// Some rank entries have NULL as parent: we don't add an edge but remember the component name
    /// for re-creating omitted coverage edges with the correct name.
    component_for_parentless_target_node: DiskMap<NodeID, Component<AnnotationComponentType>>,
}

struct LoadNodeAndCorpusResult {
    toplevel_corpus_name: String,
    id_to_node_name: DiskMap<NodeID, String>,
    textpos_table: TextPosTable,
}

struct NodeTabParseResult {
    nodes_by_text: DiskMap<NodeByTextEntry, bool>,
    missing_seg_span: DiskMap<NodeID, String>,
    id_to_node_name: DiskMap<NodeID, String>,
    textpos_table: TextPosTable,
}

struct LoadNodeResult {
    nodes_by_text: DiskMap<NodeByTextEntry, bool>,
    id_to_node_name: DiskMap<NodeID, String>,
    textpos_table: TextPosTable,
}

/// Load a c corpus in the legacy relANNIS format from the specified `path`.
///
/// Returns a tuple consisting of the corpus name and the extracted annotation graph.
pub fn load<F>(
    path: &Path,
    disk_based: bool,
    progress_callback: F,
) -> Result<(String, AnnotationGraph, CorpusConfiguration)>
where
    F: Fn(&str),
{
    // convert to path
    let path = PathBuf::from(path);
    if path.is_dir() && path.exists() {
        // check if this is the ANNIS 3.3 import format
        let annis_version_path = path.join("annis.version");
        let is_annis_33 = if annis_version_path.exists() {
            let mut file = File::open(&annis_version_path)?;
            let mut version_str = std::string::String::new();
            file.read_to_string(&mut version_str)?;

            version_str == "3.3"
        } else {
            false
        };

        let mut db = AnnotationGraph::with_default_graphstorages(disk_based)?;
        let mut config = CorpusConfiguration::default();
        let mut updates = GraphUpdate::new();
        let load_node_and_corpus_result =
            load_node_and_corpus_tables(&path, &mut updates, is_annis_33, &progress_callback)?;
        {
            let text_coverage_edges = load_edge_tables(
                &path,
                &mut updates,
                is_annis_33,
                &load_node_and_corpus_result.id_to_node_name,
                &progress_callback,
            )?;

            calculate_automatic_coverage_edges(
                &mut updates,
                &load_node_and_corpus_result,
                &text_coverage_edges,
                &progress_callback,
            )?;
        }

        load_resolver_vis_map(&path, &mut config, is_annis_33, &progress_callback)?;
        load_example_queries(&path, &mut config, is_annis_33, &progress_callback)?;
        load_corpus_properties(&path, &mut config, &progress_callback)?;

        // TODO: implement handling the "virtual_tokenization_from_namespace" and "virtual_tokenization_mapping" corpus properties

        progress_callback("calculating node statistics (before update)");
        db.get_node_annos_mut().calculate_statistics();

        db.apply_update(&mut updates, &progress_callback)?;

        progress_callback("calculating node statistics (after update)");
        db.get_node_annos_mut().calculate_statistics();

        for c in db.get_all_components(None, None) {
            progress_callback(&format!("calculating statistics for component {}", c));
            db.calculate_component_statistics(&c)?;
            progress_callback(&format!(
                "checking if implementation for component {} can be optimized",
                c
            ));
            db.optimize_gs_impl(&c)?;
        }

        progress_callback(&format!(
            "finished loading relANNIS from {}",
            path.to_string_lossy()
        ));

        return Ok((load_node_and_corpus_result.toplevel_corpus_name, db, config));
    }

    Err(RelAnnisError::DirectoryNotFound(path.to_string_lossy().to_string()).into())
}

fn load_node_and_corpus_tables<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadNodeAndCorpusResult>
where
    F: Fn(&str),
{
    let corpus_table = parse_corpus_tab(path, is_annis_33, &progress_callback)?;
    let mut texts = parse_text_tab(path, is_annis_33, &progress_callback)?;
    let corpus_id_to_annos = load_corpus_annotation(path, is_annis_33, &progress_callback)?;

    let load_nodes_result = load_nodes(
        path,
        updates,
        &mut texts,
        &corpus_table,
        is_annis_33,
        progress_callback,
    )?;

    add_subcorpora(
        updates,
        &corpus_table,
        &load_nodes_result,
        &texts,
        &corpus_id_to_annos,
        is_annis_33,
        path,
    )?;

    add_white_space_token_labels(
        updates,
        &load_nodes_result.textpos_table,
        &mut texts,
        &load_nodes_result.id_to_node_name,
        progress_callback,
    )?;

    Ok(LoadNodeAndCorpusResult {
        toplevel_corpus_name: corpus_table.toplevel_corpus_name,
        id_to_node_name: load_nodes_result.id_to_node_name,
        textpos_table: load_nodes_result.textpos_table,
    })
}

fn load_edge_tables<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    is_annis_33: bool,
    id_to_node_name: &DiskMap<NodeID, String>,
    progress_callback: &F,
) -> Result<LoadRankResult>
where
    F: Fn(&str),
{
    let load_rank_result = {
        let component_by_id = load_component_tab(path, is_annis_33, progress_callback)?;

        load_rank_tab(
            path,
            updates,
            &component_by_id,
            id_to_node_name,
            is_annis_33,
            progress_callback,
        )?
    };

    load_edge_annotation(
        path,
        updates,
        &load_rank_result,
        id_to_node_name,
        is_annis_33,
        progress_callback,
    )?;

    Ok(load_rank_result)
}

fn load_resolver_vis_map<F>(
    path: &Path,
    config: &mut CorpusConfiguration,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    let mut resolver_tab_path = PathBuf::from(path);
    resolver_tab_path.push(if is_annis_33 {
        "resolver_vis_map.annis"
    } else {
        "resolver_vis_map.tab"
    });

    if !resolver_tab_path.is_file() {
        // This is an optional file, don't fail if it does not exist
        return Ok(());
    }

    progress_callback(&format!(
        "loading {}",
        resolver_tab_path.to_str().unwrap_or_default()
    ));

    let mut resolver_tab_csv = postgresql_import_reader(resolver_tab_path.as_path())?;

    let mut rules_by_order: Vec<(i64, bool, VisualizerRule)> = DEFAULT_VISUALIZER_RULES.clone();

    for result in resolver_tab_csv.records() {
        let line = result?;

        let layer = get_field(&line, 2, "namespace", &resolver_tab_path)?.map(|l| l.to_string());
        let element =
            get_field(&line, 3, "element", &resolver_tab_path)?.and_then(|e| match e.as_ref() {
                "node" => Some(VisualizerRuleElement::Node),
                "edge" => Some(VisualizerRuleElement::Edge),
                _ => None,
            });
        let vis_type = get_field_not_null(&line, 4, "vis_type", &resolver_tab_path)?;
        let display_name = get_field_not_null(&line, 5, "display_name", &resolver_tab_path)?;

        let visibility = get_field_not_null(&line, 6, "visibility", &resolver_tab_path)?;

        let order = get_field(&line, 7, "order", &resolver_tab_path)?
            .map(|order| order.parse::<i64>().unwrap_or_default())
            .unwrap_or_default();
        let mappings: BTreeMap<std::string::String, std::string::String> =
            if let Ok(mappings_field) = get_field(&line, 8, "mappings", &resolver_tab_path) {
                mappings_field
                    .unwrap_or_default()
                    .split(';')
                    .filter_map(|key_value| {
                        let splitted: Vec<_> = key_value.splitn(2, ':').collect();
                        if splitted.len() == 2 {
                            Some((splitted[0].into(), splitted[1].into()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                BTreeMap::new()
            };
        if visibility == "removed" {
            // Remove the matching default visualization
            rules_by_order.retain(|(_, is_default, vis)| {
                !is_default
                    || vis.display_name != display_name
                    || vis.element != element
                    || vis.layer != layer
                    || vis.vis_type != vis_type
            });
        } else {
            // Insert the new Rule
            let visibility = match visibility.as_ref() {
                "hidden" => VisualizerVisibility::Hidden,
                "visible" => VisualizerVisibility::Visible,
                "permanent" => VisualizerVisibility::Permanent,
                "preloaded" => VisualizerVisibility::Preloaded,
                _ => VisualizerVisibility::default(),
            };

            let rule = VisualizerRule {
                layer,
                element,
                vis_type: vis_type.to_string(),
                display_name: display_name.to_string(),
                visibility,
                mappings,
            };

            // Remove any of the existing rules that match exactly the same conditions and show the same visualizer with
            // the same parameters.
            rules_by_order.retain(|(_, is_default, vis)| {
                !is_default || vis.layer.is_none() || vis.layer != rule.layer
            });

            // Insert at sorted position by the order
            match rules_by_order.binary_search_by_key(&order, |(o, _, _)| *o) {
                Ok(idx) => rules_by_order.insert(idx + 1, (order, false, rule)),
                Err(idx) => rules_by_order.insert(idx, (order, false, rule)),
            }
        }
    }

    config.visualizers = rules_by_order.into_iter().map(|(_, _, r)| r).collect();

    Ok(())
}

fn load_example_queries<F>(
    path: &Path,
    config: &mut CorpusConfiguration,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    let mut example_queries_path = PathBuf::from(path);
    example_queries_path.push(if is_annis_33 {
        "example_queries.annis"
    } else {
        "example_queries.tab"
    });

    if !example_queries_path.is_file() {
        // This is an optional file, don't fail if it does not exist
        return Ok(());
    }

    progress_callback(&format!(
        "loading {}",
        example_queries_path.to_str().unwrap_or_default()
    ));

    let mut example_queries_csv = postgresql_import_reader(example_queries_path.as_path())?;

    for result in example_queries_csv.records() {
        let line = result?;

        if let (Some(query), Some(description)) = (
            get_field(&line, 0, "query", &example_queries_path)?,
            get_field(&line, 1, "description", &example_queries_path)?,
        ) {
            config.example_queries.push(ExampleQuery {
                query: query.to_string(),
                description: description.to_string(),
                query_language: QueryLanguage::AQL,
            });
        }
    }
    Ok(())
}

fn load_corpus_properties<F>(
    path: &Path,
    config: &mut CorpusConfiguration,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    let corpus_config_path = path.join("ExtData").join("corpus.properties");

    if !corpus_config_path.is_file() {
        // This is an optional file, don't fail if it does not exist
        return Ok(());
    }

    progress_callback(&format!(
        "loading {}",
        corpus_config_path.to_str().unwrap_or_default()
    ));

    // property files are small, we can read them all at once
    let content = std::fs::read_to_string(corpus_config_path)?;
    // read all lines
    for line in content.lines() {
        // split into key and value
        let splitted: Vec<_> = line.splitn(2, '=').collect();
        if splitted.len() == 2 {
            let key = splitted[0];
            let value = splitted[1];

            match key {
                "max-context" => {
                    if let Ok(value) = value.parse::<usize>() {
                        config.context.max = Some(value);
                    }
                }
                "default-context" => {
                    if let Ok(value) = value.parse::<usize>() {
                        config.context.default = value;
                    }
                }
                "results-per-page" => {
                    if let Ok(value) = value.parse::<usize>() {
                        config.view.page_size = value;
                    }
                }
                "default-context-segmentation" => {
                    if !value.is_empty() {
                        config.context.segmentation = Some(value.to_string());
                    }
                }
                "default-base-text-segmentation" => {
                    if !value.is_empty() {
                        config.view.base_text_segmentation = Some(value.to_string());
                    }
                }
                "hidden_annos" => {
                    if !value.is_empty() {
                        // Entry is a comma-separated list
                        config.view.hidden_annos =
                            value.split(',').map(|a| a.trim().to_owned()).collect();
                    }
                }
                _ => {}
            };
        }
    }

    // The context step is dependent on the max-context configuration.
    // Use a second pass to make sure it already has been set.
    for line in content.lines() {
        // split into key and value
        let splitted: Vec<_> = line.splitn(2, '=').collect();
        if splitted.len() == 2 {
            let key = splitted[0];
            let value = splitted[1];

            if let "context-steps" = key {
                if let Ok(value) = value.parse::<usize>() {
                    config.context.sizes = (value..=config.context.max.unwrap_or(value))
                        .step_by(value)
                        .collect();
                }
            }
        }
    }

    Ok(())
}

fn add_external_data_files(
    import_path: &Path,
    parent_node_full_name: &str,
    document: Option<&str>,
    updates: &mut GraphUpdate,
) -> Result<()> {
    // Get a reference to the ExtData folder
    let mut ext_data = import_path.join("ExtData");
    // Toplevel corpus files are located directly in the ExtData folder,
    // files assigned to documents are in a sub-folder with the document name.
    if let Some(document) = document {
        ext_data.push(document);
    }
    if ext_data.is_dir() {
        // List all files in the target folder
        for file in std::fs::read_dir(&ext_data)? {
            let file = file?;
            if file.file_type()?.is_file() {
                // Add a node for the linked file that is part of the (sub-) corpus
                let node_name = format!(
                    "{}/{}",
                    parent_node_full_name,
                    file.file_name().to_string_lossy()
                );
                updates.add_event(UpdateEvent::AddNode {
                    node_type: "file".to_string(),
                    node_name: node_name.clone(),
                })?;
                updates.add_event(UpdateEvent::AddNodeLabel {
                    node_name: node_name.clone(),
                    anno_ns: ANNIS_NS.to_string(),
                    anno_name: "file".to_string(),
                    anno_value: file.path().canonicalize()?.to_string_lossy().to_string(),
                })?;
                updates.add_event(UpdateEvent::AddEdge {
                    source_node: node_name.clone(),
                    target_node: parent_node_full_name.to_owned(),
                    layer: ANNIS_NS.to_owned(),
                    component_type: AnnotationComponentType::PartOf.to_string(),
                    component_name: std::string::String::default(),
                })?;
            }
        }
    }

    Ok(())
}

fn postgresql_import_reader(path: &Path) -> std::result::Result<csv::Reader<File>, csv::Error> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .quote(0) // effectivly disable quoting
        .from_path(path)
}

fn get_field<'a>(
    record: &'a csv::StringRecord,
    i: usize,
    column_name: &str,
    file: &Path,
) -> crate::errors::Result<Option<SmartString<LazyCompact>>> {
    let r = record.get(i).ok_or_else(|| RelAnnisError::MissingColumn {
        pos: i,
        name: column_name.to_string(),
        file: file.to_string_lossy().to_string(),
    })?;

    if r == "NULL" {
        Ok(None)
    } else {
        // replace some known escape sequences
        Ok(Some(escape_field(r)))
    }
}

fn escape_field(val: &str) -> SmartString<LazyCompact> {
    let mut chars = val.chars().peekable();
    let mut unescaped = SmartString::<LazyCompact>::new();

    loop {
        match chars.next() {
            None => break,
            Some(c) => {
                let escaped_char = if c == '\\' {
                    if let Some(escaped_char) = chars.peek() {
                        let escaped_char = *escaped_char;
                        match escaped_char {
                            _ if escaped_char == '\\'
                                || escaped_char == '"'
                                || escaped_char == '\''
                                || escaped_char == '`'
                                || escaped_char == '$' =>
                            {
                                Some(escaped_char)
                            }
                            'n' => Some('\n'),
                            'r' => Some('\r'),
                            't' => Some('\t'),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(escaped_char) = escaped_char {
                    unescaped.push(escaped_char);
                    // skip the escaped character instead of outputting it again
                    chars.next();
                } else {
                    unescaped.push(c);
                };
            }
        }
    }

    unescaped
}

fn get_field_not_null<'a>(
    record: &'a csv::StringRecord,
    i: usize,
    column_name: &str,
    file: &Path,
) -> crate::errors::Result<SmartString<LazyCompact>> {
    let result =
        get_field(record, i, column_name, file)?.ok_or_else(|| RelAnnisError::UnexpectedNull {
            pos: i,
            name: column_name.to_string(),
            file: file.to_string_lossy().to_string(),
            line: record.position().map(|p| p.line()),
        })?;
    Ok(result)
}

fn parse_corpus_tab<F>(
    path: &Path,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<ParsedCorpusTable>
where
    F: Fn(&str),
{
    let mut corpus_tab_path = PathBuf::from(path);
    corpus_tab_path.push(if is_annis_33 {
        "corpus.annis"
    } else {
        "corpus.tab"
    });

    progress_callback(&format!(
        "loading {}",
        corpus_tab_path.to_str().unwrap_or_default()
    ));

    let mut corpus_by_preorder = BTreeMap::new();
    let mut corpus_by_id = BTreeMap::new();

    let mut corpus_tab_csv = postgresql_import_reader(corpus_tab_path.as_path())?;

    let mut document_names: HashMap<String, usize> = HashMap::new();

    for result in corpus_tab_csv.records() {
        let line = result?;

        let id = get_field_not_null(&line, 0, "id", &corpus_tab_path)?.parse::<u32>()?;
        let name = get_field_not_null(&line, 1, "name", &corpus_tab_path)?;

        let corpus_type = get_field_not_null(&line, 2, "type", &corpus_tab_path)?;
        if corpus_type == "DOCUMENT" {
            // There was always an implicit constraint that document names must be unique in the whole corpus,
            // even when the document belongs to different sub-corpora.
            // Some corpora violate this constraint and we change the document name in order to avoid duplicate node names later on
            let existing_count = document_names
                .entry(name.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            if *existing_count > 1 {
                let old_name = name.clone();
                let name = format!("{}_duplicated_document_name_{}", name, existing_count);
                warn!(
                    "duplicated document name \"{}\" detected: will be renamed to \"{}\"",
                    old_name, name
                );
            }
        }

        let pre_order = get_field_not_null(&line, 4, "pre", &corpus_tab_path)?.parse::<u32>()?;
        let post_order = get_field_not_null(&line, 5, "post", &corpus_tab_path)?.parse::<u32>()?;

        let normalized_name = utf8_percent_encode(&name, SALT_URI_ENCODE_SET);

        corpus_by_id.insert(
            id,
            CorpusTableEntry {
                pre: pre_order,
                post: post_order,
                normalized_name: String::from(normalized_name.to_string()),
                name,
            },
        );

        corpus_by_preorder.insert(pre_order, id);
    }

    let toplevel_corpus_id = corpus_by_preorder
        .iter()
        .next()
        .ok_or(RelAnnisError::ToplevelCorpusNotFound)?
        .1;
    Ok(ParsedCorpusTable {
        toplevel_corpus_name: corpus_by_id
            .get(toplevel_corpus_id)
            .ok_or_else(|| RelAnnisError::CorpusNotFound(*toplevel_corpus_id))?
            .name
            .clone(),
        corpus_by_preorder,
        corpus_by_id,
    })
}

fn parse_text_tab<F>(
    path: &Path,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<DiskMap<TextKey, Text>>
where
    F: Fn(&str),
{
    let mut text_tab_path = PathBuf::from(path);
    text_tab_path.push(if is_annis_33 {
        "text.annis"
    } else {
        "text.tab"
    });

    progress_callback(&format!(
        "loading {}",
        text_tab_path.to_str().unwrap_or_default()
    ));

    let mut texts: DiskMap<TextKey, Text> = DiskMap::default();

    let mut text_tab_csv = postgresql_import_reader(text_tab_path.as_path())?;

    for result in text_tab_csv.records() {
        let line = result?;

        let id = get_field_not_null(&line, if is_annis_33 { 1 } else { 0 }, "id", &text_tab_path)?
            .parse::<u32>()?;
        let name = get_field_not_null(
            &line,
            if is_annis_33 { 2 } else { 1 },
            "name",
            &text_tab_path,
        )?;

        let value = get_field_not_null(
            &line,
            if is_annis_33 { 3 } else { 2 },
            "text",
            &text_tab_path,
        )?;

        let corpus_ref = if is_annis_33 {
            Some(get_field_not_null(&line, 0, "corpus_ref", &text_tab_path)?.parse::<u32>()?)
        } else {
            None
        };
        let key = TextKey { id, corpus_ref };
        texts.insert(key.clone(), Text { name, val: value })?;
    }

    Ok(texts)
}

fn calculate_automatic_token_order<F>(
    updates: &mut GraphUpdate,
    token_by_index: &DiskMap<TextProperty, NodeID>,
    id_to_node_name: &DiskMap<NodeID, String>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    // iterate over all token by their order, find the nodes with the same
    // text coverage (either left or right) and add explicit Ordering edge

    let msg = "calculating the automatically generated Ordering edges";
    progress_callback(msg);

    let mut last_textprop: Option<TextProperty> = None;
    let mut last_token: Option<NodeID> = None;

    for (current_textprop, current_token) in token_by_index.try_iter()? {
        // if the last token/text value is valid and we are still in the same text
        if let (Some(last_token), Some(last_textprop)) = (last_token, last_textprop) {
            if last_textprop.corpus_id == current_textprop.corpus_id
                && last_textprop.text_id == current_textprop.text_id
                && last_textprop.segmentation == current_textprop.segmentation
            {
                // we are still in the same text, add ordering between token
                let ordering_layer = if current_textprop.segmentation.is_empty() {
                    ANNIS_NS.to_owned()
                } else {
                    DEFAULT_NS.to_owned()
                };
                updates.add_event(UpdateEvent::AddEdge {
                    source_node: id_to_node_name
                        .try_get(&last_token)?
                        .ok_or(RelAnnisError::NodeNotFound(last_token))?
                        .into(),
                    target_node: id_to_node_name
                        .try_get(&current_token)?
                        .ok_or(RelAnnisError::NodeNotFound(current_token))?
                        .into(),
                    layer: ordering_layer,
                    component_type: AnnotationComponentType::Ordering.to_string(),
                    component_name: current_textprop.segmentation.clone().into(),
                })?;
            }
        } // end if same text

        // update the iterator and other variables
        last_textprop = Some(current_textprop.clone());
        last_token = Some(current_token);
    } // end for each token

    Ok(())
}

fn add_automatic_cov_edge_for_node(
    updates: &mut GraphUpdate,
    n: NodeID,
    load_node_and_corpus_result: &LoadNodeAndCorpusResult,
    load_rank_result: &LoadRankResult,
) -> Result<()> {
    let left_pos = load_node_and_corpus_result
        .textpos_table
        .node_to_left
        .try_get(&n)?
        .ok_or(RelAnnisError::NoLeftPositionForNode(n))?;

    let right_pos = load_node_and_corpus_result
        .textpos_table
        .node_to_right
        .try_get(&n)?
        .ok_or(RelAnnisError::NoRightPositionForNode(n))?;

    // find left/right aligned basic token
    let left_aligned_tok = load_node_and_corpus_result
        .textpos_table
        .token_by_left_textpos
        .try_get(&left_pos)?;
    let right_aligned_tok = load_node_and_corpus_result
        .textpos_table
        .token_by_right_textpos
        .try_get(&right_pos)?;

    // If only one of the aligned token is missing, use it for both sides, this is consistent with
    // the relANNIS import of ANNIS3
    let left_aligned_tok = if let Some(left_aligned_tok) = left_aligned_tok {
        left_aligned_tok
    } else {
        right_aligned_tok.ok_or(RelAnnisError::AlignedNotFound(n))?
    };
    let right_aligned_tok = if let Some(right_aligned_tok) = right_aligned_tok {
        right_aligned_tok
    } else {
        left_aligned_tok
    };

    let left_tok_pos = load_node_and_corpus_result
        .textpos_table
        .token_to_index
        .try_get(&left_aligned_tok)?
        .ok_or(RelAnnisError::LeftAlignedNotFound(n))?;
    let right_tok_pos = load_node_and_corpus_result
        .textpos_table
        .token_to_index
        .try_get(&right_aligned_tok)?
        .ok_or(RelAnnisError::RightAlignedNotFound(n))?;

    // Create a template TextProperty to which we only change the
    // position (val) while iterating over all token positions.
    let mut tok_idx = TextProperty {
        segmentation: String::default(),
        corpus_id: left_tok_pos.corpus_id,
        text_id: left_tok_pos.text_id,
        val: 0,
    };

    for i in left_tok_pos.val..=right_tok_pos.val {
        tok_idx.val = i;

        let tok_id = load_node_and_corpus_result
            .textpos_table
            .token_by_index
            .try_get(&tok_idx)?
            .ok_or_else(|| RelAnnisError::NoTokenForPosition(tok_idx.clone()))?;
        if n != tok_id {
            let edge = Edge {
                source: n,
                target: tok_id,
            };

            // only add edge if no other coverage edge exists
            if !load_rank_result
                .text_coverage_edges
                .try_contains_key(&edge)?
            {
                let nodes_with_same_source = (
                    Bound::Included(Edge {
                        source: n,
                        target: NodeID::min_value(),
                    }),
                    Bound::Included(Edge {
                        source: n,
                        target: NodeID::max_value(),
                    }),
                );
                let has_any_outgoing_text_coverage_edge = load_rank_result
                    .text_coverage_edges
                    .range(nodes_with_same_source)
                    .next()
                    .is_some();
                let (component_layer, component_name) = if has_any_outgoing_text_coverage_edge {
                    // this is an additional auto-generated coverage edge, mark it as such
                    (ANNIS_NS.into(), "autogenerated-coverage".into())
                } else {
                    // Get the original component name for this target node
                    load_rank_result
                        .component_for_parentless_target_node
                        .try_get(&n)?
                        .map_or_else(
                            || (String::default(), String::default()),
                            |c| (c.layer, c.name),
                        )
                };

                updates.add_event(UpdateEvent::AddEdge {
                    source_node: load_node_and_corpus_result
                        .id_to_node_name
                        .try_get(&n)?
                        .ok_or(RelAnnisError::NodeNotFound(n))?
                        .into(),
                    target_node: load_node_and_corpus_result
                        .id_to_node_name
                        .try_get(&tok_id)?
                        .ok_or(RelAnnisError::NodeNotFound(tok_id))?
                        .into(),
                    layer: component_layer.into(),
                    component_type: AnnotationComponentType::Coverage.to_string(),
                    component_name: component_name.into(),
                })?;
            }
        }
    }

    Ok(())
}

fn calculate_automatic_coverage_edges<F>(
    updates: &mut GraphUpdate,
    load_node_and_corpus_result: &LoadNodeAndCorpusResult,
    load_rank_result: &LoadRankResult,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    // add explicit coverage edges for each node in the special annis namespace coverage component
    progress_callback("calculating the automatically generated Coverage edges");

    for (n, textprop) in load_node_and_corpus_result
        .textpos_table
        .node_to_left
        .try_iter()?
    {
        // Do not calculate automatic coverage edges for token
        if textprop.segmentation.is_empty()
            && !load_node_and_corpus_result
                .textpos_table
                .token_to_index
                .try_contains_key(&n)?
        {
            if let Err(e) = add_automatic_cov_edge_for_node(
                updates,
                n,
                load_node_and_corpus_result,
                load_rank_result,
            ) {
                // output a warning but do not fail
                warn!(
                    "Adding coverage edges (connects spans with tokens) failed: {}",
                    e
                )
            }
        } // end if not a token
    }

    Ok(())
}

fn add_white_space_token_labels<F>(
    updates: &mut GraphUpdate,
    textpos_table: &TextPosTable,
    texts: &mut DiskMap<TextKey, Text>,
    id_to_node_name: &DiskMap<NodeID, String>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    progress_callback("adding non-tokenized primary text segments as white-space label to tokens");
    let mut added_whitespace_label_count = 0;

    // Iterate over all texts of the graph separately
    for (text_key, text) in texts.try_iter()? {
        let mut text_char_it = text.val.chars();
        let mut current_text_offset = 0;

        let min_text_prop = TextProperty {
            corpus_id: text_key.corpus_ref.unwrap_or_default(),
            text_id: text_key.id,
            segmentation: "".into(),
            val: u32::min_value(),
        };
        let max_text_prop = TextProperty {
            corpus_id: text_key.corpus_ref.unwrap_or_default(),
            text_id: text_key.id,
            segmentation: "".into(),
            val: u32::max_value(),
        };

        let mut previous_token_id = None;

        // Go through each discovered token of this text and check if there is whitespace after this token in the original text.
        let mut token_iterator = textpos_table
            .token_by_index
            .range(min_text_prop..max_text_prop)
            .peekable();
        while let Some((_, current_token_id)) = token_iterator.next() {
            // Get the character borders for this token
            if let (Some(left_text_pos), Some(right_text_pos)) = (
                textpos_table.node_to_left_char.try_get(&current_token_id)?,
                textpos_table
                    .node_to_right_char
                    .try_get(&current_token_id)?,
            ) {
                let token_left_char = left_text_pos.val as usize;
                let token_right_char = right_text_pos.val as usize;

                if previous_token_id.is_none() && current_text_offset < token_left_char {
                    // We need to add the potential whitespace before this token as label
                    let mut covered_text_before =
                        std::string::String::with_capacity(token_left_char - current_text_offset);
                    let mut skipped_before_token = 0;
                    for _ in current_text_offset..token_left_char {
                        if let Some(c) = text_char_it.next() {
                            covered_text_before.push(c);
                            skipped_before_token += 1;
                        }
                    }
                    current_text_offset += skipped_before_token;

                    if let Some(token_name) = id_to_node_name.try_get(&current_token_id)? {
                        updates.add_event(UpdateEvent::AddNodeLabel {
                            node_name: token_name.into(),
                            anno_ns: ANNIS_NS.to_string(),
                            anno_name: TOK_WHITESPACE_BEFORE.to_string(),
                            anno_value: covered_text_before,
                        })?;
                        added_whitespace_label_count += 1;
                    }
                }

                // Skip the characters of the current token
                let mut skipped_token_characters = 0;
                for _ in current_text_offset..token_right_char {
                    if text_char_it.next().is_some() {
                        skipped_token_characters += 1;
                    }
                }
                current_text_offset += skipped_token_characters;

                // Get the token borders of the next token to determine where the whitespace after this token is
                // The whitespace end position is non-inclusive.
                let mut whitespace_end_pos = None;
                if let Some((_, next_token_id)) = token_iterator.peek() {
                    if let Some(next_token_left_pos) =
                        textpos_table.node_to_left_char.try_get(next_token_id)?
                    {
                        whitespace_end_pos = Some(next_token_left_pos.val as usize);
                    }
                }

                // Get the covered text which either goes until the next token or until the end of the text if there is none
                let mut covered_text_after = if let Some(end_pos) = whitespace_end_pos {
                    std::string::String::with_capacity(
                        end_pos.checked_sub(current_text_offset).unwrap_or(0),
                    )
                } else {
                    std::string::String::default()
                };

                if let Some(end_pos) = whitespace_end_pos {
                    let covered_text_start = current_text_offset;
                    for _ in covered_text_start..end_pos {
                        if let Some(c) = text_char_it.next() {
                            covered_text_after.push(c);
                            current_text_offset += 1;
                        }
                    }
                } else {
                    // Add all remaining text to the "tok-whitespace-after" annotation value.
                    // We can't borrow the iterator here (would not be an iterator) and we can't own it using a for-loop
                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(c) = text_char_it.next() {
                        covered_text_after.push(c);
                        current_text_offset += 1;
                    }
                }
                if let Some(token_name) = id_to_node_name.try_get(&current_token_id)? {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: token_name.into(),
                        anno_ns: ANNIS_NS.to_string(),
                        anno_name: TOK_WHITESPACE_AFTER.to_string(),
                        anno_value: covered_text_after,
                    })?;
                    added_whitespace_label_count += 1;
                }
            }
            previous_token_id = Some(current_token_id);
        }
    }
    progress_callback(&format!(
        "added {} non-tokenized primary text segments as white-space labels to the existing tokens",
        added_whitespace_label_count
    ));

    Ok(())
}

fn load_node_tab<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    texts: &mut DiskMap<TextKey, Text>,
    corpus_table: &ParsedCorpusTable,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<NodeTabParseResult>
where
    F: Fn(&str),
{
    let mut nodes_by_text: DiskMap<NodeByTextEntry, bool> = DiskMap::default();
    let mut missing_seg_span: DiskMap<NodeID, String> = DiskMap::default();
    let mut id_to_node_name: DiskMap<NodeID, String> = DiskMap::default();

    let mut node_tab_path = PathBuf::from(path);
    node_tab_path.push(if is_annis_33 {
        "node.annis"
    } else {
        "node.tab"
    });

    progress_callback(&format!(
        "loading {}",
        node_tab_path.to_str().unwrap_or_default()
    ));

    // maps a character position to it's token
    let mut textpos_table = TextPosTable {
        token_by_left_textpos: DiskMap::default(),
        token_by_right_textpos: DiskMap::default(),
        node_to_left: DiskMap::default(),
        node_to_right: DiskMap::default(),
        token_by_index: DiskMap::default(),
        token_to_index: DiskMap::default(),
        node_to_left_char: DiskMap::default(),
        node_to_right_char: DiskMap::default(),
    };

    // start "scan all lines" visibility block
    {
        let mut node_tab_csv = postgresql_import_reader(node_tab_path.as_path())?;

        for (line_nr, result) in node_tab_csv.records().enumerate() {
            let line = result?;

            let node_nr = get_field_not_null(&line, 0, "id", &node_tab_path)?.parse::<NodeID>()?;
            let has_segmentations = is_annis_33 || line.len() > 10;
            let token_index_raw = get_field(&line, 7, "token_index", &node_tab_path)?;
            let text_id =
                get_field_not_null(&line, 1, "text_ref", &node_tab_path)?.parse::<u32>()?;
            let corpus_id =
                get_field_not_null(&line, 2, "corpus_ref", &node_tab_path)?.parse::<u32>()?;
            let layer = get_field(&line, 3, "layer", &node_tab_path)?;
            let node_name = get_field_not_null(&line, 4, "name", &node_tab_path)?;

            nodes_by_text.insert(
                NodeByTextEntry {
                    corpus_ref: corpus_id,
                    text_id,
                    node_id: node_nr,
                },
                true,
            )?;

            // complete the corpus reference in the texts map for older relANNIS versions
            if !is_annis_33 {
                let text_key_without_corpus = TextKey {
                    id: text_id,
                    corpus_ref: None,
                };
                if let Some(existing_text) = texts.remove(&text_key_without_corpus)? {
                    let text_key = TextKey {
                        id: text_id,
                        corpus_ref: Some(corpus_id),
                    };
                    texts.insert(text_key, existing_text)?;
                }
            }

            let node_path = format!(
                "{}#{}",
                get_corpus_path(corpus_id, corpus_table)?,
                // fragments don't need escaping
                node_name
            );
            updates.add_event(UpdateEvent::AddNode {
                node_name: node_path.clone(),
                node_type: "node".to_owned(),
            })?;
            id_to_node_name.insert(node_nr, node_path.clone().into())?;

            if let Some(layer) = layer {
                if !layer.is_empty() {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: node_path.clone(),
                        anno_ns: ANNIS_NS.to_owned(),
                        anno_name: "layer".to_owned(),
                        anno_value: layer.to_string(),
                    })?;
                }
            }

            // Add the raw character offsets so it is possible to extract the text later on
            let left_char_val =
                get_field_not_null(&line, 5, "left", &node_tab_path)?.parse::<u32>()?;
            let left_char_pos = TextProperty {
                segmentation: String::from(""),
                val: left_char_val,
                corpus_id,
                text_id,
            };
            let right_char_val =
                get_field_not_null(&line, 6, "right", &node_tab_path)?.parse::<u32>()?;
            let right_char_pos = TextProperty {
                segmentation: String::from(""),
                val: right_char_val,
                corpus_id,
                text_id,
            };
            textpos_table
                .node_to_left_char
                .insert(node_nr, left_char_pos)?;
            textpos_table
                .node_to_right_char
                .insert(node_nr, right_char_pos)?;

            // Use left/right token columns for relANNIS 3.3 and the left/right character column otherwise to determine which token are aligned.
            // For some malformed corpora, the token coverage information is more robust and guaranties that a node is
            // only left/right aligned to a single token.
            let left_alignment_column = if is_annis_33 { 8 } else { 5 };
            let right_alignment_column = if is_annis_33 { 9 } else { 6 };

            let left_alignment_val =
                get_field_not_null(&line, left_alignment_column, "left_token", &node_tab_path)?
                    .parse::<u32>()?;
            let left_alignment = TextProperty {
                segmentation: String::from(""),
                val: left_alignment_val,
                corpus_id,
                text_id,
            };
            let right_alignment_val =
                get_field_not_null(&line, right_alignment_column, "right_token", &node_tab_path)?
                    .parse::<u32>()?;
            let right_alignment = TextProperty {
                segmentation: String::from(""),
                val: right_alignment_val,
                corpus_id,
                text_id,
            };
            textpos_table
                .node_to_left
                .insert(node_nr, left_alignment.clone())?;
            textpos_table
                .node_to_right
                .insert(node_nr, right_alignment.clone())?;

            if let Some(token_index_raw) = token_index_raw {
                let span = if has_segmentations {
                    get_field_not_null(&line, 12, "span", &node_tab_path)?
                } else {
                    get_field_not_null(&line, 9, "span", &node_tab_path)?
                };

                updates.add_event(UpdateEvent::AddNodeLabel {
                    node_name: node_path,
                    anno_ns: ANNIS_NS.to_owned(),
                    anno_name: TOK.to_owned(),
                    anno_value: span.to_string(),
                })?;

                let index = TextProperty {
                    segmentation: String::from(""),
                    val: token_index_raw.parse::<u32>()?,
                    text_id,
                    corpus_id,
                };
                textpos_table
                    .token_by_index
                    .insert(index.clone(), node_nr)?;
                textpos_table.token_to_index.insert(node_nr, index)?;
                textpos_table
                    .token_by_left_textpos
                    .insert(left_alignment, node_nr)?;
                textpos_table
                    .token_by_right_textpos
                    .insert(right_alignment, node_nr)?;
            } else if has_segmentations {
                let segmentation_name = if is_annis_33 {
                    get_field(&line, 11, "seg_name", &node_tab_path)?
                } else {
                    get_field(&line, 8, "seg_name", &node_tab_path)?
                };

                if let Some(segmentation_name) = segmentation_name {
                    let seg_index = if is_annis_33 {
                        get_field_not_null(&line, 10, "seg_index", &node_tab_path)?
                            .parse::<u32>()?
                    } else {
                        get_field_not_null(&line, 9, "seg_index", &node_tab_path)?.parse::<u32>()?
                    };

                    if is_annis_33 {
                        // directly add the span information
                        updates.add_event(UpdateEvent::AddNodeLabel {
                            node_name: node_path,
                            anno_ns: ANNIS_NS.to_owned(),
                            anno_name: TOK.to_owned(),
                            anno_value: get_field_not_null(&line, 12, "span", &node_tab_path)?
                                .to_string(),
                        })?;
                    } else {
                        // we need to get the span information from the node_annotation file later
                        missing_seg_span.insert(node_nr, segmentation_name.clone())?;
                    }
                    // also add the specific segmentation index
                    let index = TextProperty {
                        segmentation: segmentation_name,
                        val: seg_index,
                        corpus_id,
                        text_id,
                    };
                    textpos_table.token_by_index.insert(index, node_nr)?;
                } // end if node has segmentation info
            } // endif if check segmentations

            if (line_nr + 1) % 100_000 == 0 {
                progress_callback(&format!(
                    "loaded {} lines from {}",
                    line_nr + 1,
                    node_tab_path.to_str().unwrap_or_default()
                ));
            }
        }
    } // end "scan all lines" visibility block

    info!(
        "creating index for content of {}",
        &node_tab_path.to_string_lossy()
    );
    id_to_node_name.compact()?;
    nodes_by_text.compact()?;
    missing_seg_span.compact()?;
    textpos_table.node_to_left.compact()?;
    textpos_table.node_to_right.compact()?;
    textpos_table.node_to_left_char.compact()?;
    textpos_table.node_to_right_char.compact()?;
    textpos_table.token_to_index.compact()?;
    textpos_table.token_by_index.compact()?;
    textpos_table.token_by_left_textpos.compact()?;
    textpos_table.token_by_right_textpos.compact()?;

    if !(textpos_table.token_by_index.try_is_empty())? {
        calculate_automatic_token_order(
            updates,
            &textpos_table.token_by_index,
            &id_to_node_name,
            progress_callback,
        )?;
    } // end if token_by_index not empty

    Ok(NodeTabParseResult {
        nodes_by_text,
        missing_seg_span,
        id_to_node_name,
        textpos_table,
    })
}

fn load_node_anno_tab<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    missing_seg_span: &DiskMap<NodeID, String>,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    let mut node_anno_tab_path = PathBuf::from(path);
    node_anno_tab_path.push(if is_annis_33 {
        "node_annotation.annis"
    } else {
        "node_annotation.tab"
    });

    progress_callback(&format!(
        "loading {}",
        node_anno_tab_path.to_str().unwrap_or_default()
    ));

    let mut node_anno_tab_csv = postgresql_import_reader(node_anno_tab_path.as_path())?;

    for (line_nr, result) in node_anno_tab_csv.records().enumerate() {
        let line = result?;

        let col_id = get_field_not_null(&line, 0, "id", &node_anno_tab_path)?;
        let node_id: NodeID = col_id.parse()?;
        let node_name = id_to_node_name
            .try_get(&node_id)?
            .ok_or(RelAnnisError::NodeNotFound(node_id))?;
        let col_ns = get_field(&line, 1, "namespace", &node_anno_tab_path)?.unwrap_or_default();
        let col_name = get_field_not_null(&line, 2, "name", &node_anno_tab_path)?;
        let col_val = get_field(&line, 3, "value", &node_anno_tab_path)?;
        // we have to make some sanity checks
        if col_ns != "annis" || col_name != "tok" {
            let has_valid_value = col_val.is_some();
            // If 'NULL', use an "invalid" string so it can't be found by its value, but only by its annotation name
            let anno_val = &col_val.unwrap_or_else(|| INVALID_STRING.clone());

            if let Some(seg) = missing_seg_span.try_get(&node_id)? {
                // add all missing span values from the annotation, but don't add NULL values
                if seg == col_name.as_ref() && has_valid_value {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: node_name.clone().into(),
                        anno_ns: ANNIS_NS.to_owned(),
                        anno_name: TOK.to_owned(),
                        anno_value: anno_val.to_string(),
                    })?;
                }
            }

            updates.add_event(UpdateEvent::AddNodeLabel {
                node_name: node_name.into(),
                anno_ns: col_ns.to_string(),
                anno_name: col_name.to_string(),
                anno_value: anno_val.to_string(),
            })?;
        }

        if (line_nr + 1) % 100_000 == 0 {
            progress_callback(&format!(
                "loaded {} lines from {}",
                line_nr + 1,
                node_anno_tab_path.to_str().unwrap_or_default()
            ));
        }
    }

    Ok(())
}

fn load_component_tab<F>(
    path: &Path,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<BTreeMap<u32, Component<AnnotationComponentType>>>
where
    F: Fn(&str),
{
    let mut component_tab_path = PathBuf::from(path);
    component_tab_path.push(if is_annis_33 {
        "component.annis"
    } else {
        "component.tab"
    });

    progress_callback(&format!(
        "loading {}",
        component_tab_path.to_str().unwrap_or_default()
    ));

    let mut component_by_id: BTreeMap<u32, Component<AnnotationComponentType>> = BTreeMap::new();

    let mut component_tab_csv = postgresql_import_reader(component_tab_path.as_path())?;
    for result in component_tab_csv.records() {
        let line = result?;

        let cid: u32 = get_field_not_null(&line, 0, "id", &component_tab_path)?.parse()?;
        if let Some(col_type) = get_field(&line, 1, "type", &component_tab_path)? {
            let layer = get_field(&line, 2, "layer", &component_tab_path)?.unwrap_or_default();
            let name = get_field(&line, 3, "name", &component_tab_path)?.unwrap_or_default();
            let ctype = component_type_from_short_name(&col_type)?;
            component_by_id.insert(cid, Component::new(ctype, layer, name));
        }
    }
    Ok(component_by_id)
}

fn load_nodes<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    texts: &mut DiskMap<TextKey, Text>,
    corpus_table: &ParsedCorpusTable,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadNodeResult>
where
    F: Fn(&str),
{
    let node_tab_parse_result = load_node_tab(
        path,
        updates,
        texts,
        corpus_table,
        is_annis_33,
        progress_callback,
    )?;

    load_node_anno_tab(
        path,
        updates,
        &node_tab_parse_result.missing_seg_span,
        &node_tab_parse_result.id_to_node_name,
        is_annis_33,
        progress_callback,
    )?;

    Ok(LoadNodeResult {
        nodes_by_text: node_tab_parse_result.nodes_by_text,
        id_to_node_name: node_tab_parse_result.id_to_node_name,
        textpos_table: node_tab_parse_result.textpos_table,
    })
}

fn load_rank_tab<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    component_by_id: &BTreeMap<u32, Component<AnnotationComponentType>>,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadRankResult>
where
    F: Fn(&str),
{
    let mut rank_tab_path = PathBuf::from(path);
    rank_tab_path.push(if is_annis_33 {
        "rank.annis"
    } else {
        "rank.tab"
    });

    progress_callback(&format!(
        "loading {}",
        rank_tab_path.to_str().unwrap_or_default()
    ));

    let mut load_rank_result = LoadRankResult {
        components_by_pre: DiskMap::default(),
        edges_by_pre: DiskMap::default(),
        text_coverage_edges: DiskMap::default(),
        component_for_parentless_target_node: DiskMap::default(),
    };

    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    let pos_node_ref = if is_annis_33 { 3 } else { 2 };
    let pos_component_ref = if is_annis_33 { 4 } else { 3 };
    let pos_parent = if is_annis_33 { 5 } else { 4 };

    // first run: collect all pre-order values for a node
    let mut pre_to_node_id: DiskMap<u32, NodeID> = DiskMap::default();
    for result in rank_tab_csv.records() {
        let line = result?;
        let pre: u32 = get_field_not_null(&line, 0, "pre", &rank_tab_path)?.parse()?;
        let node_id: NodeID =
            get_field_not_null(&line, pos_node_ref, "node_ref", &rank_tab_path)?.parse()?;
        pre_to_node_id.insert(pre, node_id)?;
    }

    // second run: get the actual edges
    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    for result in rank_tab_csv.records() {
        let line = result?;

        let component_ref: u32 =
            get_field_not_null(&line, pos_component_ref, "component_ref", &rank_tab_path)?
                .parse()?;

        let target: NodeID =
            get_field_not_null(&line, pos_node_ref, "node_ref", &rank_tab_path)?.parse()?;

        if let Some(parent_as_str) = get_field(&line, pos_parent, "parent", &rank_tab_path)? {
            let parent: u32 = parent_as_str.parse()?;
            if let Some(source) = pre_to_node_id.get(&parent) {
                // find the responsible edge database by the component ID
                if let Some(c) = component_by_id.get(&component_ref) {
                    updates.add_event(UpdateEvent::AddEdge {
                        source_node: id_to_node_name
                            .try_get(&source)?
                            .ok_or(RelAnnisError::NodeNotFound(source))?
                            .into(),
                        target_node: id_to_node_name
                            .try_get(&target)?
                            .ok_or(RelAnnisError::NodeNotFound(target))?
                            .into(),
                        layer: c.layer.clone().into(),
                        component_type: c.get_type().to_string(),
                        component_name: c.name.clone().into(),
                    })?;

                    let pre: u32 = get_field_not_null(&line, 0, "pre", &rank_tab_path)?.parse()?;

                    let e = Edge { source, target };

                    if c.get_type() == AnnotationComponentType::Coverage {
                        load_rank_result
                            .text_coverage_edges
                            .insert(e.clone(), true)?;
                    }
                    load_rank_result.components_by_pre.insert(pre, c.clone())?;
                    load_rank_result.edges_by_pre.insert(pre, e)?;
                }
            }
        } else if let Some(c) = component_by_id.get(&component_ref) {
            if c.get_type() == AnnotationComponentType::Coverage {
                load_rank_result
                    .component_for_parentless_target_node
                    .insert(target, c.clone())?;
            }
        }
    }

    info!(
        "creating index for content of {}",
        &rank_tab_path.to_string_lossy()
    );
    load_rank_result.components_by_pre.compact()?;
    load_rank_result.edges_by_pre.compact()?;
    load_rank_result.text_coverage_edges.compact()?;
    load_rank_result
        .component_for_parentless_target_node
        .compact()?;

    Ok(load_rank_result)
}

fn load_edge_annotation<F>(
    path: &Path,
    updates: &mut GraphUpdate,
    rank_result: &LoadRankResult,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str),
{
    let mut edge_anno_tab_path = PathBuf::from(path);
    edge_anno_tab_path.push(if is_annis_33 {
        "edge_annotation.annis"
    } else {
        "edge_annotation.tab"
    });

    progress_callback(&format!(
        "loading {}",
        edge_anno_tab_path.to_str().unwrap_or_default()
    ));

    let mut edge_anno_tab_csv = postgresql_import_reader(edge_anno_tab_path.as_path())?;

    for result in edge_anno_tab_csv.records() {
        let line = result?;

        let pre = get_field_not_null(&line, 0, "pre", &edge_anno_tab_path)?.parse::<u32>()?;
        if let Some(c) = rank_result.components_by_pre.try_get(&pre)? {
            if let Some(e) = rank_result.edges_by_pre.try_get(&pre)? {
                let ns = get_field(&line, 1, "namespace", &edge_anno_tab_path)?.unwrap_or_default();
                let name = get_field_not_null(&line, 2, "name", &edge_anno_tab_path)?;
                // If 'NULL', use an "invalid" string so it can't be found by its value, but only by its annotation name
                let val = get_field(&line, 3, "value", &edge_anno_tab_path)?
                    .unwrap_or_else(|| INVALID_STRING.clone());

                updates.add_event(UpdateEvent::AddEdgeLabel {
                    source_node: id_to_node_name
                        .try_get(&e.source)?
                        .ok_or(RelAnnisError::NodeNotFound(e.source))?
                        .into(),
                    target_node: id_to_node_name
                        .try_get(&e.target)?
                        .ok_or(RelAnnisError::NodeNotFound(e.target))?
                        .into(),
                    layer: c.layer.clone().into(),
                    component_type: c.get_type().to_string(),
                    component_name: c.name.into(),
                    anno_ns: ns.to_string(),
                    anno_name: name.to_string(),
                    anno_value: val.to_string(),
                })?;
            }
        }
    }

    Ok(())
}

fn load_corpus_annotation<F>(
    path: &Path,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<BTreeMap<(u32, AnnoKey), std::string::String>>
where
    F: Fn(&str),
{
    let mut corpus_id_to_anno = BTreeMap::new();

    let mut corpus_anno_tab_path = PathBuf::from(path);
    corpus_anno_tab_path.push(if is_annis_33 {
        "corpus_annotation.annis"
    } else {
        "corpus_annotation.tab"
    });

    progress_callback(&format!(
        "loading {}",
        corpus_anno_tab_path.to_str().unwrap_or_default()
    ));

    let mut corpus_anno_tab_csv = postgresql_import_reader(corpus_anno_tab_path.as_path())?;

    for result in corpus_anno_tab_csv.records() {
        let line = result?;

        let id = get_field_not_null(&line, 0, "id", &corpus_anno_tab_path)?.parse::<u32>()?;
        let ns = get_field(&line, 1, "namespace", &corpus_anno_tab_path)?.unwrap_or_default();
        let name = get_field_not_null(&line, 2, "name", &corpus_anno_tab_path)?;
        // If 'NULL', use an "invalid" string so it can't be found by its value, but only by its annotation name
        let val = get_field(&line, 3, "value", &corpus_anno_tab_path)?
            .unwrap_or_else(|| INVALID_STRING.clone());

        let anno_key = AnnoKey { ns, name };

        corpus_id_to_anno.insert((id, anno_key), val.to_string());
    }

    Ok(corpus_id_to_anno)
}

fn get_parent_path(cid: u32, corpus_table: &ParsedCorpusTable) -> Result<std::string::String> {
    let corpus = corpus_table
        .corpus_by_id
        .get(&cid)
        .ok_or(RelAnnisError::CorpusNotFound(cid))?;
    let pre = corpus.pre;
    let post = corpus.post;

    Ok(corpus_table
        .corpus_by_preorder
        .range(0..pre)
        .filter_map(|(_, cid)| corpus_table.corpus_by_id.get(cid))
        .filter(|parent_corpus| post < parent_corpus.post)
        .map(|parent_corpus| parent_corpus.normalized_name.clone())
        .join("/"))
}

fn get_corpus_path(cid: u32, corpus_table: &ParsedCorpusTable) -> Result<String> {
    let mut result: String = get_parent_path(cid, corpus_table)?.into();
    let corpus = corpus_table
        .corpus_by_id
        .get(&cid)
        .ok_or(RelAnnisError::CorpusNotFound(cid))?;
    result.push_str("/");
    result.push_str(&corpus.normalized_name);
    Ok(result)
}

fn add_subcorpora(
    updates: &mut GraphUpdate,
    corpus_table: &ParsedCorpusTable,
    node_node_result: &LoadNodeResult,
    texts: &DiskMap<TextKey, Text>,
    corpus_id_to_annos: &BTreeMap<(u32, AnnoKey), std::string::String>,
    is_annis_33: bool,
    path: &Path,
) -> Result<()> {
    // add the toplevel corpus as node
    {
        updates.add_event(UpdateEvent::AddNode {
            node_name: corpus_table.toplevel_corpus_name.as_str().into(),
            node_type: "corpus".into(),
        })?;

        // save the relANNIS version as meta data attribute on the toplevel corpus
        updates.add_event(UpdateEvent::AddNodeLabel {
            node_name: corpus_table.toplevel_corpus_name.as_str().into(),
            anno_ns: ANNIS_NS.to_owned(),
            anno_name: "relannis-version".to_owned(),
            anno_value: if is_annis_33 {
                "3.3".to_owned()
            } else {
                "3.2".to_owned()
            },
        })?;

        // add all metadata for the top-level corpus node
        if let Some(cid) = corpus_table.corpus_by_preorder.get(&0) {
            let start_key = (
                *cid,
                AnnoKey {
                    ns: "".into(),
                    name: "".into(),
                },
            );
            for ((entry_cid, anno_key), val) in corpus_id_to_annos.range(start_key..) {
                if entry_cid == cid {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: corpus_table.toplevel_corpus_name.as_str().into(),
                        anno_ns: anno_key.ns.clone().into(),
                        anno_name: anno_key.name.clone().into(),
                        anno_value: val.into(),
                    })?;
                } else {
                    break;
                }
            }
            add_external_data_files(path, &corpus_table.toplevel_corpus_name, None, updates)?;
        }
    }

    // add all subcorpora/documents (start with the smallest pre-order)
    for (pre, corpus_id) in corpus_table.corpus_by_preorder.iter() {
        if *pre != 0 {
            let corpus = corpus_table
                .corpus_by_id
                .get(corpus_id)
                .ok_or_else(|| RelAnnisError::CorpusNotFound(*corpus_id))?;

            let corpus_name = &corpus.name;

            let subcorpus_full_name = get_corpus_path(*corpus_id, corpus_table)?;

            // add a basic node labels for the new (sub-) corpus/document
            updates.add_event(UpdateEvent::AddNode {
                node_name: subcorpus_full_name.to_string(),
                node_type: "corpus".to_owned(),
            })?;
            updates.add_event(UpdateEvent::AddNodeLabel {
                node_name: subcorpus_full_name.to_string(),
                anno_ns: ANNIS_NS.to_owned(),
                anno_name: "doc".to_owned(),
                anno_value: corpus_name.as_str().into(),
            })?;

            // add all metadata for the document node
            let start_key = (
                *corpus_id,
                AnnoKey {
                    ns: "".into(),
                    name: "".into(),
                },
            );
            for ((entry_cid, anno_key), val) in corpus_id_to_annos.range(start_key..) {
                if entry_cid == corpus_id {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: subcorpus_full_name.to_string(),
                        anno_ns: anno_key.ns.clone().into(),
                        anno_name: anno_key.name.clone().into(),
                        anno_value: val.clone(),
                    })?;
                } else {
                    break;
                }
            }
            // add an edge from the document (or sub-corpus) to the top-level corpus
            updates.add_event(UpdateEvent::AddEdge {
                source_node: subcorpus_full_name.to_string(),
                target_node: corpus_table.toplevel_corpus_name.as_str().into(),
                layer: ANNIS_NS.to_owned(),
                component_type: AnnotationComponentType::PartOf.to_string(),
                component_name: std::string::String::default(),
            })?;

            add_external_data_files(path, &subcorpus_full_name, Some(corpus_name), updates)?;
        } // end if not toplevel corpus
    } // end for each document/sub-corpus

    // add a node for each text and the connection between all sub-nodes of the text
    for (text_key, text) in texts.iter() {
        // add text node (including its name)
        if let Some(corpus_ref) = text_key.corpus_ref {
            let text_name = utf8_percent_encode(&text.name, SALT_URI_ENCODE_SET).to_string();
            let subcorpus_full_name = get_corpus_path(corpus_ref, corpus_table)?;
            let text_full_name = format!("{}#{}", &subcorpus_full_name, &text_name);

            updates.add_event(UpdateEvent::AddNode {
                node_name: text_full_name.clone(),
                node_type: "datasource".to_owned(),
            })?;

            // add an edge from the text to the document
            updates.add_event(UpdateEvent::AddEdge {
                source_node: text_full_name.clone(),
                target_node: subcorpus_full_name.to_string(),
                layer: ANNIS_NS.to_owned(),
                component_type: AnnotationComponentType::PartOf.to_string(),
                component_name: std::string::String::default(),
            })?;

            // find all nodes belonging to this text and add a relation
            let min_key = NodeByTextEntry {
                corpus_ref,
                text_id: text_key.id,
                node_id: NodeID::min_value(),
            };
            let max_key = NodeByTextEntry {
                corpus_ref,
                text_id: text_key.id,
                node_id: NodeID::max_value(),
            };
            for (text_entry, _) in node_node_result.nodes_by_text.range(min_key..=max_key) {
                let n = text_entry.node_id;
                updates.add_event(UpdateEvent::AddEdge {
                    source_node: node_node_result
                        .id_to_node_name
                        .try_get(&n)?
                        .ok_or(RelAnnisError::NodeNotFound(n))?
                        .into(),
                    target_node: text_full_name.clone(),
                    layer: ANNIS_NS.to_owned(),
                    component_type: AnnotationComponentType::PartOf.to_string(),
                    component_name: std::string::String::default(),
                })?;
            }
        }
    } // end for each text
    Ok(())
}

fn component_type_from_short_name(short_type: &str) -> Result<AnnotationComponentType> {
    match short_type {
        "c" => Ok(AnnotationComponentType::Coverage),
        "d" => Ok(AnnotationComponentType::Dominance),
        "p" => Ok(AnnotationComponentType::Pointing),
        "o" => Ok(AnnotationComponentType::Ordering),
        _ => Err(RelAnnisError::InvalidComponentShortName(short_type.to_string()).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            normalized_name: "document".into(),
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
        assert_eq!(true, result.missing_seg_span.contains_key(&680));
    }
}
