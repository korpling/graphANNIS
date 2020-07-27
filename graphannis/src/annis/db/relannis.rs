use super::aql::model::{AnnotationComponentType, IGNORED_TOK};
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
use csv;
use graphannis_core::{
    graph::{ANNIS_NS, DEFAULT_NS},
    serializer::KeySerializer,
    types::{AnnoKey, Component, Edge, NodeID},
    util::disk_collections::DiskMap,
};
use percent_encoding::utf8_percent_encode;
use std;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Bound::Included;
use std::path::{Path, PathBuf};

#[derive(
    Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize, MallocSizeOf,
)]
struct TextProperty {
    segmentation: String,
    corpus_id: u32,
    text_id: u32,
    val: u32,
}

impl KeySerializer for TextProperty {
    fn create_key<'a>(&'a self) -> Cow<'a, [u8]> {
        let mut result =
            Vec::with_capacity(self.segmentation.len() + 1 + std::mem::size_of::<u32>() * 3);
        result.extend(create_str_vec_key(&[&self.segmentation]));
        result.extend(&self.corpus_id.to_be_bytes());
        result.extend(&self.text_id.to_be_bytes());
        result.extend(&self.val.to_be_bytes());
        Cow::Owned(result)
    }

    fn parse_key(key: &[u8]) -> Self {
        let id_size = std::mem::size_of::<u32>();
        let mut id_offset = key.len() - id_size * 3;
        let key_as_string = String::from_utf8_lossy(key);
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
            segmentation_vector[0].to_string()
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
    fn create_key<'a>(&'a self) -> Cow<'a, [u8]> {
        let mut result = Vec::with_capacity(std::mem::size_of::<u32>() * 2);
        result.extend(&self.id.to_be_bytes());
        if let Some(corpus_ref) = self.corpus_ref {
            result.extend(&corpus_ref.to_be_bytes());
        }
        Cow::Owned(result)
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
    fn create_key<'a>(&'a self) -> Cow<'a, [u8]> {
        let mut result =
            Vec::with_capacity(std::mem::size_of::<u32>() * 2 + std::mem::size_of::<NodeID>());
        result.extend(&self.text_id.to_be_bytes());
        result.extend(&self.corpus_ref.to_be_bytes());
        result.extend(&self.node_id.to_be_bytes());
        Cow::Owned(result)
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
    // map as node to it's "left" value
    node_to_left: DiskMap<NodeID, TextProperty>,
    // map as node to it's "right" value
    node_to_right: DiskMap<NodeID, TextProperty>,
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
    F: Fn(&str) -> (),
{
    // convert to path
    let path = PathBuf::from(path);
    if path.is_dir() && path.exists() {
        // check if this is the ANNIS 3.3 import format
        let annis_version_path = path.clone().join("annis.version");
        let is_annis_33 = if annis_version_path.exists() {
            let mut file = File::open(&annis_version_path)?;
            let mut version_str = String::new();
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

        db.apply_update(&mut updates, &progress_callback)?;

        progress_callback("calculating node statistics");
        db.get_node_annos_mut().calculate_statistics();

        for c in db.get_all_components(None, None) {
            progress_callback(&format!("calculating statistics for component {}", c));
            db.calculate_component_statistics(&c)?;
            db.optimize_impl(&c)?;
        }

        progress_callback(&format!(
            "finished loading relANNIS from {}",
            path.to_string_lossy()
        ));

        return Ok((load_node_and_corpus_result.toplevel_corpus_name, db, config));
    }

    Err(anyhow!("Directory {} not found", path.to_string_lossy()))
}

fn load_node_and_corpus_tables<F>(
    path: &PathBuf,
    updates: &mut GraphUpdate,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadNodeAndCorpusResult>
where
    F: Fn(&str) -> (),
{
    let corpus_table = parse_corpus_tab(&path, is_annis_33, &progress_callback)?;
    let mut texts = parse_text_tab(&path, is_annis_33, &progress_callback)?;
    let corpus_id_to_annos = load_corpus_annotation(&path, is_annis_33, &progress_callback)?;

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

    add_white_space_token(
        updates,
        &load_nodes_result.textpos_table,
        &mut texts,
        &load_nodes_result.id_to_node_name,
        &corpus_table,
        progress_callback,
    )?;

    Ok(LoadNodeAndCorpusResult {
        toplevel_corpus_name: corpus_table.toplevel_corpus_name,
        id_to_node_name: load_nodes_result.id_to_node_name,
        textpos_table: load_nodes_result.textpos_table,
    })
}

fn load_edge_tables<F>(
    path: &PathBuf,
    updates: &mut GraphUpdate,
    is_annis_33: bool,
    id_to_node_name: &DiskMap<NodeID, String>,
    progress_callback: &F,
) -> Result<LoadRankResult>
where
    F: Fn(&str) -> (),
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
    F: Fn(&str) -> (),
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

    let mut rules_by_order: Vec<(i64, VisualizerRule)> = Vec::new();

    for result in resolver_tab_csv.records() {
        let line = result?;

        let layer = get_field_str(&line, 2).filter(|l| l != "NULL");
        let element = get_field_str(&line, 3).map_or(None, |e| match e.as_str() {
            "node" => Some(VisualizerRuleElement::Node),
            "edge" => Some(VisualizerRuleElement::Edge),
            _ => None,
        });
        let vis_type =
            get_field_str(&line, 4).ok_or(anyhow!("Missing vis_type column in resolver table"))?;
        let display_name = get_field_str(&line, 5)
            .ok_or(anyhow!("Missing display_name column in resolver table"))?;

        let visibility = get_field_str(&line, 6)
            .ok_or(anyhow!("Missing visibility column in resolver table"))?;
        let visibility = match visibility.as_str() {
            "hidden" => VisualizerVisibility::Hidden,
            "visible" => VisualizerVisibility::Visible,
            "permanent" => VisualizerVisibility::Permanent,
            "preloaded" => VisualizerVisibility::Preloaded,
            "removed" => VisualizerVisibility::Removed,
            _ => VisualizerVisibility::default(),
        };
        let order =
            get_field_str(&line, 7).ok_or(anyhow!("Missing order column in resolver table"))?;
        let order = i64::from_str_radix(&order, 10).unwrap_or_default();

        let mappings: BTreeMap<String, String> =
            if let Some(mappings_field) = get_field_str(&line, 8) {
                mappings_field
                    .split(";")
                    .filter_map(|key_value| {
                        let splitted: Vec<_> = key_value.splitn(2, ":").collect();
                        if splitted.len() == 2 {
                            Some((splitted[0].to_string(), splitted[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                BTreeMap::new()
            };
        let rule = VisualizerRule {
            layer,
            element,
            vis_type: vis_type.to_string(),
            display_name: display_name.to_string(),
            visibility,
            mappings,
        };

        // Insert at sorted position by the order
        match rules_by_order.binary_search_by_key(&order, |(o, _)| *o) {
            Ok(idx) => rules_by_order.insert(idx + 1, (order, rule)),
            Err(idx) => rules_by_order.insert(idx, (order, rule)),
        }
    }

    config.visualizer = rules_by_order.into_iter().map(|(_, r)| r).collect();

    Ok(())
}

fn load_example_queries<F>(
    path: &Path,
    config: &mut CorpusConfiguration,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
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

        if let (Some(query), Some(description)) = (get_field_str(&line, 0), get_field_str(&line, 1))
        {
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
    F: Fn(&str) -> (),
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
        let splitted: Vec<_> = line.splitn(2, "=").collect();
        if splitted.len() == 2 {
            let key = splitted[0];
            let value = splitted[1];

            match key {
                "max-context" => {
                    if let Ok(value) = usize::from_str_radix(value, 10) {
                        config.context.max = Some(value);
                    }
                }
                "default-context" => {
                    if let Ok(value) = usize::from_str_radix(value, 10) {
                        config.context.default = value;
                    }
                }
                "results-per-page" => {
                    if let Ok(value) = usize::from_str_radix(value, 10) {
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
                _ => {}
            };
        }
    }

    // The context step is dependent on the max-context configuration.
    // Use a second pass to make sure it already has been set.
    for line in content.lines() {
        // split into key and value
        let splitted: Vec<_> = line.splitn(2, "=").collect();
        if splitted.len() == 2 {
            let key = splitted[0];
            let value = splitted[1];

            match key {
                "context-steps" => {
                    if let Ok(value) = usize::from_str_radix(value, 10) {
                        config.context.sizes = (value..=config.context.max.unwrap_or(value))
                            .step_by(value)
                            .collect();
                    }
                }
                _ => {}
            };
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
                    anno_value: file.path().to_string_lossy().to_string(),
                })?;
                updates.add_event(UpdateEvent::AddEdge {
                    source_node: node_name.clone(),
                    target_node: parent_node_full_name.to_owned(),
                    layer: ANNIS_NS.to_owned(),
                    component_type: AnnotationComponentType::PartOf.to_string(),
                    component_name: String::default(),
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

fn get_field_str(record: &csv::StringRecord, i: usize) -> Option<String> {
    if let Some(r) = record.get(i) {
        // replace some known escape sequences
        return Some(
            r.replace("\\t", "\t")
                .replace("\\'", "'")
                .replace("\\\\", "\\"),
        );
    }
    None
}

fn parse_corpus_tab<F>(
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<ParsedCorpusTable>
where
    F: Fn(&str) -> (),
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

    for result in corpus_tab_csv.records() {
        let line = result?;

        let id = line
            .get(0)
            .ok_or(anyhow!("Missing column"))?
            .parse::<u32>()?;
        let name = get_field_str(&line, 1).ok_or(anyhow!("Missing column"))?;
        let pre_order = line
            .get(4)
            .ok_or(anyhow!("Missing column"))?
            .parse::<u32>()?;
        let post_order = line
            .get(5)
            .ok_or(anyhow!("Missing column"))?
            .parse::<u32>()?;

        corpus_by_id.insert(
            id,
            CorpusTableEntry {
                pre: pre_order,
                post: post_order,
                name: name.clone(),
            },
        );

        corpus_by_preorder.insert(pre_order, id);
    }

    let toplevel_corpus_id = corpus_by_preorder
        .iter()
        .next()
        .ok_or(anyhow!("Toplevel corpus not found"))?
        .1;
    Ok(ParsedCorpusTable {
        toplevel_corpus_name: corpus_by_id
            .get(toplevel_corpus_id)
            .ok_or(anyhow!("Toplevel corpus name not found"))?
            .name
            .clone(),
        corpus_by_preorder,
        corpus_by_id,
    })
}

fn parse_text_tab<F>(
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<DiskMap<TextKey, Text>>
where
    F: Fn(&str) -> (),
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

        let id = line
            .get(if is_annis_33 { 1 } else { 0 })
            .ok_or(anyhow!("Missing column"))?
            .parse::<u32>()?;
        let name = get_field_str(&line, if is_annis_33 { 2 } else { 1 })
            .ok_or(anyhow!("Missing column"))?;

        let value = get_field_str(&line, if is_annis_33 { 3 } else { 2 })
            .ok_or(anyhow!("Missing column"))?;

        let corpus_ref = if is_annis_33 {
            Some(
                line.get(0)
                    .ok_or(anyhow!("Missing column"))?
                    .parse::<u32>()?,
            )
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
    F: Fn(&str) -> (),
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
                updates.add_event(
                    UpdateEvent::AddEdge {
                        source_node: id_to_node_name
                            .try_get(&last_token)?
                            .ok_or_else(|| anyhow!("Can't get node name for last token with ID {} in \"calculate_automatic_token_order\" function.", last_token))?
                            .clone(),
                        target_node: id_to_node_name
                            .try_get(&current_token)?
                            .ok_or_else(|| anyhow!("Can't get node name for current token with ID {} in \"calculate_automatic_token_order\" function.", current_token))?
                            .clone(),
                        layer: ordering_layer,
                        component_type: AnnotationComponentType::Ordering.to_string(),
                        component_name: current_textprop.segmentation.clone(),
                    },
                )?;
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
    left_pos: TextProperty,
    right_pos: TextProperty,
    load_node_and_corpus_result: &LoadNodeAndCorpusResult,
    load_rank_result: &LoadRankResult,
) -> Result<()> {
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
        right_aligned_tok.ok_or_else(|| {
            anyhow!(
                "Can't get both left- and right-aligned token for node {}",
                n
            )
        })?
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
        .ok_or_else(|| {
            anyhow!(
                "Can't get position of left-aligned token {}",
                left_aligned_tok
            )
        })?;
    let right_tok_pos = load_node_and_corpus_result
        .textpos_table
        .token_to_index
        .try_get(&right_aligned_tok)?
        .ok_or_else(|| {
            anyhow!(
                "Can't get position of right-aligned token {}",
                right_aligned_tok
            )
        })?;

    for i in left_tok_pos.val..=right_tok_pos.val {
        let tok_idx = TextProperty {
            segmentation: String::default(),
            corpus_id: left_tok_pos.corpus_id,
            text_id: left_tok_pos.text_id,
            val: i,
        };
        let tok_id = load_node_and_corpus_result
            .textpos_table
            .token_by_index
            .try_get(&tok_idx)?
            .ok_or_else(|| anyhow!("Can't get token ID for position {:?}", tok_idx))?;
        if n != tok_id {
            let edge = Edge {
                source: n,
                target: tok_id,
            };

            // only add edge of no other coverage edge exists
            if !load_rank_result
                .text_coverage_edges
                .try_contains_key(&edge)?
            {
                let nodes_with_same_source = (
                    Included(Edge {
                        source: n,
                        target: NodeID::min_value(),
                    }),
                    Included(Edge {
                        source: n,
                        target: NodeID::max_value(),
                    }),
                );
                let has_outgoing_text_coverage_edge = load_rank_result
                    .text_coverage_edges
                    .range(nodes_with_same_source)
                    .next()
                    .is_some();
                let (component_layer, component_name) = if has_outgoing_text_coverage_edge {
                    // this is an additional auto-generated coverage edge, mark it as such
                    (ANNIS_NS.to_owned(), "autogenerated-coverage".to_owned())
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
                        .ok_or(anyhow!("Missing node name"))?
                        .clone(),
                    target_node: load_node_and_corpus_result
                        .id_to_node_name
                        .try_get(&tok_id)?
                        .ok_or(anyhow!("Missing node name"))?
                        .clone(),
                    layer: component_layer,
                    component_type: AnnotationComponentType::Coverage.to_string(),
                    component_name: component_name,
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
    F: Fn(&str) -> (),
{
    // add explicit coverage edges for each node in the special annis namespace coverage component
    progress_callback("calculating the automatically generated Coverage edges");

    for (n, textprop) in load_node_and_corpus_result
        .textpos_table
        .node_to_left
        .try_iter()?
    {
        if textprop.segmentation == ""
            && !load_node_and_corpus_result
                .textpos_table
                .token_to_index
                .try_contains_key(&n)?
        {
            let left_pos = TextProperty {
                segmentation: String::from(""),
                corpus_id: textprop.corpus_id,
                text_id: textprop.text_id,
                val: textprop.val,
            };
            let right_pos = load_node_and_corpus_result
                .textpos_table
                .node_to_right
                .try_get(&n)?
                .ok_or_else(|| anyhow!("Can't get right position of node {}", n))?;
            let right_pos = TextProperty {
                segmentation: String::from(""),
                corpus_id: textprop.corpus_id,
                text_id: textprop.text_id,
                val: right_pos.val,
            };

            if let Err(e) = add_automatic_cov_edge_for_node(
                updates,
                n,
                left_pos,
                right_pos,
                &load_node_and_corpus_result,
                &load_rank_result,
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

fn add_white_space_token<F>(
    updates: &mut GraphUpdate,
    textpos_table: &TextPosTable,
    texts: &mut DiskMap<TextKey, Text>,
    id_to_node_name: &DiskMap<NodeID, String>,
    corpus_table: &ParsedCorpusTable,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    progress_callback("adding non-tokenized primary text segments as white-space tokens");
    let mut added_token_count = 0;

    // Iterate over all texts of the graph separately
    for (text_key, text) in texts.try_iter()? {
        let mut text_char_it = text.val.chars();
        let mut current_text_offset = 0;

        let min_text_prop = TextProperty {
            corpus_id: text_key.corpus_ref.unwrap_or_default(),
            text_id: text_key.id,
            segmentation: "".to_string(),
            val: u32::min_value(),
        };
        let max_text_prop = TextProperty {
            corpus_id: text_key.corpus_ref.unwrap_or_default(),
            text_id: text_key.id,
            segmentation: "".to_string(),
            val: u32::max_value(),
        };

        let mut previous_real_token_id = None;

        // Go through each discovered token of this text and check if there is whitespace before this token in the original text.
        for (text, next_real_token_id) in textpos_table
            .token_by_index
            .range(min_text_prop..max_text_prop)
        {
            // Get the left character border for this token
            if let (Some(left_text_pos), Some(right_text_pos)) = (
                textpos_table.node_to_left.try_get(&next_real_token_id)?,
                textpos_table.node_to_right.try_get(&next_real_token_id)?,
            ) {
                let token_left_char = left_text_pos.val as usize;
                let token_char_length = (right_text_pos.val - left_text_pos.val) as usize;
                if current_text_offset < token_left_char {
                    // Create the new white space token from the start to left border of this token
                    if let Some(t) = texts.try_get(&text_key)? {
                        let text_name =
                            utf8_percent_encode(&t.name, SALT_URI_ENCODE_SET).to_string();
                        let subcorpus_full_name = get_corpus_path(text.corpus_id, corpus_table)?;
                        let text_full_name = format!("{}#{}", &subcorpus_full_name, &text_name);

                        let created_token_id = format!(
                            "{}#white_space_token_{}_{}_{}_{}",
                            subcorpus_full_name,
                            text_name,
                            current_text_offset,
                            token_left_char,
                            added_token_count,
                        );

                        // Get the covered text
                        let mut covered_text =
                            String::with_capacity(token_left_char - current_text_offset);
                        for _ in current_text_offset..token_left_char {
                            if let Some(c) = text_char_it.next() {
                                covered_text.push(c);
                                current_text_offset += 1;
                            }
                        }

                        // Skip the text of the current token
                        for _ in 0..token_char_length {
                            if text_char_it.next().is_some() {
                                current_text_offset += 1;
                            }
                        }

                        // Add events
                        updates.add_event(UpdateEvent::AddNode {
                            node_name: created_token_id.clone(),
                            node_type: IGNORED_TOK.to_string(),
                        })?;
                        updates.add_event(UpdateEvent::AddNodeLabel {
                            node_name: created_token_id.clone(),
                            anno_ns: ANNIS_NS.to_string(),
                            anno_name: IGNORED_TOK.to_string(),
                            anno_value: covered_text.to_string(),
                        })?;

                        updates.add_event(UpdateEvent::AddEdge {
                            source_node: created_token_id.clone(),
                            target_node: text_full_name,
                            component_type: AnnotationComponentType::PartOf.to_string(),
                            component_name: String::default(),
                            layer: ANNIS_NS.to_owned(),
                        })?;

                        // Connect the new node with Ordering edges to the token before and after
                        if let Some(previous_real_token_id) = previous_real_token_id {
                            if let Some(previous_token) =
                                id_to_node_name.try_get(&previous_real_token_id)?
                            {
                                updates.add_event(UpdateEvent::AddEdge {
                                    source_node: previous_token.clone(),
                                    target_node: created_token_id.to_string(),
                                    component_type: AnnotationComponentType::Ordering.to_string(),
                                    component_name: "text".to_string(),
                                    layer: ANNIS_NS.to_string(),
                                })?;
                            }
                        }
                        if let Some(next_token) = id_to_node_name.try_get(&next_real_token_id)? {
                            updates.add_event(UpdateEvent::AddEdge {
                                source_node: created_token_id.to_string(),
                                target_node: next_token.to_string(),
                                component_type: AnnotationComponentType::Ordering.to_string(),
                                component_name: "text".to_string(),
                                layer: ANNIS_NS.to_string(),
                            })?;
                        }
                        added_token_count += 1;
                    }
                } else {
                    // There is no whitespace between the token, create a direct ordering edge between them
                    if let Some(previous_real_token_id) = previous_real_token_id {
                        if let (Some(previous_token), Some(next_token)) = (
                            id_to_node_name.try_get(&previous_real_token_id)?,
                            id_to_node_name.try_get(&next_real_token_id)?,
                        ) {
                            updates.add_event(UpdateEvent::AddEdge {
                                source_node: previous_token.to_string(),
                                target_node: next_token.to_string(),
                                component_type: AnnotationComponentType::Ordering.to_string(),
                                component_name: "text".to_string(),
                                layer: ANNIS_NS.to_string(),
                            })?;
                        }
                    }
                }
            }
            previous_real_token_id = Some(next_real_token_id);
        }
    }
    progress_callback(&format!(
        "added {} non-tokenized primary text segments as white-space tokens",
        added_token_count
    ));

    Ok(())
}

fn load_node_tab<F>(
    path: &PathBuf,
    updates: &mut GraphUpdate,
    texts: &mut DiskMap<TextKey, Text>,
    corpus_table: &ParsedCorpusTable,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<NodeTabParseResult>
where
    F: Fn(&str) -> (),
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
    };

    // start "scan all lines" visibility block
    {
        let mut node_tab_csv = postgresql_import_reader(node_tab_path.as_path())?;

        for (line_nr, result) in node_tab_csv.records().enumerate() {
            let line = result?;

            let node_nr = line
                .get(0)
                .ok_or(anyhow!("Missing column"))?
                .parse::<NodeID>()?;
            let has_segmentations = is_annis_33 || line.len() > 10;
            let token_index_raw = line.get(7).ok_or(anyhow!("Missing column"))?;
            let text_id = line
                .get(1)
                .ok_or(anyhow!("Missing column"))?
                .parse::<u32>()?;
            let corpus_id = line
                .get(2)
                .ok_or(anyhow!("Missing column"))?
                .parse::<u32>()?;
            let layer = get_field_str(&line, 3).ok_or(anyhow!("Missing column"))?;
            let node_name = get_field_str(&line, 4).ok_or(anyhow!("Missing column"))?;

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
            id_to_node_name.insert(node_nr, node_path.clone())?;

            if !layer.is_empty() && layer != "NULL" {
                updates.add_event(UpdateEvent::AddNodeLabel {
                    node_name: node_path.clone(),
                    anno_ns: ANNIS_NS.to_owned(),
                    anno_name: "layer".to_owned(),
                    anno_value: layer,
                })?;
            }

            // Use left/right token columns for relANNIS 3.3 and the left/right character column otherwise.
            // For some malformed corpora, the token coverage information is more robust and guaranties that a node is
            // only left/right aligned to a single token.
            let left_column = if is_annis_33 { 8 } else { 5 };
            let right_column = if is_annis_33 { 9 } else { 6 };

            let left_val = line
                .get(left_column)
                .ok_or(anyhow!("Missing column"))?
                .parse::<u32>()?;
            let left = TextProperty {
                segmentation: String::from(""),
                val: left_val,
                corpus_id,
                text_id,
            };
            let right_val = line
                .get(right_column)
                .ok_or(anyhow!("Missing column"))?
                .parse::<u32>()?;
            let right = TextProperty {
                segmentation: String::from(""),
                val: right_val,
                corpus_id,
                text_id,
            };
            textpos_table.node_to_left.insert(node_nr, left.clone())?;
            textpos_table.node_to_right.insert(node_nr, right.clone())?;

            if token_index_raw != "NULL" {
                let span = if has_segmentations {
                    get_field_str(&line, 12).ok_or(anyhow!("Missing column"))?
                } else {
                    get_field_str(&line, 9).ok_or(anyhow!("Missing column"))?
                };

                updates.add_event(UpdateEvent::AddNodeLabel {
                    node_name: node_path,
                    anno_ns: ANNIS_NS.to_owned(),
                    anno_name: TOK.to_owned(),
                    anno_value: span,
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
                textpos_table.token_by_left_textpos.insert(left, node_nr)?;
                textpos_table
                    .token_by_right_textpos
                    .insert(right, node_nr)?;
            } else if has_segmentations {
                let segmentation_name = if is_annis_33 {
                    get_field_str(&line, 11).ok_or(anyhow!("Missing column"))?
                } else {
                    get_field_str(&line, 8).ok_or(anyhow!("Missing column"))?
                };

                if segmentation_name != "NULL" {
                    let seg_index = if is_annis_33 {
                        line.get(10)
                            .ok_or(anyhow!("Missing column"))?
                            .parse::<u32>()?
                    } else {
                        line.get(9)
                            .ok_or(anyhow!("Missing column"))?
                            .parse::<u32>()?
                    };

                    if is_annis_33 {
                        // directly add the span information
                        updates.add_event(UpdateEvent::AddNodeLabel {
                            node_name: node_path,
                            anno_ns: ANNIS_NS.to_owned(),
                            anno_name: TOK.to_owned(),
                            anno_value: get_field_str(&line, 12)
                                .ok_or(anyhow!("Missing column"))?,
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
    path: &PathBuf,
    updates: &mut GraphUpdate,
    missing_seg_span: &DiskMap<NodeID, String>,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
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

        let col_id = line.get(0).ok_or(anyhow!("Missing column"))?;
        let node_id: NodeID = col_id.parse()?;
        let node_name = id_to_node_name
            .try_get(&node_id)?
            .ok_or(anyhow!("Missing node name"))?;
        let col_ns = get_field_str(&line, 1).ok_or(anyhow!("Missing column"))?;
        let col_name = get_field_str(&line, 2).ok_or(anyhow!("Missing column"))?;
        let col_val = get_field_str(&line, 3).ok_or(anyhow!("Missing column"))?;
        // we have to make some sanity checks
        if col_ns != "annis" || col_name != "tok" {
            let anno_val: String = if col_val == "NULL" {
                // use an "invalid" string so it can't be found by its value, but only by its annotation name
                std::char::MAX.to_string()
            } else {
                col_val
            };

            let col_ns = if col_ns == "NULL" {
                String::default()
            } else {
                col_ns
            };

            updates.add_event(UpdateEvent::AddNodeLabel {
                node_name: node_name.clone(),
                anno_ns: col_ns,
                anno_name: col_name,
                anno_value: anno_val.clone(),
            })?;

            // add all missing span values from the annotation, but don't add NULL values
            if let Some(seg) = missing_seg_span.try_get(&node_id)? {
                if seg == get_field_str(&line, 2).ok_or(anyhow!("Missing column"))?
                    && get_field_str(&line, 3).ok_or(anyhow!("Missing column"))? != "NULL"
                {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: node_name.clone(),
                        anno_ns: ANNIS_NS.to_owned(),
                        anno_name: TOK.to_owned(),
                        anno_value: anno_val,
                    })?;
                }
            }
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
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<BTreeMap<u32, Component<AnnotationComponentType>>>
where
    F: Fn(&str) -> (),
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

        let cid: u32 = line.get(0).ok_or(anyhow!("Missing column"))?.parse()?;
        let col_type = get_field_str(&line, 1).ok_or(anyhow!("Missing column"))?;
        if col_type != "NULL" {
            let layer = get_field_str(&line, 2).ok_or(anyhow!("Missing column"))?;
            let name = get_field_str(&line, 3).ok_or(anyhow!("Missing column"))?;
            let name = if name == "NULL" {
                String::from("")
            } else {
                name
            };
            let ctype = component_type_from_short_name(&col_type)?;
            component_by_id.insert(cid, Component::new(ctype, layer, name));
        }
    }
    Ok(component_by_id)
}

fn load_nodes<F>(
    path: &PathBuf,
    updates: &mut GraphUpdate,
    texts: &mut DiskMap<TextKey, Text>,
    corpus_table: &ParsedCorpusTable,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadNodeResult>
where
    F: Fn(&str) -> (),
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
    path: &PathBuf,
    updates: &mut GraphUpdate,
    component_by_id: &BTreeMap<u32, Component<AnnotationComponentType>>,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<LoadRankResult>
where
    F: Fn(&str) -> (),
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
        let pre: u32 = line.get(0).ok_or(anyhow!("Missing column"))?.parse()?;
        let node_id: NodeID = line
            .get(pos_node_ref)
            .ok_or(anyhow!("Missing column"))?
            .parse()?;
        pre_to_node_id.insert(pre, node_id)?;
    }

    // second run: get the actual edges
    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    for result in rank_tab_csv.records() {
        let line = result?;

        let component_ref: u32 = line
            .get(pos_component_ref)
            .ok_or(anyhow!("Missing column"))?
            .parse()?;

        let target: NodeID = line
            .get(pos_node_ref)
            .ok_or(anyhow!("Missing column"))?
            .parse()?;

        let parent_as_str = line.get(pos_parent).ok_or(anyhow!("Missing column"))?;
        if parent_as_str == "NULL" {
            if let Some(c) = component_by_id.get(&component_ref) {
                if c.get_type() == AnnotationComponentType::Coverage {
                    load_rank_result
                        .component_for_parentless_target_node
                        .insert(target, c.clone())?;
                }
            }
        } else {
            let parent: u32 = parent_as_str.parse()?;
            if let Some(source) = pre_to_node_id.get(&parent) {
                // find the responsible edge database by the component ID
                if let Some(c) = component_by_id.get(&component_ref) {
                    updates.add_event(UpdateEvent::AddEdge {
                        source_node: id_to_node_name
                            .try_get(&source)?
                            .ok_or(anyhow!("Missing node name"))?
                            .to_owned(),
                        target_node: id_to_node_name
                            .try_get(&target)?
                            .ok_or(anyhow!("Missing node name"))?
                            .to_owned(),
                        layer: c.layer.clone(),
                        component_type: c.get_type().to_string(),
                        component_name: c.name.clone(),
                    })?;

                    let pre: u32 = line.get(0).ok_or(anyhow!("Missing column"))?.parse()?;

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
    path: &PathBuf,
    updates: &mut GraphUpdate,
    rank_result: &LoadRankResult,
    id_to_node_name: &DiskMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
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

        let pre: u32 = line.get(0).ok_or(anyhow!("Missing column"))?.parse()?;
        if let Some(c) = rank_result.components_by_pre.try_get(&pre)? {
            if let Some(e) = rank_result.edges_by_pre.try_get(&pre)? {
                let ns = get_field_str(&line, 1).ok_or(anyhow!("Missing column"))?;
                let ns = if ns == "NULL" { String::default() } else { ns };
                let name = get_field_str(&line, 2).ok_or(anyhow!("Missing column"))?;
                let val = get_field_str(&line, 3).ok_or(anyhow!("Missing column"))?;

                updates.add_event(UpdateEvent::AddEdgeLabel {
                    source_node: id_to_node_name
                        .try_get(&e.source)?
                        .ok_or(anyhow!("Missing node name"))?
                        .to_owned(),
                    target_node: id_to_node_name
                        .try_get(&e.target)?
                        .ok_or(anyhow!("Missing node name"))?
                        .to_owned(),
                    layer: c.layer.clone(),
                    component_type: c.get_type().to_string(),
                    component_name: c.name.clone(),
                    anno_ns: ns,
                    anno_name: name,
                    anno_value: val,
                })?;
            }
        }
    }

    Ok(())
}

fn load_corpus_annotation<F>(
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<BTreeMap<(u32, AnnoKey), String>>
where
    F: Fn(&str) -> (),
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

        let id = line.get(0).ok_or(anyhow!("Missing column"))?.parse()?;
        let ns = get_field_str(&line, 1).ok_or(anyhow!("Missing column"))?;
        let ns = if ns == "NULL" { String::default() } else { ns };
        let name = get_field_str(&line, 2).ok_or(anyhow!("Missing column"))?;
        let val = get_field_str(&line, 3).ok_or(anyhow!("Missing column"))?;

        let anno_key = AnnoKey { ns, name };

        corpus_id_to_anno.insert((id, anno_key), val);
    }

    Ok(corpus_id_to_anno)
}

fn get_parent_path(cid: u32, corpus_table: &ParsedCorpusTable) -> Result<String> {
    let corpus = corpus_table
        .corpus_by_id
        .get(&cid)
        .ok_or_else(|| anyhow!("Corpus with ID {} not found", cid))?;
    let pre = corpus.pre;
    let post = corpus.post;

    let parent_corpus_path: Vec<String> = corpus_table
        .corpus_by_preorder
        .range(0..pre)
        .filter_map(|(_, cid)| corpus_table.corpus_by_id.get(cid))
        .filter(|parent_corpus| post < parent_corpus.post)
        .map(|parent_corpus| {
            utf8_percent_encode(parent_corpus.name.as_ref(), SALT_URI_ENCODE_SET).to_string()
        })
        .collect();
    Ok(parent_corpus_path.join("/"))
}

fn get_corpus_path(cid: u32, corpus_table: &ParsedCorpusTable) -> Result<String> {
    let parent_path = get_parent_path(cid, corpus_table)?;
    let corpus = corpus_table
        .corpus_by_id
        .get(&cid)
        .ok_or_else(|| anyhow!("Corpus with ID {} not found", cid))?;
    let corpus_name = utf8_percent_encode(&corpus.name, SALT_URI_ENCODE_SET).to_string();
    Ok(format!("{}/{}", parent_path, &corpus_name))
}

fn add_subcorpora(
    updates: &mut GraphUpdate,
    corpus_table: &ParsedCorpusTable,
    node_node_result: &LoadNodeResult,
    texts: &DiskMap<TextKey, Text>,
    corpus_id_to_annos: &BTreeMap<(u32, AnnoKey), String>,
    is_annis_33: bool,
    path: &Path,
) -> Result<()> {
    // add the toplevel corpus as node
    {
        updates.add_event(UpdateEvent::AddNode {
            node_name: corpus_table.toplevel_corpus_name.to_owned(),
            node_type: "corpus".to_owned(),
        })?;

        // save the relANNIS version as meta data attribute on the toplevel corpus
        updates.add_event(UpdateEvent::AddNodeLabel {
            node_name: corpus_table.toplevel_corpus_name.to_owned(),
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
                    ns: "".to_string(),
                    name: "".to_string(),
                },
            );
            for ((entry_cid, anno_key), val) in corpus_id_to_annos.range(start_key..) {
                if entry_cid == cid {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: corpus_table.toplevel_corpus_name.to_owned(),
                        anno_ns: anno_key.ns.clone(),
                        anno_name: anno_key.name.clone(),
                        anno_value: val.clone(),
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
                .ok_or_else(|| anyhow!("Can't get name for corpus with ID {}", corpus_id))?;

            let corpus_name = &corpus.name;

            let subcorpus_full_name = get_corpus_path(*corpus_id, corpus_table)?;

            // add a basic node labels for the new (sub-) corpus/document
            updates.add_event(UpdateEvent::AddNode {
                node_name: subcorpus_full_name.clone(),
                node_type: "corpus".to_owned(),
            })?;
            updates.add_event(UpdateEvent::AddNodeLabel {
                node_name: subcorpus_full_name.clone(),
                anno_ns: ANNIS_NS.to_owned(),
                anno_name: "doc".to_owned(),
                anno_value: corpus_name.to_owned(),
            })?;

            // add all metadata for the document node
            let start_key = (
                *corpus_id,
                AnnoKey {
                    ns: "".to_string(),
                    name: "".to_string(),
                },
            );
            for ((entry_cid, anno_key), val) in corpus_id_to_annos.range(start_key..) {
                if entry_cid == corpus_id {
                    updates.add_event(UpdateEvent::AddNodeLabel {
                        node_name: subcorpus_full_name.clone(),
                        anno_ns: anno_key.ns.clone(),
                        anno_name: anno_key.name.clone(),
                        anno_value: val.clone(),
                    })?;
                } else {
                    break;
                }
            }
            // add an edge from the document (or sub-corpus) to the top-level corpus
            updates.add_event(UpdateEvent::AddEdge {
                source_node: subcorpus_full_name.clone(),
                target_node: corpus_table.toplevel_corpus_name.to_owned(),
                layer: ANNIS_NS.to_owned(),
                component_type: AnnotationComponentType::PartOf.to_string(),
                component_name: String::default(),
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
                target_node: subcorpus_full_name,
                layer: ANNIS_NS.to_owned(),
                component_type: AnnotationComponentType::PartOf.to_string(),
                component_name: String::default(),
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
                        .ok_or(anyhow!("Missing node name"))?
                        .clone(),
                    target_node: text_full_name.clone(),
                    layer: ANNIS_NS.to_owned(),
                    component_type: AnnotationComponentType::PartOf.to_string(),
                    component_name: String::default(),
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
        _ => Err(anyhow!(
            "Invalid component type short name '{}'",
            short_type
        )),
    }
}
