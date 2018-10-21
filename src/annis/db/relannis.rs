use annis::db::graphstorage::WriteableGraphStorage;
use annis::db::{Graph, ANNIS_NS};
use annis::errors::*;
use annis::types::{AnnoKey, Annotation, Component, ComponentType, Edge, NodeID};
use csv;
use multimap::MultiMap;
use std;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

        let mut db = Graph::new();

        let (corpus_name, corpus_by_preorder, corpus_id_to_name) =
            parse_corpus_tab(&path, is_annis_33, &progress_callback)?;

        let texts = parse_text_tab(&path, is_annis_33, &progress_callback)?;

        let nodes_by_text = load_nodes(
            &path,
            &mut db,
            &corpus_id_to_name,
            &corpus_name,
            is_annis_33,
            &progress_callback,
        )?;

        let component_by_id = load_component_tab(&path, &mut db, is_annis_33, &progress_callback)?;

        let (pre_to_component, pre_to_edge) = load_rank_tab(
            &path,
            &mut db,
            &component_by_id,
            is_annis_33,
            &progress_callback,
        )?;
        load_edge_annotation(
            &path,
            &mut db,
            &pre_to_component,
            &pre_to_edge,
            is_annis_33,
            &progress_callback,
        )?;

        let corpus_id_to_annos = load_corpus_annotation(&path, is_annis_33, &progress_callback)?;

        add_subcorpora(
            &mut db,
            &corpus_name,
            &corpus_by_preorder,
            &corpus_id_to_name,
            &nodes_by_text,
            &texts,
            &corpus_id_to_annos,
            is_annis_33,
        )?;

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

        return Ok((corpus_name, db));
    }

    Err(format!("Directory {} not found", path.to_string_lossy()).into())
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
) -> Result<(String, BTreeMap<u32, u32>, BTreeMap<u32, String>)>
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

    let mut toplevel_corpus_name: Option<String> = None;
    let mut corpus_by_preorder = BTreeMap::new();
    let mut corpus_id_to_name = BTreeMap::new();

    let mut corpus_tab_csv = postgresql_import_reader(corpus_tab_path.as_path())?;

    for result in corpus_tab_csv.records() {
        let line = result?;

        let id = line.get(0).ok_or("Missing column")?.parse::<u32>()?;
        let name = get_field_str(&line, 1).ok_or("Missing column")?;
        let type_str = get_field_str(&line, 2).ok_or("Missing column")?;
        let pre_order = line.get(4).ok_or("Missing column")?.parse::<u32>()?;

        corpus_id_to_name.insert(id, name.clone());
        if type_str == "CORPUS" && pre_order == 0 {
            toplevel_corpus_name = Some(name);
            corpus_by_preorder.insert(pre_order, id);
        } else if type_str == "DOCUMENT" {
            // TODO: do not only add documents but also sub-corpora
            corpus_by_preorder.insert(pre_order, id);
        }
    }

    let toplevel_corpus_name = toplevel_corpus_name.ok_or("Toplevel corpus name not found")?;
    Ok((toplevel_corpus_name, corpus_by_preorder, corpus_id_to_name))
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

fn calculate_automatic_token_info<F>(
    db: &mut Graph,
    token_by_index: &BTreeMap<TextProperty, NodeID>,
    node_to_left: &BTreeMap<NodeID, u32>,
    node_to_right: &BTreeMap<NodeID, u32>,
    left_to_node: &MultiMap<TextProperty, NodeID>,
    right_to_node: &MultiMap<TextProperty, NodeID>,
    progress_callback: F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    // TODO: cleanup, better variable naming
    // iterate over all token by their order, find the nodes with the same
    // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges

    progress_callback(
        "calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges",
    );

    let mut last_textprop: Option<TextProperty> = None;
    let mut last_token: Option<NodeID> = None;

    let component_left = Component {
        ctype: ComponentType::LeftToken,
        layer: String::from("annis"),
        name: String::from(""),
    };
    let component_right = Component {
        ctype: ComponentType::RightToken,
        layer: String::from("annis"),
        name: String::from(""),
    };

    for (current_textprop, current_token) in token_by_index {
        if current_textprop.segmentation == "" {
            // find all nodes that start together with the current token
            let current_token_left = TextProperty {
                segmentation: String::from(""),
                text_id: current_textprop.text_id,
                corpus_id: current_textprop.corpus_id,
                val: try!(node_to_left.get(&current_token).ok_or_else(|| format!(
                    "Can't find node that starts together with token {}",
                    current_token
                ))).clone(),
            };
            let left_aligned = left_to_node.get_vec(&current_token_left);
            if left_aligned.is_some() {
                let gs_left = db.get_or_create_writable(component_left.clone())?;

                for n in left_aligned.unwrap() {
                    gs_left.add_edge(Edge {
                        source: n.clone(),
                        target: current_token.clone(),
                    });
                    gs_left.add_edge(Edge {
                        source: current_token.clone(),
                        target: n.clone(),
                    });
                }
            }
            // find all nodes that end together with the current token
            let current_token_right = TextProperty {
                segmentation: String::from(""),
                text_id: current_textprop.text_id,
                corpus_id: current_textprop.corpus_id,
                val: try!(node_to_right.get(current_token).ok_or_else(|| format!(
                    "Can't find node that has the same end as token {}",
                    current_token
                ))).clone(),
            };
            let right_aligned = right_to_node.get_vec(&current_token_right);
            if right_aligned.is_some() {
                let gs_right = db.get_or_create_writable(component_right.clone())?;
                for n in right_aligned.unwrap() {
                    gs_right.add_edge(Edge {
                        source: n.clone(),
                        target: current_token.clone(),
                    });
                    gs_right.add_edge(Edge {
                        source: current_token.clone(),
                        target: n.clone(),
                    });
                }
            }
        } // end if current segmentation is default

        let component_order = Component {
            ctype: ComponentType::Ordering,
            layer: String::from("annis"),
            name: current_textprop.segmentation.clone(),
        };

        let gs_order = db.get_or_create_writable(component_order.clone())?;

        // if the last token/text value is valid and we are still in the same text
        if last_token.is_some() && last_textprop.is_some() {
            let last = last_textprop.clone().unwrap();
            if last.corpus_id == current_textprop.corpus_id
                && last.text_id == current_textprop.text_id
                && last.segmentation == current_textprop.segmentation
            {
                // we are still in the same text, add ordering between token
                gs_order.add_edge(Edge {
                    source: last_token.unwrap(),
                    target: current_token.clone(),
                });
            }
        } // end if same text

        // update the iterator and other variables
        last_textprop = Some(current_textprop.clone());
        last_token = Some(current_token.clone());
    } // end for each token

    Ok(())
}

fn calculate_automatic_coverage_edges<F>(
    db: &mut Graph,
    token_by_index: &BTreeMap<TextProperty, NodeID>,
    token_to_index: &BTreeMap<NodeID, TextProperty>,
    node_to_right: &BTreeMap<NodeID, u32>,
    left_to_node: &MultiMap<TextProperty, NodeID>,
    token_by_left_textpos: &BTreeMap<TextProperty, NodeID>,
    token_by_right_textpos: &BTreeMap<TextProperty, NodeID>,
    progress_callback: &F,
) -> Result<()>
where
    F: Fn(&str) -> (),
{
    // add explicit coverage edges for each node in the special annis namespace coverage component
    let component_coverage = Component {
        ctype: ComponentType::Coverage,
        layer: String::from("annis"),
        name: String::from(""),
    };
    let component_inv_cov = Component {
        ctype: ComponentType::InverseCoverage,
        layer: String::from("annis"),
        name: String::from(""),
    };

    // make sure the components exists, even if they are empty
    db.get_or_create_writable(component_coverage.clone())?;
    db.get_or_create_writable(component_inv_cov.clone())?;

    {
        progress_callback("calculating the automatically generated COVERAGE edges");
        for (textprop, n_vec) in left_to_node {
            for n in n_vec {
                if !token_to_index.contains_key(&n) {
                    let left_pos = TextProperty {
                        segmentation: String::from(""),
                        corpus_id: textprop.corpus_id,
                        text_id: textprop.text_id,
                        val: textprop.val,
                    };
                    let right_pos = node_to_right
                        .get(&n)
                        .ok_or_else(|| format!("Can't get right position of node {}", n))?;
                    let right_pos = TextProperty {
                        segmentation: String::from(""),
                        corpus_id: textprop.corpus_id,
                        text_id: textprop.text_id,
                        val: right_pos.clone(),
                    };

                    // find left/right aligned basic token
                    let left_aligned_tok =
                        token_by_left_textpos.get(&left_pos).ok_or_else(|| {
                            format!("Can't get left-aligned token for node {:?}", left_pos)
                        })?;
                    let right_aligned_tok =
                        token_by_right_textpos.get(&right_pos).ok_or_else(|| {
                            format!("Can't get right-aligned token for node {:?}", right_pos)
                        })?;

                    let left_tok_pos = token_to_index.get(&left_aligned_tok).ok_or_else(|| {
                        format!(
                            "Can't get position of left-aligned token {}",
                            left_aligned_tok
                        )
                    })?;
                    let right_tok_pos =
                        token_to_index.get(&right_aligned_tok).ok_or_else(|| {
                            format!(
                                "Can't get position of right-aligned token {}",
                                right_aligned_tok
                            )
                        })?;
                    for i in left_tok_pos.val..(right_tok_pos.val + 1) {
                        let tok_idx = TextProperty {
                            segmentation: String::from(""),
                            corpus_id: textprop.corpus_id,
                            text_id: textprop.text_id,
                            val: i,
                        };
                        let tok_id = token_by_index.get(&tok_idx).ok_or_else(|| {
                            format!("Can't get token ID for position {:?}", tok_idx)
                        })?;
                        if n.clone() != tok_id.clone() {
                            {
                                let gs = db.get_or_create_writable(component_coverage.clone())?;
                                gs.add_edge(Edge {
                                    source: n.clone(),
                                    target: tok_id.clone(),
                                });
                            }
                            {
                                let gs = db.get_or_create_writable(component_inv_cov.clone())?;
                                gs.add_edge(Edge {
                                    source: tok_id.clone(),
                                    target: n.clone(),
                                });
                            }
                        }
                    }
                } // end if not a token
            }
        }
    }

    Ok(())
}

fn load_node_tab<F>(
    path: &PathBuf,
    db: &mut Graph,
    corpus_id_to_name: &BTreeMap<u32, String>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(MultiMap<TextKey, NodeID>, BTreeMap<NodeID, String>)>
where
    F: Fn(&str) -> (),
{
    let mut nodes_by_text: MultiMap<TextKey, NodeID> = MultiMap::new();
    let mut missing_seg_span: BTreeMap<NodeID, String> = BTreeMap::new();

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

    // maps a token index to an node ID
    let mut token_by_index: BTreeMap<TextProperty, NodeID> = BTreeMap::new();

    // map the "left" value to the nodes it belongs to
    let mut left_to_node: MultiMap<TextProperty, NodeID> = MultiMap::new();
    // map the "right" value to the nodes it belongs to
    let mut right_to_node: MultiMap<TextProperty, NodeID> = MultiMap::new();

    // map as node to it's "left" value
    let mut node_to_left: BTreeMap<NodeID, u32> = BTreeMap::new();
    // map as node to it's "right" value
    let mut node_to_right: BTreeMap<NodeID, u32> = BTreeMap::new();

    // maps a character position to it's token
    let mut token_by_left_textpos: BTreeMap<TextProperty, NodeID> = BTreeMap::new();
    let mut token_by_right_textpos: BTreeMap<TextProperty, NodeID> = BTreeMap::new();

    // maps a token node id to the token index
    let mut token_to_index: BTreeMap<NodeID, TextProperty> = BTreeMap::new();

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
                    corpus_ref: Some(corpus_id.clone()),
                    id: text_id.clone(),
                },
                node_nr.clone(),
            );

            let doc_name = corpus_id_to_name
                .get(&corpus_id)
                .ok_or_else(|| format!("Document with ID {} missing", corpus_id))?;

            let node_qname = format!("{}/{}#{}", toplevel_corpus_name, doc_name, node_name);
            let node_name_anno = Annotation {
                key: db.get_node_name_key(),
                val: node_qname,
            };
            Arc::make_mut(&mut db.node_annos).insert(node_nr, node_name_anno);

            let node_type_anno = Annotation {
                key: db.get_node_type_key(),
                val: "node".to_owned(),
            };
            Arc::make_mut(&mut db.node_annos).insert(node_nr, node_type_anno);

            if !layer.is_empty() && layer != "NULL" {
                let layer_anno = Annotation {
                    key: AnnoKey {
                        ns: "annis".to_owned(),
                        name: "layer".to_owned(),
                    },
                    val: layer,
                };
                Arc::make_mut(&mut db.node_annos).insert(node_nr, layer_anno);
            }

            let left_val = line.get(5).ok_or("Missing column")?.parse::<u32>()?;
            let left = TextProperty {
                segmentation: String::from(""),
                val: left_val,
                corpus_id,
                text_id,
            };
            let right_val = line.get(6).ok_or("Missing column")?.parse::<u32>()?;
            let right = TextProperty {
                segmentation: String::from(""),
                val: right_val,
                corpus_id,
                text_id,
            };
            left_to_node.insert(left.clone(), node_nr);
            right_to_node.insert(right.clone(), node_nr);
            node_to_left.insert(node_nr, left_val);
            node_to_right.insert(node_nr, right_val);

            if token_index_raw != "NULL" {
                let span = if has_segmentations {
                    get_field_str(&line, 12).ok_or("Missing column")?
                } else {
                    get_field_str(&line, 9).ok_or("Missing column")?
                };

                let tok_anno = Annotation {
                    key: db.get_token_key(),
                    val: span,
                };
                Arc::make_mut(&mut db.node_annos).insert(node_nr, tok_anno);

                let index = TextProperty {
                    segmentation: String::from(""),
                    val: token_index_raw.parse::<u32>()?,
                    text_id,
                    corpus_id,
                };
                token_by_index.insert(index.clone(), node_nr);
                token_to_index.insert(node_nr, index);
                token_by_left_textpos.insert(left, node_nr);
                token_by_right_textpos.insert(right, node_nr);
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
                        let tok_anno = Annotation {
                            key: db.get_token_key(),
                            val: get_field_str(&line, 12).ok_or("Missing column")?,
                        };
                        Arc::make_mut(&mut db.node_annos).insert(node_nr, tok_anno);
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
                    token_by_index.insert(index, node_nr);
                } // end if node has segmentation info
            } // endif if check segmentations
        }
    } // end "scan all lines" visibility block

    if !token_by_index.is_empty() {
        calculate_automatic_token_info(
            db,
            &token_by_index,
            &node_to_left,
            &node_to_right,
            &left_to_node,
            &right_to_node,
            progress_callback,
        )?;
    } // end if token_by_index not empty

    calculate_automatic_coverage_edges(
        db,
        &token_by_index,
        &token_to_index,
        &node_to_right,
        &left_to_node,
        &token_by_left_textpos,
        &token_by_right_textpos,
        progress_callback,
    )?;
    Ok((nodes_by_text, missing_seg_span))
}

fn load_node_anno_tab<F>(
    path: &PathBuf,
    db: &mut Graph,
    missing_seg_span: &BTreeMap<NodeID, String>,
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

    for result in node_anno_tab_csv.records() {
        let line = result?;

        let col_id = line.get(0).ok_or("Missing column")?;
        let node_id: NodeID = col_id.parse()?;
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

            Arc::make_mut(&mut db.node_annos).insert(
                node_id.clone(),
                Annotation {
                    key: AnnoKey {
                        ns: col_ns,
                        name: col_name,
                    },
                    val: anno_val.clone(),
                },
            );

            // add all missing span values from the annotation, but don't add NULL values
            if let Some(seg) = missing_seg_span.get(&node_id) {
                if seg == &get_field_str(&line, 2).ok_or("Missing column")?
                    && get_field_str(&line, 3).ok_or("Missing column")? != "NULL"
                {
                    let tok_key = db.get_token_key();
                    Arc::make_mut(&mut db.node_annos).insert(
                        node_id.clone(),
                        Annotation {
                            key: tok_key,
                            val: anno_val,
                        },
                    );
                }
            }
        }
    }

    Ok(())
}

fn load_component_tab<F>(
    path: &PathBuf,
    db: &mut Graph,
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
            let mut name = get_field_str(&line, 3).ok_or("Missing column")?;
            let name = if name == "NULL" {
                String::from("")
            } else {
                name
            };
            let ctype = component_type_from_short_name(&col_type)?;
            let c = Component { ctype, layer, name };
            db.get_or_create_writable(c.clone())?;
            component_by_id.insert(cid, c);
        }
    }
    Ok(component_by_id)
}

fn load_nodes<F>(
    path: &PathBuf,
    db: &mut Graph,
    corpus_id_to_name: &BTreeMap<u32, String>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<MultiMap<TextKey, NodeID>>
where
    F: Fn(&str) -> (),
{
    let (nodes_by_text, missing_seg_span) = load_node_tab(
        path,
        db,
        corpus_id_to_name,
        toplevel_corpus_name,
        is_annis_33,
        progress_callback,
    )?;
    load_node_anno_tab(path, db, &missing_seg_span, is_annis_33, progress_callback)?;

    Ok(nodes_by_text)
}

fn load_rank_tab<F>(
    path: &PathBuf,
    db: &mut Graph,
    component_by_id: &BTreeMap<u32, Component>,
    is_annis_33: bool,
    progress_callback: &F,
) -> Result<(BTreeMap<u32, Component>, BTreeMap<u32, Edge>)>
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

                    let gs = db.get_or_create_writable(c.clone())?;
                    let e = Edge {
                        source: source.clone(),
                        target,
                    };
                    gs.add_edge(e.clone());

                    let pre: u32 = line.get(0).ok_or("Missing column")?.parse()?;
                    pre_to_edge.insert(pre.clone(), e);
                    pre_to_component.insert(pre, c.clone());
                }
            }
        }
    }

    Ok((pre_to_component, pre_to_edge))
}

fn load_edge_annotation<F>(
    path: &PathBuf,
    db: &mut Graph,
    pre_to_component: &BTreeMap<u32, Component>,
    pre_to_edge: &BTreeMap<u32, Edge>,
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

        let pre: u32 = line.get(0).ok_or("Missing column")?.parse()?;
        if let Some(c) = pre_to_component.get(&pre) {
            if let Some(e) = pre_to_edge.get(&pre) {
                let ns = get_field_str(&line, 1).ok_or("Missing column")?;
                let name = get_field_str(&line, 2).ok_or("Missing column")?;
                let val = get_field_str(&line, 3).ok_or("Missing column")?;

                let anno = Annotation {
                    key: AnnoKey { ns: ns, name: name },
                    val: val,
                };
                let gs: &mut WriteableGraphStorage = db.get_or_create_writable(c.clone())?;
                gs.add_edge_annotation(e.clone(), anno);
            }
        }
    }

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
        let ns = if ns == "NULL" { "".to_owned() } else { ns };
        let name = get_field_str(&line, 2).ok_or("Missing column")?;
        let val = get_field_str(&line, 3).ok_or("Missing column")?;

        let anno = Annotation {
            key: AnnoKey { ns, name: name },
            val: val,
        };

        corpus_id_to_anno.insert(id, anno);
    }

    Ok(corpus_id_to_anno)
}

fn add_subcorpora(
    db: &mut Graph,
    toplevel_corpus_name: &str,
    corpus_by_preorder: &BTreeMap<u32, u32>,
    corpus_id_to_name: &BTreeMap<u32, String>,
    nodes_by_text: &MultiMap<TextKey, NodeID>,
    texts: &HashMap<TextKey, Text>,
    corpus_id_to_annos: &MultiMap<u32, Annotation>,
    is_annis_33: bool,
) -> Result<()> {
    let component_subcorpus = Component {
        ctype: ComponentType::PartOfSubcorpus,
        layer: String::from("annis"),
        name: String::from(""),
    };

    let mut next_node_id: NodeID = if let Some(id) = db.node_annos.get_largest_item() {
        id + 1
    } else {
        0
    };

    // add the toplevel corpus as node
    {
        let top_anno = Annotation {
            key: db.get_node_name_key(),
            val: toplevel_corpus_name.to_owned(),
        };
        Arc::make_mut(&mut db.node_annos).insert(next_node_id, top_anno);
        let anno_type = Annotation {
            key: db.get_node_type_key(),
            val: "corpus".to_owned(),
        };
        Arc::make_mut(&mut db.node_annos).insert(next_node_id, anno_type);
        // add all metadata for the top-level corpus node
        if let Some(cid) = corpus_by_preorder.get(&0) {
            if let Some(anno_vec) = corpus_id_to_annos.get_vec(cid) {
                for anno in anno_vec {
                    Arc::make_mut(&mut db.node_annos).insert(next_node_id, anno.clone());
                }
            }
        }
    }
    let toplevel_node_id = next_node_id;
    next_node_id += 1;

    let mut corpus_id_2_nid: HashMap<u32, NodeID> = HashMap::default();

    // add all subcorpora/documents (start with the largest pre-order)
    for (pre, corpus_id) in corpus_by_preorder.iter().rev() {
        let corpus_node_id = next_node_id;
        next_node_id += 1;

        corpus_id_2_nid.insert(corpus_id.clone(), corpus_node_id.clone());

        if pre.clone() != 0 {
            let corpus_name = corpus_id_to_name
                .get(corpus_id)
                .ok_or_else(|| format!("Can't get name for corpus with ID {}", corpus_id))?;
            let full_name = format!("{}/{}", toplevel_corpus_name, corpus_name);

            // add a basic node labels for the new (sub-) corpus/document
            let anno_name = Annotation {
                key: db.get_node_name_key(),
                val: full_name,
            };
            Arc::make_mut(&mut db.node_annos).insert(corpus_node_id.clone(), anno_name);

            let anno_doc = Annotation {
                key: AnnoKey {
                    ns: ANNIS_NS.to_owned(),
                    name: "doc".to_owned(),
                },
                val: corpus_name.to_owned(),
            };
            Arc::make_mut(&mut db.node_annos).insert(corpus_node_id.clone(), anno_doc);

            let anno_type = Annotation {
                key: db.get_node_type_key(),
                val: "corpus".to_owned(),
            };
            Arc::make_mut(&mut db.node_annos).insert(corpus_node_id.clone(), anno_type);

            // add all metadata for the document node
            if let Some(anno_vec) = corpus_id_to_annos.get_vec(&corpus_id) {
                for anno in anno_vec {
                    Arc::make_mut(&mut db.node_annos).insert(corpus_node_id.clone(), anno.clone());
                }
            }
            // add an edge from the document (or sub-corpus) to the top-level corpus
            {
                let gs = db.get_or_create_writable(component_subcorpus.clone())?;
                gs.add_edge(Edge {
                    source: corpus_node_id,
                    target: toplevel_node_id,
                });
            }
        } // end if not toplevel corpus
    } // end for each document/sub-corpus

    // add a node for each text and the connection between all sub-nodes of the text
    for text_key in nodes_by_text.keys() {
        let text_node_id = next_node_id;
        next_node_id += 1;

        // add text node (including its namee)
        let anno_type = Annotation {
            key: db.get_node_type_key(),
            val: "datasource".to_owned(),
        };
        Arc::make_mut(&mut db.node_annos).insert(text_node_id.clone(), anno_type);
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
            let corpus_name = corpus_id_to_name
                .get(&corpus_ref)
                .ok_or_else(|| format!("Can't get name for corpus with ID {}", corpus_ref))?;
            let full_name = format!("{}/{}#{}", toplevel_corpus_name, corpus_name, text_name);
            let anno_name = Annotation {
                key: db.get_node_name_key(),
                val: full_name,
            };
            Arc::make_mut(&mut db.node_annos).insert(text_node_id.clone(), anno_name);
        }

        // add an edge from the text to the document
        if let Some(corpus_ref) = text_key.corpus_ref {
            let gs = db.get_or_create_writable(component_subcorpus.clone())?;

            if let Some(corpus_node_id) = corpus_id_2_nid.get(&corpus_ref) {
                gs.add_edge(Edge {
                    source: text_node_id,
                    target: corpus_node_id.clone(),
                });
            }
        }

        // find all nodes belonging to this text and add a relation
        if let Some(n_vec) = nodes_by_text.get_vec(text_key) {
            let gs = db.get_or_create_writable(component_subcorpus.clone())?;

            for n in n_vec {
                gs.add_edge(Edge {
                    source: n.clone(),
                    target: text_node_id.clone(),
                });
            }
        }
    } // end for each text

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
