use graphdb::GraphDB;
use annis::{AnnoKey, Annotation, NodeID, StringID, Component, ComponentType, Edge};
use annis::graphstorage::WriteableGraphStorage;
use annis;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::num::ParseIntError;
use std::collections::BTreeMap;
use multimap::MultiMap;

use std;
use csv;

pub struct RelANNISLoader;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    CSVError(csv::Error),
    GraphDBError(annis::graphdb::Error),
    MissingColumn,
    InvalidDataType,
    ToplevelCorpusNameNotFound,
    DirectoryNotFound,
    DocumentMissing,
    InvalidShortComponentType,
    Other,
}

type Result<T> = std::result::Result<T, Error>;

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Error {
        Error::InvalidDataType
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Error {
        Error::CSVError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IOError(e)
    }
}

impl From<annis::graphdb::Error> for Error {
    fn from(e: annis::graphdb::Error) -> Error {
        Error::GraphDBError(e)
    }
}


#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct TextProperty {
    segmentation: String,
    corpus_id: u32,
    text_id: u32,
    val: u32,
}

pub fn load(path: &str) -> Result<GraphDB> {
    // convert to path
    let path_str = path;
    let mut path = PathBuf::from(path);
    if path.is_dir() && path.exists() {
        // check if this is the ANNIS 3.3 import format
        path.push("annis.version");
        let mut is_annis_33 = false;
        if path.exists() {
            let mut file = File::open(&path)?;
            let mut version_str = String::new();
            file.read_to_string(&mut version_str)?;

            is_annis_33 = version_str == "3.3";
        }

        let mut db = GraphDB::new();

        let (corpus_name, corpus_by_preorder, corpus_id_to_name) = parse_corpus_tab(
            &path,
            is_annis_33,
        )?;

        let nodes_by_corpus_id = load_nodes(
            &path,
            &mut db,
            &corpus_id_to_name,
            &corpus_name,
            is_annis_33,
        )?;

        let component_by_id = load_component_tab(&path, &mut db, is_annis_33)?;

        let (pre_to_component, pre_to_edge) =
            load_rank_tab(&path, &mut db, &component_by_id, is_annis_33)?;
        load_edge_annotation(&path, &mut db, &pre_to_component, &pre_to_edge, is_annis_33)?;

        let corpus_id_to_annos = load_corpus_annotation(&path, &mut db, is_annis_33)?;

        add_subcorpora(&mut db, &corpus_name, &corpus_by_preorder, &corpus_id_to_name, &nodes_by_corpus_id, &corpus_id_to_annos)?;

        // TODO: optimize all components
        // TODO: update statistics for node annotations

        info!("finished loading relANNIS from {}", path_str);

        return Ok(db);
    }

    return Err(Error::DirectoryNotFound);
}

fn postgresql_import_reader(path: &Path) -> std::result::Result<csv::Reader<File>, csv::Error> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(path)
}

fn parse_corpus_tab(
    path: &PathBuf,
    is_annis_33: bool,
) -> Result<(String, BTreeMap<u32, u32>, BTreeMap<u32, String>)> {
    let mut corpus_tab_path = PathBuf::from(path);
    corpus_tab_path.push(if is_annis_33 {
        "corpus.annis"
    } else {
        "corpus.tab"
    });

    let mut toplevel_corpus_name: Option<String> = None;
    let mut corpus_by_preorder = BTreeMap::new();
    let mut corpus_id_to_name = BTreeMap::new();
        

    let mut corpus_tab_csv = postgresql_import_reader(corpus_tab_path.as_path())?;

    for result in corpus_tab_csv.records() {
        let line = result?;

        let id = line.get(0).ok_or(Error::MissingColumn)?.parse::<u32>()?;
        let name = line.get(1).ok_or(Error::MissingColumn)?;
        let type_str = line.get(2).ok_or(Error::MissingColumn)?;
        let pre_order = line.get(4).ok_or(Error::MissingColumn)?.parse::<u32>()?;

        corpus_id_to_name.insert(id, String::from(name));
        if type_str == "CORPUS" && pre_order == 0 {
            toplevel_corpus_name = Some(String::from(name));
            corpus_by_preorder.insert(pre_order, id);
        } else if type_str == "DOCUMENT" {
            // TODO: do not only add documents but also sub-corpora
            corpus_by_preorder.insert(pre_order, id);
        }
    }

    let toplevel_corpus_name = toplevel_corpus_name.ok_or(Error::ToplevelCorpusNameNotFound)?;
    Ok((toplevel_corpus_name, corpus_by_preorder, corpus_id_to_name))
}

fn calculate_automatic_token_info(
    db: &mut GraphDB,
    token_by_index: &BTreeMap<TextProperty, NodeID>,
    node_to_left: &BTreeMap<NodeID, u32>,
    node_to_right: &BTreeMap<NodeID, u32>,
    left_to_node: &MultiMap<TextProperty, NodeID>,
    right_to_node: &MultiMap<TextProperty, NodeID>,
) -> Result<()> {

    // TODO: cleanup, better variable naming
    // iterate over all token by their order, find the nodes with the same
    // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges

    info!("calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges");

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
                val: try!(node_to_left.get(&current_token).ok_or(Error::Other)).clone(),
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
                val: try!(node_to_right.get(current_token).ok_or(Error::Other)).clone(),
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
        if last_token.is_some() && last_textprop.is_some() &&
            last_textprop.unwrap() == current_textprop.clone()
        {
            // we are still in the same text, add ordering between token
            gs_order.add_edge(Edge {
                source: last_token.unwrap(),
                target: current_token.clone(),
            });
        } // end if same text

        // update the iterator and other variables
        last_textprop = Some(current_textprop.clone());
        last_token = Some(current_token.clone());

    } // end for each token


    Ok(())
}

fn calculate_automatic_coverage_edges(
    db: &mut GraphDB,
    token_by_index: &BTreeMap<TextProperty, NodeID>,
    token_to_index: &BTreeMap<NodeID, TextProperty>,
    node_to_right: &BTreeMap<NodeID, u32>,
    left_to_node: &MultiMap<TextProperty, NodeID>,
    token_by_left_textpos: &BTreeMap<TextProperty, NodeID>,
    token_by_right_textpos: &BTreeMap<TextProperty, NodeID>,
) -> Result<()> {

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
    {
        info!("calculating the automatically generated COVERAGE edges");
        for (textprop, n_vec) in left_to_node {
            for n in n_vec {
                if !token_to_index.contains_key(&n) {

                    let left_pos = TextProperty {
                        segmentation: String::from(""),
                        corpus_id: textprop.corpus_id,
                        text_id: textprop.text_id,
                        val: textprop.val,
                    };
                    let right_pos = node_to_right.get(&n).ok_or(Error::Other)?;
                    let right_pos = TextProperty {
                        segmentation: String::from(""),
                        corpus_id: textprop.corpus_id,
                        text_id: textprop.text_id,
                        val: right_pos.clone(),
                    };

                    // find left/right aligned basic token
                    let left_aligned_tok =
                        token_by_left_textpos.get(&left_pos).ok_or(Error::Other)?;
                    let right_aligned_tok =
                        token_by_right_textpos.get(&right_pos).ok_or(Error::Other)?;

                    let left_tok_pos = token_to_index.get(&left_aligned_tok).ok_or(Error::Other)?;
                    let right_tok_pos = token_to_index.get(&right_aligned_tok).ok_or(Error::Other)?;
                    for i in left_tok_pos.val..(right_tok_pos.val + 1) {
                        let tok_idx = TextProperty {
                            segmentation: String::from(""),
                            corpus_id: textprop.corpus_id,
                            text_id: textprop.text_id,
                            val: i,
                        };
                        let tok_id = token_by_index.get(&tok_idx).ok_or(Error::Other)?;
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

fn load_node_tab(
    path: &PathBuf,
    db: &mut GraphDB,
    corpus_id_to_name: &BTreeMap<u32, String>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
) -> Result<(MultiMap<u32, NodeID>, BTreeMap<NodeID, String>)> {

    let mut nodes_by_corpus_id : MultiMap<u32, NodeID> = MultiMap::new();
    let mut missing_seg_span : BTreeMap<NodeID, String> = BTreeMap::new();

    let mut node_tab_path = PathBuf::from(path);
    node_tab_path.push(if is_annis_33 {
        "node.annis"
    } else {
        "node.tab"
    });

    info!("loading {}", node_tab_path.to_str().unwrap_or_default());

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

            let node_nr = line.get(0).ok_or(Error::MissingColumn)?.parse::<NodeID>()?;
            let has_segmentations = is_annis_33 || line.len() > 10;
            let token_index_raw = line.get(7).ok_or(Error::MissingColumn)?;
            let text_id = line.get(1).ok_or(Error::MissingColumn)?.parse::<u32>()?;
            let corpus_id = line.get(2).ok_or(Error::MissingColumn)?.parse::<u32>()?;
            let layer: &str = line.get(3).ok_or(Error::MissingColumn)?;
            let node_name = line.get(4).ok_or(Error::MissingColumn)?;


            nodes_by_corpus_id.insert(corpus_id.clone(), node_nr.clone());

            let doc_name = corpus_id_to_name.get(&corpus_id).ok_or(
                Error::DocumentMissing,
            )?;

            let node_qname = format!("{}/{}#{}", toplevel_corpus_name, doc_name, node_name);
            let node_name_anno = Annotation {
                key: db.get_node_name_key(),
                val: db.strings.add(&node_qname),
            };
            db.node_annos.insert(node_nr, node_name_anno);

            let node_type_anno = Annotation {
                key: db.get_node_type_key(),
                val: db.strings.add("node"),
            };
            db.node_annos.insert(node_nr, node_type_anno);

            if !layer.is_empty() && layer != "NULL" {
                let layer_anno = Annotation {
                    key: AnnoKey {
                        ns: db.strings.add("annis"),
                        name: db.strings.add("layer"),
                    },
                    val: db.strings.add(layer),
                };
                db.node_annos.insert(node_nr, layer_anno);
            }

            let left_val = line.get(5).ok_or(Error::MissingColumn)?.parse::<u32>()?;
            let left = TextProperty {
                segmentation: String::from(""),
                val: left_val,
                corpus_id,
                text_id,
            };
            let right_val = line.get(6).ok_or(Error::MissingColumn)?.parse::<u32>()?;
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
                    line.get(12).ok_or(Error::MissingColumn)?
                } else {
                    line.get(9).ok_or(Error::MissingColumn)?
                };

                let tok_anno = Annotation {
                    key: db.get_token_key(),
                    val: db.strings.add(span),
                };
                db.node_annos.insert(node_nr, tok_anno);

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
                    line.get(11).ok_or(Error::MissingColumn)?
                } else {
                    line.get(8).ok_or(Error::MissingColumn)?
                };

                if segmentation_name != "NULL" {
                    let seg_index = if is_annis_33 {
                        line.get(10).ok_or(Error::MissingColumn)?.parse::<u32>()?
                    } else {
                        line.get(9).ok_or(Error::MissingColumn)?.parse::<u32>()?
                    };

                    if is_annis_33 {
                        // directly add the span information
                        let tok_anno = Annotation {
                            key: db.get_token_key(),
                            val: db.strings.add(line.get(12).ok_or(Error::MissingColumn)?),
                        };
                        db.node_annos.insert(node_nr, tok_anno);
                    } else {
                        // we need to get the span information from the node_annotation file later
                        missing_seg_span.insert(node_nr, String::from(segmentation_name));
                    }
                    // also add the specific segmentation index
                    let index = TextProperty {
                        segmentation: String::from(segmentation_name),
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
    )?;
    Ok((nodes_by_corpus_id, missing_seg_span))
}

fn load_node_anno_tab(
    path: &PathBuf,
    db: &mut GraphDB,
    missing_seg_span: &BTreeMap<NodeID, String>,
    is_annis_33: bool,
) -> Result<()> {

    let mut node_anno_tab_path = PathBuf::from(path);
    node_anno_tab_path.push(if is_annis_33 {
        "node_annotation.annis"
    } else {
        "node_annotation.tab"
    });

    info!(
        "loading {}",
        node_anno_tab_path.to_str().unwrap_or_default()
    );

    let mut node_anno_tab_csv = postgresql_import_reader(node_anno_tab_path.as_path())?;

    for result in node_anno_tab_csv.records() {
        let line = result?;

        let col_id = line.get(0).ok_or(Error::MissingColumn)?;
        let node_id: NodeID = col_id.parse()?;
        let col_ns = line.get(1).ok_or(Error::MissingColumn)?;
        let col_name = line.get(2).ok_or(Error::MissingColumn)?;
        let col_val = line.get(3).ok_or(Error::MissingColumn)?;
        // we have to make some sanity checks
        if col_ns != "annis" || col_name != "tok" {
            let anno_val = if col_val == "NULL" {
                // use an "invalid" string so it can't be found by its value, but only by its annotation name
                <StringID>::max_value()
            } else {
                db.strings.add(col_val)
            };

            db.node_annos.insert(
                node_id.clone(),
                Annotation {
                    key: AnnoKey {
                        ns: db.strings.add(col_ns),
                        name: db.strings.add(col_name),
                    },
                    val: anno_val,
                },
            );

            // add all missing span values from the annotation, but don't add NULL values
            if let Some(seg) = missing_seg_span.get(&node_id) {
                if seg == line.get(2).ok_or(Error::MissingColumn)? &&
                    line.get(3).ok_or(Error::MissingColumn)? != "NULL"
                {

                    let tok_key = db.get_token_key();
                    db.node_annos.insert(
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

fn load_component_tab(
    path: &PathBuf,
    db: &mut GraphDB,
    is_annis_33: bool,
) -> Result<BTreeMap<u32, Component>> {

    let mut component_tab_path = PathBuf::from(path);
    component_tab_path.push(if is_annis_33 {
        "component.annis"
    } else {
        "component.tab"
    });

    info!(
        "loading {}",
        component_tab_path.to_str().unwrap_or_default()
    );

    let mut component_by_id: BTreeMap<u32, Component> = BTreeMap::new();

    let mut component_tab_csv = postgresql_import_reader(component_tab_path.as_path())?;
    for result in component_tab_csv.records() {
        let line = result?;

        let cid: u32 = line.get(0).ok_or(Error::MissingColumn)?.parse()?;
        let col_type = line.get(1).ok_or(Error::MissingColumn)?;
        if col_type != "NULL" {
            let layer = String::from(line.get(2).ok_or(Error::MissingColumn)?);
            let name = String::from(line.get(3).ok_or(Error::MissingColumn)?);

            let ctype = component_type_from_short_name(col_type)?;
            let c = Component { ctype, layer, name };
            db.get_or_create_writable(c.clone())?;
            component_by_id.insert(cid, c);
        }
    }
    Ok(component_by_id)
}

fn load_nodes(
    path: &PathBuf,
    db: &mut GraphDB,
    corpus_id_to_name: &BTreeMap<u32, String>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
) -> Result<MultiMap<u32, NodeID>> {
    
    let (nodes_by_corpus_id, missing_seg_span) = load_node_tab(
        path,
        db,
        corpus_id_to_name,
        toplevel_corpus_name,
        is_annis_33,
    )?;
    load_node_anno_tab(path, db, &missing_seg_span, is_annis_33)?;

    load_component_tab(path, db, is_annis_33)?;

    return Ok(nodes_by_corpus_id);
}

fn load_rank_tab(
    path: &PathBuf,
    db: &mut GraphDB,
    component_by_id: &BTreeMap<u32, Component>,
    is_annis_33: bool,
) -> Result<(BTreeMap<u32, Component>, BTreeMap<u32, Edge>)> {

    let mut rank_tab_path = PathBuf::from(path);
    rank_tab_path.push(if is_annis_33 {
        "rank.annis"
    } else {
        "rank.tab"
    });

    info!("loading {}", rank_tab_path.to_str().unwrap_or_default());

    let mut rank_tab_csv = postgresql_import_reader(rank_tab_path.as_path())?;

    let pos_node_ref = if is_annis_33 { 3 } else { 2 };
    let pos_component_ref = if is_annis_33 { 4 } else { 3 };
    let pos_parent = if is_annis_33 { 5 } else { 4 };

    // first run: collect all pre-order values for a node
    let mut pre_to_node_id: BTreeMap<u32, NodeID> = BTreeMap::new();
    for result in rank_tab_csv.records() {
        let line = result?;
        let pre: u32 = line.get(0).ok_or(Error::MissingColumn)?.parse()?;
        let node_id: NodeID = line.get(pos_node_ref).ok_or(Error::MissingColumn)?.parse()?;
        pre_to_node_id.insert(pre, node_id);
    }

    let mut pre_to_component: BTreeMap<u32, Component> = BTreeMap::new();
    let mut pre_to_edge: BTreeMap<u32, Edge> = BTreeMap::new();
    // second run: get the actual edges
    for result in rank_tab_csv.records() {
        let line = result?;

        let parent_as_str = line.get(pos_parent).ok_or(Error::MissingColumn)?;
        if parent_as_str != "NULL" {
            let parent: u32 = parent_as_str.parse()?;
            if let Some(source) = pre_to_node_id.get(&parent) {
                // find the responsible edge database by the component ID
                let component_ref: u32 = line.get(pos_component_ref)
                    .ok_or(Error::MissingColumn)?
                    .parse()?;
                if let Some(c) = component_by_id.get(&component_ref) {
                    let target: NodeID =
                        line.get(pos_node_ref).ok_or(Error::MissingColumn)?.parse()?;

                    let gs = db.get_or_create_writable(c.clone())?;
                    let e = Edge {
                        source: source.clone(),
                        target,
                    };
                    gs.add_edge(e.clone());

                    let pre: u32 = line.get(0).ok_or(Error::MissingColumn)?.parse()?;
                    pre_to_edge.insert(pre.clone(), e);
                    pre_to_component.insert(pre, c.clone());
                }
            }
        }
    }


    Ok((pre_to_component, pre_to_edge))
}

fn load_edge_annotation(
    path: &PathBuf,
    db: &mut GraphDB,
    pre_to_component: &BTreeMap<u32, Component>,
    pre_to_edge: &BTreeMap<u32, Edge>,
    is_annis_33: bool,
) -> Result<()> {

    let mut edge_anno_tab_path = PathBuf::from(path);
    edge_anno_tab_path.push(if is_annis_33 {
        "ede_annotation.annis"
    } else {
        "ede_annotation.tab"
    });

    info!("loading {}", edge_anno_tab_path.to_str().unwrap_or_default());

    let mut edge_anno_tab_csv = postgresql_import_reader(edge_anno_tab_path.as_path())?;

    for result in edge_anno_tab_csv.records() {
        let line = result?;

        let pre : u32 = line.get(0).ok_or(Error::MissingColumn)?.parse()?;
        if let Some(c) = pre_to_component.get(&pre) {
            if let Some(e) = pre_to_edge.get(&pre) {
                let ns = line.get(1).ok_or(Error::MissingColumn)?;
                let name = line.get(2).ok_or(Error::MissingColumn)?;
                let val = line.get(3).ok_or(Error::MissingColumn)?;
                
                let anno = Annotation {
                    key : AnnoKey {ns: db.strings.add(ns), name: db.strings.add(name)},
                    val : db.strings.add(val),
                };
                let gs : &mut WriteableGraphStorage = db.get_or_create_writable(c.clone())?;
                gs.add_edge_annotation(e.clone(), anno);
            }
        }
    }

    Ok(())
}

fn load_corpus_annotation(path : &PathBuf, db : &mut GraphDB, is_annis_33 : bool) -> Result<MultiMap<u32,Annotation>> {
    
    let mut corpus_id_to_anno = MultiMap::new();

    let mut corpus_anno_tab_path = PathBuf::from(path);
    corpus_anno_tab_path.push(if is_annis_33 {
        "corpus_annotation.annis"
    } else {
        "corpus_annotation.tab"
    });

    info!("loading {}", corpus_anno_tab_path.to_str().unwrap_or_default());

    let mut corpus_anno_tab_csv = postgresql_import_reader(corpus_anno_tab_path.as_path())?;

    for result in corpus_anno_tab_csv.records() {
        let line = result?;

        let id = line.get(0).ok_or(Error::MissingColumn)?.parse()?;
        let ns = line.get(1).ok_or(Error::MissingColumn)?;
        let ns = if ns == "NULL" {""} else {ns};
        let name = line.get(2).ok_or(Error::MissingColumn)?;
        let val = line.get(3).ok_or(Error::MissingColumn)?;

        let anno = Annotation {
            key : AnnoKey{ns: db.strings.add(ns), name : db.strings.add(name)},
            val : db.strings.add(val),
        };

        corpus_id_to_anno.insert(id, anno);
        
    }

    Ok(corpus_id_to_anno)
}

fn add_subcorpora(db : &mut GraphDB,
    toplevel_corpus_name : &str, 
    corpus_by_preorder : &BTreeMap<u32, u32>, 
    corpus_id_to_name : &BTreeMap<u32, String>,
    nodes_by_corpus_id : &MultiMap<u32, NodeID>,
    corpus_id_to_annos : &MultiMap<u32,Annotation>,
    ) 
    -> Result<()> {

    let component_subcorpus = Component {
        ctype: ComponentType::PartOfSubcorpus,
        layer: String::from("annis"),
        name: String::from(""), 
    };

    let mut next_node_id : NodeID = if let Some(id) = db.node_annos.largest_key() {id+1} else {0};


    // add the toplevel corpus as node
    let top_anno = Annotation {
        key : db.get_node_name_key(),
        val : db.strings.add(toplevel_corpus_name),
    };
    db.node_annos.insert(next_node_id, top_anno);
    // add all metadata for the top-level corpus node
    if let Some(cid) = corpus_by_preorder.get(&0) {
        if let Some(anno_vec) = corpus_id_to_annos.get_vec(cid) {
            for anno in anno_vec {
                db.node_annos.insert(next_node_id, anno.clone());
            }
        }   
    }
    let toplevel_node_id = next_node_id;
    next_node_id += 1;
    
    // add all subcorpora/documents (start with the largest pre-order)
    for (pre, corpus_id) in corpus_by_preorder.iter().rev() {

        let corpus_name = corpus_id_to_name.get(corpus_id).ok_or(Error::Other)?;
        let full_name = format!("{}/{}", toplevel_corpus_name, corpus_name);


        // add a basic node labels for the new (sub-) corpus/document
        let anno_name = Annotation {
            key : db.get_node_name_key(),
            val : db.strings.add(&full_name)
        };
        db.node_annos.insert(next_node_id.clone(), anno_name);

        let anno_doc = Annotation {
            key : AnnoKey {ns: db.strings.add("annis"), name : db.strings.add("doc")},
            val : db.strings.add(corpus_name)
        };
        db.node_annos.insert(next_node_id.clone(), anno_doc);

        let anno_type = Annotation {
            key : db.get_node_type_key(),
            val : db.strings.add("corpus")
        };
        db.node_annos.insert(next_node_id.clone(), anno_type);

        // add all metadata for the document node
        if let Some(anno_vec) = corpus_id_to_annos.get_vec(&pre) {
            for anno in anno_vec {
                db.node_annos.insert(next_node_id.clone(), anno.clone());
            }
        }

        {
            let gs = db.get_or_create_writable(component_subcorpus.clone())?;

            // find all nodes belonging to this document and add a relation
            if let Some(n_vec) = nodes_by_corpus_id.get_vec(corpus_id) {
                
                for n in n_vec {
                    gs.add_edge(Edge {source: n.clone(), target: next_node_id.clone()});
                }
            }
            // also add an edge from the document to the top-level corpus
            gs.add_edge(Edge {source: next_node_id, target : toplevel_node_id});
        }

        next_node_id += 1;
    }

    Ok(())
}

fn component_type_from_short_name(short_type: &str) -> Result<ComponentType> {
    match short_type {
        "c" => Ok(ComponentType::Coverage),
        "d" => Ok(ComponentType::Dominance),
        "p" => Ok(ComponentType::Pointing),
        "o" => Ok(ComponentType::Ordering),
        _ => Err(Error::InvalidShortComponentType),
    }
}
