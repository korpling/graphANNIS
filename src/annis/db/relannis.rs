use crate::annis::db::{Graph, ANNIS_NS, TOK};
use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, Annotation, Component, ComponentType, Edge, NodeID};
use crate::update::{GraphUpdate, UpdateEvent};
use csv;
use multimap::MultiMap;
use std;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::prelude::*;
use std::ops::Bound::Included;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustc_hash::FxHashMap;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug)]
struct TextProperty {
    segmentation: String,
    corpus_id: u32,
    text_id: u32,
    val: u32,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextKey {
    id: u32,
    corpus_ref: Option<u32>,
}

struct Text {
    name: String,
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
    token_by_left_textpos: BTreeMap<TextProperty, NodeID>,
    token_by_right_textpos: BTreeMap<TextProperty, NodeID>,
    // maps a token index to an node ID
    token_by_index: BTreeMap<TextProperty, NodeID>,
    // maps a token node id to the token index
    token_to_index: BTreeMap<NodeID, TextProperty>,
    // map as node to it's "left" value
    node_to_left: BTreeMap<NodeID, TextProperty>,
    // map as node to it's "right" value
    node_to_right: BTreeMap<NodeID, TextProperty>,
}

struct ChunkUpdater<'a> {
    g: &'a mut Graph,
    max_number_events: usize,
    update: GraphUpdate,
}

impl<'a> ChunkUpdater<'a> {
    fn new(g: &'a mut Graph, max_number_events: usize) -> ChunkUpdater<'a> {
        ChunkUpdater {
            g,
            max_number_events,
            update: GraphUpdate::new(),
        }
    }

    fn add_event<F>(
        &mut self,
        event: UpdateEvent,
        message: Option<&str>,
        progress_callback: &F,
    ) -> Result<()>
    where
        F: Fn(&str),
    {
        self.update.add_event(event);

        if self.update.len() >= self.max_number_events {
            self.commit(message, progress_callback)?;
        }

        Ok(())
    }

    fn commit<F>(&mut self, message: Option<&str>, progress_callback: &F) -> Result<()>
    where
        F: Fn(&str),
    {
        if let Some(message) = message {
            progress_callback(&format!("{} ({} updates)", message, self.update.len()));
        } else {
            progress_callback(&format!("committing {} updates", self.update.len()));
        }
        self.g.apply_update(&mut self.update)?;
        self.update = GraphUpdate::new();
        Ok(())
    }
}

/// Load a c corpus in the legacy relANNIS format from the specified `path`.
///
/// Returns a tuple consisting of the corpus name and the extracted annotation graph.
pub fn load<F>(path: &Path, progress_callback: F) -> Result<(String, Graph)>
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

        let mut db = Graph::with_default_graphstorages()?;
        let mut updater = ChunkUpdater::new(&mut db, 1_000_000);
        let (toplevel_corpus_name, id_to_node_name, textpos_table) = {
            let (toplevel_corpus_name, id_to_node_name, textpos_table) =
                load_node_and_corpus_tables(&path, &mut updater, is_annis_33, &progress_callback)?;

            (toplevel_corpus_name, id_to_node_name, textpos_table)
        };
        {
            let text_coverage_edges = load_edge_tables(
                &path,
                &mut updater,
                is_annis_33,
                &id_to_node_name,
                &progress_callback,
            )?;

            calculate_automatic_coverage_edges(
                &mut updater,
                &textpos_table,
                &id_to_node_name,
                &text_coverage_edges,
                &progress_callback,
            )?;
        }

        progress_callback("calculating node statistics");
        Arc::make_mut(&mut db.node_annos).calculate_statistics();

        for c in db.get_all_components(None, None) {
            progress_callback(&format!("calculating statistics for component {}", c));
            db.calculate_component_statistics(&c)?;
            db.optimize_impl(&c);
        }

        progress_callback(&format!(
            "finished loading relANNIS from {}",
            path.to_string_lossy()
        ));

        return Ok((toplevel_corpus_name, db));
    }

    Err(format!("Directory {} not found", path.to_string_lossy()).into())
}

fn load_node_and_corpus_tables<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(String, FxHashMap<NodeID, String>, TextPosTable)>
where
    F: Fn(&str) -> (),
{
    let corpus_table = parse_corpus_tab(&path, is_annis_33, &progress_callback)?;
    let texts = parse_text_tab(&path, is_annis_33, &progress_callback)?;
    let corpus_id_to_annos = load_corpus_annotation(&path, is_annis_33, &progress_callback)?;

    let (nodes_by_text, id_to_node_name, textpos_table) = load_nodes(
        path,
        updater,
        &corpus_table.corpus_by_id,
        &corpus_table.toplevel_corpus_name,
        is_annis_33,
        progress_callback,
    )?;

    add_subcorpora(
        updater,
        &corpus_table,
        &nodes_by_text,
        &texts,
        &corpus_id_to_annos,
        &id_to_node_name,
        is_annis_33,
        progress_callback,
    )?;

    Ok((
        corpus_table.toplevel_corpus_name,
        id_to_node_name,
        textpos_table,
    ))
}

fn load_edge_tables<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    is_annis_33: bool,
    id_to_node_name: &FxHashMap<NodeID, String>,
    progress_callback: &F,
) -> Result<(BTreeSet<Edge>)>
where
    F: Fn(&str) -> (),
{
    let (pre_to_component, pre_to_edge, text_coverage_edges) = {
        let component_by_id = load_component_tab(path, is_annis_33, progress_callback)?;

        let (pre_to_component, pre_to_edge, text_coverage_edges) = load_rank_tab(
            path,
            updater,
            &component_by_id,
            id_to_node_name,
            is_annis_33,
            progress_callback,
        )?;

        (pre_to_component, pre_to_edge, text_coverage_edges)
    };

    load_edge_annotation(
        path,
        updater,
        &pre_to_component,
        &pre_to_edge,
        id_to_node_name,
        is_annis_33,
        progress_callback,
    )?;

    Ok(text_coverage_edges)
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

        let id = line.get(0).ok_or("Missing column")?.parse::<u32>()?;
        let name = get_field_str(&line, 1).ok_or("Missing column")?;
        let pre_order = line.get(4).ok_or("Missing column")?.parse::<u32>()?;
        let post_order = line.get(5).ok_or("Missing column")?.parse::<u32>()?;

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
        .ok_or("Toplevel corpus not found")?
        .1;
    Ok(ParsedCorpusTable {
        toplevel_corpus_name: corpus_by_id
            .get(toplevel_corpus_id)
            .ok_or("Toplevel corpus name not found")?
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
) -> Result<HashMap<TextKey, Text>>
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

    let mut texts: HashMap<TextKey, Text> = HashMap::default();

    let mut text_tab_csv = postgresql_import_reader(text_tab_path.as_path())?;

    for result in text_tab_csv.records() {
        let line = result?;

        let id = line
            .get(if is_annis_33 { 1 } else { 0 })
            .ok_or("Missing column")?
            .parse::<u32>()?;
        let name = get_field_str(&line, if is_annis_33 { 2 } else { 1 }).ok_or("Missing column")?;

        let corpus_ref = if is_annis_33 {
            Some(line.get(0).ok_or("Missing column")?.parse::<u32>()?)
        } else {
            None
        };
        let key = TextKey { id, corpus_ref };
        texts.insert(key.clone(), Text { name });
    }

    Ok(texts)
}

fn calculate_automatic_token_order<F>(
    updater: &mut ChunkUpdater,
    token_by_index: &BTreeMap<TextProperty, NodeID>,
    id_to_node_name: &FxHashMap<NodeID, String>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    // TODO: cleanup, better variable naming
    // iterate over all token by their order, find the nodes with the same
    // text coverage (either left or right) and add explicit Ordering edge

    let msg = "calculating the automatically generated Ordering edges";
    progress_callback(msg);

    let mut last_textprop: Option<TextProperty> = None;
    let mut last_token: Option<NodeID> = None;

    for (current_textprop, current_token) in token_by_index {
        // if the last token/text value is valid and we are still in the same text
        if let (Some(last_token), Some(last_textprop)) = (last_token, last_textprop) {
            if last_textprop.corpus_id == current_textprop.corpus_id
                && last_textprop.text_id == current_textprop.text_id
                && last_textprop.segmentation == current_textprop.segmentation
            {
                // we are still in the same text, add ordering between token
                updater.add_event(
                    UpdateEvent::AddEdge {
                        source_node: id_to_node_name
                            .get(&last_token)
                            .ok_or("Missing node name")?
                            .clone(),
                        target_node: id_to_node_name
                            .get(current_token)
                            .ok_or("Missing node name")?
                            .clone(),
                        layer: ANNIS_NS.to_owned(),
                        component_type: ComponentType::Ordering.to_string(),
                        component_name: current_textprop.segmentation.clone(),
                    },
                    Some(msg),
                    progress_callback,
                )?;
            }
        } // end if same text

        // update the iterator and other variables
        last_textprop = Some(current_textprop.clone());
        last_token = Some(*current_token);
    } // end for each token

    updater.commit(Some(msg), progress_callback)?;

    Ok(())
}

fn add_automatic_cov_edge_for_node<F>(
    updater: &mut ChunkUpdater,
    n: NodeID,
    left_pos: TextProperty,
    right_pos: TextProperty,
    textpos_table: &TextPosTable,
    id_to_node_name: &FxHashMap<NodeID, String>,
    text_coverage_edges: &BTreeSet<Edge>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    // find left/right aligned basic token
    let left_aligned_tok = textpos_table
        .token_by_left_textpos
        .get(&left_pos)
        .ok_or_else(|| format!("Can't get left-aligned token for node {}", n,));
    let right_aligned_tok = textpos_table
        .token_by_right_textpos
        .get(&right_pos)
        .ok_or_else(|| format!("Can't get right-aligned token for node {}", n,));

    // If only one of the aligned token is missing, use it for both sides, this is consistent with
    // the relANNIS import of ANNIS3
    let left_aligned_tok = if let Ok(left_aligned_tok) = left_aligned_tok {
        left_aligned_tok
    } else {
        right_aligned_tok.clone()?
    };
    let right_aligned_tok = if let Ok(right_aligned_tok) = right_aligned_tok {
        right_aligned_tok
    } else {
        left_aligned_tok
    };

    let left_tok_pos = textpos_table
        .token_to_index
        .get(&left_aligned_tok)
        .ok_or_else(|| {
            format!(
                "Can't get position of left-aligned token {}",
                left_aligned_tok
            )
        })?;
    let right_tok_pos = textpos_table
        .token_to_index
        .get(&right_aligned_tok)
        .ok_or_else(|| {
            format!(
                "Can't get position of right-aligned token {}",
                right_aligned_tok
            )
        })?;

    for i in left_tok_pos.val..(right_tok_pos.val + 1) {
        let tok_idx = TextProperty {
            segmentation: String::default(),
            corpus_id: left_tok_pos.corpus_id,
            text_id: left_tok_pos.text_id,
            val: i,
        };
        let tok_id = textpos_table
            .token_by_index
            .get(&tok_idx)
            .ok_or_else(|| format!("Can't get token ID for position {:?}", tok_idx))?;
        if n != *tok_id {
            let edge = Edge {
                source: n,
                target: *tok_id,
            };

            // only add edge of no other coverage edge exists
            if !text_coverage_edges.contains(&edge) {
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
                let component_name = if text_coverage_edges
                    .range(nodes_with_same_source)
                    .next()
                    .is_some()
                {
                    // this is an additional auto-generated coverage edge, mark it as such
                    "autogenerated-coverage"
                } else {
                    // the node has no other coverage edges, use a neutral component
                    ""
                };

                updater.add_event(
                    UpdateEvent::AddEdge {
                        source_node: id_to_node_name.get(&n).ok_or("Missing node name")?.clone(),
                        target_node: id_to_node_name
                            .get(tok_id)
                            .ok_or("Missing node name")?
                            .clone(),
                        layer: ANNIS_NS.to_owned(),
                        component_type: ComponentType::Coverage.to_string(),
                        component_name: component_name.to_owned(),
                    },
                    Some("calculating the automatically generated Coverage edges"),
                    progress_callback,
                )?;
            }
        }
    }

    Ok(())
}

fn calculate_automatic_coverage_edges<F>(
    updater: &mut ChunkUpdater,
    textpos_table: &TextPosTable,
    id_to_node_name: &FxHashMap<NodeID, String>,
    text_coverage_edges: &BTreeSet<Edge>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    // add explicit coverage edges for each node in the special annis namespace coverage component
    progress_callback("calculating the automatically generated Coverage edges");

    for (n, textprop) in textpos_table.node_to_left.iter() {
        if textprop.segmentation == "" {
            if !textpos_table.token_to_index.contains_key(&n) {
                let left_pos = TextProperty {
                    segmentation: String::from(""),
                    corpus_id: textprop.corpus_id,
                    text_id: textprop.text_id,
                    val: textprop.val,
                };
                let right_pos = textpos_table
                    .node_to_right
                    .get(&n)
                    .ok_or_else(|| format!("Can't get right position of node {}", n))?;
                let right_pos = TextProperty {
                    segmentation: String::from(""),
                    corpus_id: textprop.corpus_id,
                    text_id: textprop.text_id,
                    val: right_pos.val,
                };

                if let Err(e) = add_automatic_cov_edge_for_node(
                    updater,
                    *n,
                    left_pos,
                    right_pos,
                    textpos_table,
                    id_to_node_name,
                    text_coverage_edges,
                    progress_callback,
                ) {
                    // output a warning but do not fail
                    warn!(
                        "Adding coverage edges (connects spans with tokens) failed: {}",
                        e
                    )
                }
            } // end if not a token
        }
    }

    updater.commit(
        Some("calculating the automatically generated Coverage edges"),
        progress_callback,
    )?;

    Ok(())
}

fn load_node_tab<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    corpus_by_id: &BTreeMap<u32, CorpusTableEntry>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(
    MultiMap<TextKey, NodeID>,
    BTreeMap<NodeID, String>,
    FxHashMap<NodeID, String>,
    TextPosTable,
)>
where
    F: Fn(&str) -> (),
{
    let mut nodes_by_text: MultiMap<TextKey, NodeID> = MultiMap::new();
    let mut missing_seg_span: BTreeMap<NodeID, String> = BTreeMap::new();
    let mut id_to_node_name: FxHashMap<NodeID, String> = FxHashMap::default();

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

    let msg = "committing nodes";

    // map the "left" value to the nodes it belongs to
    let mut left_to_node: MultiMap<TextProperty, NodeID> = MultiMap::new();
    // map the "right" value to the nodes it belongs to
    let mut right_to_node: MultiMap<TextProperty, NodeID> = MultiMap::new();

    // maps a character position to it's token
    let mut textpos_table = TextPosTable {
        token_by_left_textpos: BTreeMap::new(),
        token_by_right_textpos: BTreeMap::new(),
        node_to_left: BTreeMap::new(),
        node_to_right: BTreeMap::new(),
        token_by_index: BTreeMap::new(),
        token_to_index: BTreeMap::new(),
    };

    // start "scan all lines" visibility block
    {
        let mut node_tab_csv = postgresql_import_reader(node_tab_path.as_path())?;

        for result in node_tab_csv.records() {
            let line = result?;

            let node_nr = line.get(0).ok_or("Missing column")?.parse::<NodeID>()?;
            let has_segmentations = is_annis_33 || line.len() > 10;
            let token_index_raw = line.get(7).ok_or("Missing column")?;
            let text_id = line.get(1).ok_or("Missing column")?.parse::<u32>()?;
            let corpus_id = line.get(2).ok_or("Missing column")?.parse::<u32>()?;
            let layer = get_field_str(&line, 3).ok_or("Missing column")?;
            let node_name = get_field_str(&line, 4).ok_or("Missing column")?;

            nodes_by_text.insert(
                TextKey {
                    corpus_ref: Some(corpus_id),
                    id: text_id,
                },
                node_nr,
            );

            let doc_name = &corpus_by_id
                .get(&corpus_id)
                .ok_or_else(|| format!("Document with ID {} missing", corpus_id))?
                .name;

            let node_qname = format!("{}/{}#{}", toplevel_corpus_name, doc_name, node_name);
            updater.add_event(
                UpdateEvent::AddNode {
                    node_name: node_qname.clone(),
                    node_type: "node".to_owned(),
                },
                Some(msg),
                progress_callback,
            )?;
            id_to_node_name.insert(node_nr, node_qname.clone());

            if !layer.is_empty() && layer != "NULL" {
                updater.add_event(
                    UpdateEvent::AddNodeLabel {
                        node_name: node_qname.clone(),
                        anno_ns: ANNIS_NS.to_owned(),
                        anno_name: "layer".to_owned(),
                        anno_value: layer,
                    },
                    Some(msg),
                    progress_callback,
                )?;
            }

            // Use left/right token columns for relANNIS 3.3 and the left/right character column otherwise.
            // For some malformed corpora, the token coverage information is more robust and guaranties that a node is
            // only left/right aligned to a single token.
            let left_column = if is_annis_33 { 8 } else { 5 };
            let right_column = if is_annis_33 { 9 } else { 6 };

            let left_val = line
                .get(left_column)
                .ok_or("Missing column")?
                .parse::<u32>()?;
            let left = TextProperty {
                segmentation: String::from(""),
                val: left_val,
                corpus_id,
                text_id,
            };
            let right_val = line
                .get(right_column)
                .ok_or("Missing column")?
                .parse::<u32>()?;
            let right = TextProperty {
                segmentation: String::from(""),
                val: right_val,
                corpus_id,
                text_id,
            };
            left_to_node.insert(left.clone(), node_nr);
            right_to_node.insert(right.clone(), node_nr);
            textpos_table.node_to_left.insert(node_nr, left.clone());
            textpos_table.node_to_right.insert(node_nr, right.clone());

            if token_index_raw != "NULL" {
                let span = if has_segmentations {
                    get_field_str(&line, 12).ok_or("Missing column")?
                } else {
                    get_field_str(&line, 9).ok_or("Missing column")?
                };

                updater.add_event(
                    UpdateEvent::AddNodeLabel {
                        node_name: node_qname,
                        anno_ns: ANNIS_NS.to_owned(),
                        anno_name: TOK.to_owned(),
                        anno_value: span,
                    },
                    Some(msg),
                    progress_callback,
                )?;

                let index = TextProperty {
                    segmentation: String::from(""),
                    val: token_index_raw.parse::<u32>()?,
                    text_id,
                    corpus_id,
                };
                textpos_table.token_by_index.insert(index.clone(), node_nr);
                textpos_table.token_to_index.insert(node_nr, index);
                textpos_table.token_by_left_textpos.insert(left, node_nr);
                textpos_table.token_by_right_textpos.insert(right, node_nr);
            } else if has_segmentations {
                let segmentation_name = if is_annis_33 {
                    get_field_str(&line, 11).ok_or("Missing column")?
                } else {
                    get_field_str(&line, 8).ok_or("Missing column")?
                };

                if segmentation_name != "NULL" {
                    let seg_index = if is_annis_33 {
                        line.get(10).ok_or("Missing column")?.parse::<u32>()?
                    } else {
                        line.get(9).ok_or("Missing column")?.parse::<u32>()?
                    };

                    if is_annis_33 {
                        // directly add the span information
                        updater.add_event(
                            UpdateEvent::AddNodeLabel {
                                node_name: node_qname,
                                anno_ns: ANNIS_NS.to_owned(),
                                anno_name: TOK.to_owned(),
                                anno_value: get_field_str(&line, 12).ok_or("Missing column")?,
                            },
                            Some(msg),
                            progress_callback,
                        )?;
                    } else {
                        // we need to get the span information from the node_annotation file later
                        missing_seg_span.insert(node_nr, segmentation_name.clone());
                    }
                    // also add the specific segmentation index
                    let index = TextProperty {
                        segmentation: segmentation_name,
                        val: seg_index,
                        corpus_id,
                        text_id,
                    };
                    textpos_table.token_by_index.insert(index, node_nr);
                } // end if node has segmentation info
            } // endif if check segmentations
        }
    } // end "scan all lines" visibility block

    updater.commit(Some(msg), progress_callback)?;

    if !textpos_table.token_by_index.is_empty() {
        calculate_automatic_token_order(
            updater,
            &textpos_table.token_by_index,
            &id_to_node_name,
            progress_callback,
        )?;
    } // end if token_by_index not empty

    Ok((
        nodes_by_text,
        missing_seg_span,
        id_to_node_name,
        textpos_table,
    ))
}

fn load_node_anno_tab<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    missing_seg_span: &BTreeMap<NodeID, String>,
    id_to_node_name: &FxHashMap<NodeID, String>,
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

    let msg = "committing node annotations";

    let mut node_anno_tab_csv = postgresql_import_reader(node_anno_tab_path.as_path())?;

    for result in node_anno_tab_csv.records() {
        let line = result?;

        let col_id = line.get(0).ok_or("Missing column")?;
        let node_id: NodeID = col_id.parse()?;
        let node_name = id_to_node_name.get(&node_id).ok_or("Missing node name")?;
        let col_ns = get_field_str(&line, 1).ok_or("Missing column")?;
        let col_name = get_field_str(&line, 2).ok_or("Missing column")?;
        let col_val = get_field_str(&line, 3).ok_or("Missing column")?;
        // we have to make some sanity checks
        if col_ns != "annis" || col_name != "tok" {
            let anno_val: String = if col_val == "NULL" {
                // use an "invalid" string so it can't be found by its value, but only by its annotation name
                std::char::MAX.to_string()
            } else {
                col_val
            };

            updater.add_event(
                UpdateEvent::AddNodeLabel {
                    node_name: node_name.clone(),
                    anno_ns: col_ns,
                    anno_name: col_name,
                    anno_value: anno_val.clone(),
                },
                Some(msg),
                progress_callback,
            )?;

            // add all missing span values from the annotation, but don't add NULL values
            if let Some(seg) = missing_seg_span.get(&node_id) {
                if seg == &get_field_str(&line, 2).ok_or("Missing column")?
                    && get_field_str(&line, 3).ok_or("Missing column")? != "NULL"
                {
                    updater.add_event(
                        UpdateEvent::AddNodeLabel {
                            node_name: node_name.clone(),
                            anno_ns: ANNIS_NS.to_owned(),
                            anno_name: TOK.to_owned(),
                            anno_value: anno_val,
                        },
                        Some(msg),
                        progress_callback,
                    )?;
                }
            }
        }
    }

    updater.commit(Some(msg), progress_callback)?;

    Ok(())
}

fn load_component_tab<F>(
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<BTreeMap<u32, Component>>
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

    let mut component_by_id: BTreeMap<u32, Component> = BTreeMap::new();

    let mut component_tab_csv = postgresql_import_reader(component_tab_path.as_path())?;
    for result in component_tab_csv.records() {
        let line = result?;

        let cid: u32 = line.get(0).ok_or("Missing column")?.parse()?;
        let col_type = get_field_str(&line, 1).ok_or("Missing column")?;
        if col_type != "NULL" {
            let layer = get_field_str(&line, 2).ok_or("Missing column")?;
            let name = get_field_str(&line, 3).ok_or("Missing column")?;
            let name = if name == "NULL" {
                String::from("")
            } else {
                name
            };
            let ctype = component_type_from_short_name(&col_type)?;
            component_by_id.insert(cid, Component { ctype, layer, name });
        }
    }
    Ok(component_by_id)
}

fn load_nodes<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    corpus_by_id: &BTreeMap<u32, CorpusTableEntry>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(
    MultiMap<TextKey, NodeID>,
    FxHashMap<NodeID, String>,
    TextPosTable,
)>
where
    F: Fn(&str) -> (),
{
    let (nodes_by_text, missing_seg_span, id_to_node_name, textpos_table) = load_node_tab(
        path,
        updater,
        corpus_by_id,
        toplevel_corpus_name,
        is_annis_33,
        progress_callback,
    )?;

    for order_component in updater
        .g
        .get_all_components(Some(ComponentType::Ordering), None)
    {
        updater.g.calculate_component_statistics(&order_component)?;
        updater.g.optimize_impl(&order_component);
    }

    load_node_anno_tab(
        path,
        updater,
        &missing_seg_span,
        &id_to_node_name,
        is_annis_33,
        progress_callback,
    )?;

    Ok((nodes_by_text, id_to_node_name, textpos_table))
}

fn load_rank_tab<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    component_by_id: &BTreeMap<u32, Component>,
    id_to_node_name: &FxHashMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(
    BTreeMap<u32, Component>,
    BTreeMap<u32, Edge>,
    BTreeSet<Edge>,
)>
where
    F: Fn(&str) -> (),
{
    let mut text_coverage_edges: BTreeSet<Edge> = BTreeSet::default();

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

    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    let pos_node_ref = if is_annis_33 { 3 } else { 2 };
    let pos_component_ref = if is_annis_33 { 4 } else { 3 };
    let pos_parent = if is_annis_33 { 5 } else { 4 };

    // first run: collect all pre-order values for a node
    let mut pre_to_node_id: BTreeMap<u32, NodeID> = BTreeMap::new();
    for result in rank_tab_csv.records() {
        let line = result?;
        let pre: u32 = line.get(0).ok_or("Missing column")?.parse()?;
        let node_id: NodeID = line.get(pos_node_ref).ok_or("Missing column")?.parse()?;
        pre_to_node_id.insert(pre, node_id);
    }

    let mut pre_to_component: BTreeMap<u32, Component> = BTreeMap::new();
    let mut pre_to_edge: BTreeMap<u32, Edge> = BTreeMap::new();
    // second run: get the actual edges
    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    let msg = "committing edges";

    for result in rank_tab_csv.records() {
        let line = result?;

        let parent_as_str = line.get(pos_parent).ok_or("Missing column")?;
        if parent_as_str != "NULL" {
            let parent: u32 = parent_as_str.parse()?;
            if let Some(source) = pre_to_node_id.get(&parent) {
                // find the responsible edge database by the component ID
                let component_ref: u32 = line
                    .get(pos_component_ref)
                    .ok_or("Missing column")?
                    .parse()?;
                if let Some(c) = component_by_id.get(&component_ref) {
                    let target: NodeID = line.get(pos_node_ref).ok_or("Missing column")?.parse()?;

                    updater.add_event(
                        UpdateEvent::AddEdge {
                            source_node: id_to_node_name
                                .get(&source)
                                .ok_or("Missing node name")?
                                .to_owned(),
                            target_node: id_to_node_name
                                .get(&target)
                                .ok_or("Missing node name")?
                                .to_owned(),
                            layer: c.layer.clone(),
                            component_type: c.ctype.to_string(),
                            component_name: c.name.clone(),
                        },
                        Some(msg),
                        progress_callback,
                    )?;

                    let pre: u32 = line.get(0).ok_or("Missing column")?.parse()?;

                    let e = Edge {
                        source: *source,
                        target,
                    };

                    if c.ctype == ComponentType::Coverage {
                        text_coverage_edges.insert(e.clone());
                    }

                    pre_to_edge.insert(pre, e);
                    pre_to_component.insert(pre, c.clone());
                }
            }
        }
    }

    updater.commit(Some(msg), progress_callback)?;

    Ok((pre_to_component, pre_to_edge, text_coverage_edges))
}

fn load_edge_annotation<F>(
    path: &PathBuf,
    updater: &mut ChunkUpdater,
    pre_to_component: &BTreeMap<u32, Component>,
    pre_to_edge: &BTreeMap<u32, Edge>,
    id_to_node_name: &FxHashMap<NodeID, String>,
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

    let msg = "committing edge annotations";

    let mut edge_anno_tab_csv = postgresql_import_reader(edge_anno_tab_path.as_path())?;

    for result in edge_anno_tab_csv.records() {
        let line = result?;

        let pre: u32 = line.get(0).ok_or("Missing column")?.parse()?;
        if let Some(c) = pre_to_component.get(&pre) {
            if let Some(e) = pre_to_edge.get(&pre) {
                let ns = get_field_str(&line, 1).ok_or("Missing column")?;
                let name = get_field_str(&line, 2).ok_or("Missing column")?;
                let val = get_field_str(&line, 3).ok_or("Missing column")?;

                updater.add_event(
                    UpdateEvent::AddEdgeLabel {
                        source_node: id_to_node_name
                            .get(&e.source)
                            .ok_or("Missing node name")?
                            .to_owned(),
                        target_node: id_to_node_name
                            .get(&e.target)
                            .ok_or("Missing node name")?
                            .to_owned(),
                        layer: c.layer.clone(),
                        component_type: c.ctype.to_string(),
                        component_name: c.name.clone(),
                        anno_ns: ns,
                        anno_name: name,
                        anno_value: val,
                    },
                    Some(msg),
                    progress_callback,
                )?;
            }
        }
    }

    updater.commit(Some(msg), progress_callback)?;

    Ok(())
}

fn load_corpus_annotation<F>(
    path: &PathBuf,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<MultiMap<u32, Annotation>>
where
    F: Fn(&str) -> (),
{
    let mut corpus_id_to_anno = MultiMap::new();

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

        let id = line.get(0).ok_or("Missing column")?.parse()?;
        let ns = get_field_str(&line, 1).ok_or("Missing column")?;
        let ns = if ns == "NULL" { String::default() } else { ns };
        let name = get_field_str(&line, 2).ok_or("Missing column")?;
        let val = get_field_str(&line, 3).ok_or("Missing column")?;

        let anno = Annotation {
            key: AnnoKey { ns, name },
            val,
        };

        corpus_id_to_anno.insert(id, anno);
    }

    Ok(corpus_id_to_anno)
}

fn add_subcorpora<F>(
    updater: &mut ChunkUpdater,
    corpus_table: &ParsedCorpusTable,
    nodes_by_text: &MultiMap<TextKey, NodeID>,
    texts: &HashMap<TextKey, Text>,
    corpus_id_to_annos: &MultiMap<u32, Annotation>,
    id_to_node_name: &FxHashMap<NodeID, String>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    let msg = "committing corpus structure";
    // add the toplevel corpus as node
    {
        updater.add_event(
            UpdateEvent::AddNode {
                node_name: corpus_table.toplevel_corpus_name.to_owned(),
                node_type: "corpus".to_owned(),
            },
            Some(msg),
            progress_callback,
        )?;

        // add all metadata for the top-level corpus node
        if let Some(cid) = corpus_table.corpus_by_preorder.get(&0) {
            if let Some(anno_vec) = corpus_id_to_annos.get_vec(cid) {
                for anno in anno_vec {
                    updater.add_event(
                        UpdateEvent::AddNodeLabel {
                            node_name: corpus_table.toplevel_corpus_name.to_owned(),
                            anno_ns: anno.key.ns.clone(),
                            anno_name: anno.key.name.clone(),
                            anno_value: anno.val.clone(),
                        },
                        Some(msg),
                        progress_callback,
                    )?;
                }
            }
        }
    }

    // add all subcorpora/documents (start with the smallest pre-order)
    for (pre, corpus_id) in corpus_table.corpus_by_preorder.iter() {
        if *pre != 0 {
            let corpus = corpus_table
                .corpus_by_id
                .get(corpus_id)
                .ok_or_else(|| format!("Can't get name for corpus with ID {}", corpus_id))?;

            let corpus_name = &corpus.name;

            let parent_corpus_path: Vec<&str> = corpus_table
                .corpus_by_preorder
                .range(0..*pre)
                .filter_map(|(_, cid)| corpus_table.corpus_by_id.get(cid))
                .filter(|parent_corpus| parent_corpus.post < corpus.post)
                .map(|parent_corpus| parent_corpus.name.as_ref())
                .collect();
            let parent_corpus_path = parent_corpus_path.join("/");

            let subcorpus_full_name = format!("{}/{}", parent_corpus_path, corpus_name);

            // add a basic node labels for the new (sub-) corpus/document
            updater.add_event(
                UpdateEvent::AddNode {
                    node_name: subcorpus_full_name.clone(),
                    node_type: "corpus".to_owned(),
                },
                Some(msg),
                progress_callback,
            )?;
            updater.add_event(
                UpdateEvent::AddNodeLabel {
                    node_name: subcorpus_full_name.clone(),
                    anno_ns: ANNIS_NS.to_owned(),
                    anno_name: "doc".to_owned(),
                    anno_value: corpus_name.to_owned(),
                },
                Some(msg),
                progress_callback,
            )?;

            // add all metadata for the document node
            if let Some(anno_vec) = corpus_id_to_annos.get_vec(&corpus_id) {
                for anno in anno_vec {
                    updater.add_event(
                        UpdateEvent::AddNodeLabel {
                            node_name: subcorpus_full_name.clone(),
                            anno_ns: anno.key.ns.clone(),
                            anno_name: anno.key.name.clone(),
                            anno_value: anno.val.clone(),
                        },
                        Some(msg),
                        progress_callback,
                    )?;
                }
            }
            // add an edge from the document (or sub-corpus) to the top-level corpus
            updater.add_event(
                UpdateEvent::AddEdge {
                    source_node: subcorpus_full_name.clone(),
                    target_node: corpus_table.toplevel_corpus_name.to_owned(),
                    layer: ANNIS_NS.to_owned(),
                    component_type: ComponentType::PartOf.to_string(),
                    component_name: String::default(),
                },
                Some(msg),
                progress_callback,
            )?;
        } // end if not toplevel corpus
    } // end for each document/sub-corpus

    // add a node for each text and the connection between all sub-nodes of the text
    for text_key in nodes_by_text.keys() {
        // add text node (including its name)
        let text_name: Option<String> = if is_annis_33 {
            // corpus_ref is included in the text.annis
            texts.get(text_key).map(|k| k.name.clone())
        } else {
            // create a text key without corpus_ref, since it is not in the parsed result
            let new_text_key = TextKey {
                id: text_key.id,
                corpus_ref: None,
            };
            texts.get(&new_text_key).map(|k| k.name.clone())
        };
        if let (Some(text_name), Some(corpus_ref)) = (text_name, text_key.corpus_ref) {
            let corpus = corpus_table
                .corpus_by_id
                .get(&corpus_ref)
                .ok_or_else(|| format!("Can't get name for corpus with ID {}", corpus_ref))?;

            let parent_corpus_path: Vec<&str> = corpus_table
                .corpus_by_preorder
                .range(0..corpus.pre)
                .filter_map(|(_, cid)| corpus_table.corpus_by_id.get(cid))
                .filter(|parent_corpus| parent_corpus.post < corpus.post)
                .map(|parent_corpus| parent_corpus.name.as_ref())
                .collect();
            let parent_corpus_path = parent_corpus_path.join("/");

            let subcorpus_full_name = format!("{}/{}", parent_corpus_path, &corpus.name);
            let text_full_name = format!("{}#{}", &subcorpus_full_name, text_name);

            updater.add_event(
                UpdateEvent::AddNode {
                    node_name: text_full_name.clone(),
                    node_type: "datasource".to_owned(),
                },
                Some(msg),
                progress_callback,
            )?;

            // add an edge from the text to the document
            updater.add_event(
                UpdateEvent::AddEdge {
                    source_node: text_full_name.clone(),
                    target_node: subcorpus_full_name,
                    layer: ANNIS_NS.to_owned(),
                    component_type: ComponentType::PartOf.to_string(),
                    component_name: String::default(),
                },
                Some(msg),
                progress_callback,
            )?;

            // find all nodes belonging to this text and add a relation
            if let Some(n_vec) = nodes_by_text.get_vec(text_key) {
                for n in n_vec {
                    updater.add_event(
                        UpdateEvent::AddEdge {
                            source_node: id_to_node_name.get(n).ok_or("Missing node name")?.clone(),
                            target_node: text_full_name.clone(),
                            layer: ANNIS_NS.to_owned(),
                            component_type: ComponentType::PartOf.to_string(),
                            component_name: String::default(),
                        },
                        Some(msg),
                        progress_callback,
                    )?;
                }
            }
        }
    } // end for each text

    updater.commit(Some(msg), progress_callback)?;

    Ok(())
}

fn component_type_from_short_name(short_type: &str) -> Result<ComponentType> {
    match short_type {
        "c" => Ok(ComponentType::Coverage),
        "d" => Ok(ComponentType::Dominance),
        "p" => Ok(ComponentType::Pointing),
        "o" => Ok(ComponentType::Ordering),
        _ => Err(format!("Invalid component type short name '{}'", short_type).into()),
    }
}
