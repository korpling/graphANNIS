use graphdb::GraphDB;
use annis::{AnnoKey, Annotation, NodeID};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
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
    MissingColumn,
    InvalidDataType,
    ToplevelCorpusNameNotFound,
    DirectoryNotFound,
    DocumentMissing,
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


#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct TextProperty {
    segmentation: String,
    corpus_id: u32,
    text_id: u32,
    val: u32,
}

fn postgresql_import_reader(path: &Path) -> std::result::Result<csv::Reader<File>, csv::Error> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(path)
}

fn parse_corpus_tab(
    path: &PathBuf,
    corpus_by_preorder: &mut BTreeMap<u32, u32>,
    corpus_id_to_name: &mut BTreeMap<u32, String>,
    is_annis_33: bool,
) -> Result<String> {
    let mut corpus_tab_path = PathBuf::from(path);
    corpus_tab_path.push(if is_annis_33 {
        "corpus.annis"
    } else {
        "corpus.tab"
    });

    let mut toplevel_corpus_name: Option<String> = None;

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

    toplevel_corpus_name.ok_or(Error::ToplevelCorpusNameNotFound)
}

fn load_node_tab(
    path: &PathBuf,
    db: &mut GraphDB,
    nodes_by_corpus_id: &mut MultiMap<u32, NodeID>,
    corpus_id_to_name: &mut BTreeMap<u32, String>,
    toplevel_corpus_name: &str,
    is_annis_33: bool,
) -> Result<()> {

    let mut missing_seg_span: BTreeMap<NodeID, String> = BTreeMap::new();


    let mut node_tab_path = PathBuf::from(path);
    node_tab_path.push(if is_annis_33 {
        "node.annis"
    } else {
        "node.tab"
    });

    info!("loading {}", node_tab_path.to_str().unwrap_or_default());

    // start "node.annis" visibility block
    {
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

                let doc_name = corpus_id_to_name
                    .get(&corpus_id)
                    .ok_or(Error::DocumentMissing)?;
                nodes_by_corpus_id.insert(corpus_id, node_nr);

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

                    let tok_anno = Annotation{
                        key : db.get_token_key(),
                        val : db.strings.add(span),
                    };
                    db.node_annos.insert(node_nr, tok_anno);

                    let index = TextProperty {
                        segmentation : String::from(""),
                        val : token_index_raw.parse::<u32>()?,
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
                                key : db.get_token_key(),
                                val : db.strings.add(line.get(12).ok_or(Error::MissingColumn)?),
                            };
                            db.node_annos.insert(node_nr, tok_anno);
                        } else {
                            // we need to get the span information from the node_annotation file later
                            missing_seg_span.insert(node_nr, String::from(segmentation_name));
                        }
                        // also add the specific segmentation index
                        let index = TextProperty {
                            segmentation : String::from(segmentation_name),
                            val : seg_index,
                            corpus_id,
                            text_id,
                        };
                        token_by_index.insert(index, node_nr);

                    } // end if node has segmentation info

                } // endif if check segmentations
            }
        } // end "scan all lines" visibility block

        // TODO: cleanup, better variable naming and put this into it's own function
        // iterate over all token by their order, find the nodes with the same
        // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges
        if !token_by_index.is_empty() {
            info!("calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges");

        }

    } // "node.annis" visibility block

    return Ok(());
}



pub fn load(path: &str) -> Result<GraphDB> {
    // convert to path
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

        let mut corpus_by_preorder = BTreeMap::new();
        let mut corpus_id_to_name = BTreeMap::new();
        let mut nodes_by_corpus_id: MultiMap<u32, NodeID> = MultiMap::new();
        let corpus_name = parse_corpus_tab(
            &path,
            &mut corpus_by_preorder,
            &mut corpus_id_to_name,
            is_annis_33,
        )?;

        load_node_tab(
            &path,
            &mut db,
            &mut nodes_by_corpus_id,
            &mut corpus_id_to_name,
            &corpus_name,
            is_annis_33,
        )?;


        return Ok(db);
    }

    return Err(Error::DirectoryNotFound);
}
