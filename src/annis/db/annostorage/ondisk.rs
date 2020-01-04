use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::errors::*;
use crate::annis::types::AnnoKey;
use crate::annis::types::Annotation;
use crate::annis::types::NodeID;
use crate::annis::util;
use crate::annis::util::memory_estimation;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::path::Path;
use std::sync::Arc;

const DEFAULT_MSG : &str = "Accessing the disk-database failed. This is a non-recoverable error since it means something serious is wrong with the disk or file system.";
const UTF_8_MSG: &str = "String must be valid UTF-8 but was corrupted";

/// An on-disk implementation of an annotation storage.
///
/// # Error handling
///
/// In contrast to the main-memory implementation, accessing the disk can fail.
/// This is handled as a fatal error with panic except for specific scenarios where we know how to recover from this error.
/// Panics are used because these errors are unrecoverable
/// (e.g. if the file is suddenly missing this is like if someone removed the main memory)
/// and there is no way of delivering a correct answer.
/// Retrying the same query again will also not succeed since temporary errors are already handled internally.
#[derive(MallocSizeOf)]
pub struct AnnoStorageImpl {
    #[ignore_malloc_size_of = "is stored on disk"]
    by_container: sled::Tree,
    #[ignore_malloc_size_of = "is stored on disk"]
    by_anno_qname: sled::Tree,

    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    anno_key_sizes: BTreeMap<AnnoKey, usize>,
    largest_item: Option<NodeID>,
    total_number_of_annos: usize,
}

fn create_str_vec_key(val: &[&str]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::default();
    for v in val {
        // append null-terminated string to result
        for b in v.as_bytes() {
            result.push(*b)
        }
        result.push(0);
    }
    result
}

fn parse_str_vec_key(data: &[u8]) -> Vec<&str> {
    data.split(|b| *b == 0)
        .map(|part| std::str::from_utf8(part).expect(UTF_8_MSG))
        .collect()
}

/// Creates a key for the `by_container` tree.
///
/// Structure:
/// ```text
/// [64 Bits Node ID][Namespace]\0[Name]\0
/// ```
fn create_by_container_key(node: NodeID, anno_key: &AnnoKey) -> Vec<u8> {
    let mut result: Vec<u8> = node.to_le_bytes().iter().cloned().collect();
    result.extend(create_str_vec_key(&[&anno_key.ns, &anno_key.name]));
    result
}

fn parse_by_container_key(data: &[u8]) -> (NodeID, AnnoKey) {
    let item = NodeID::from_le_bytes(
        data[0..8]
            .try_into()
            .expect("Key data must at least have length 8"),
    );
    let str_vec = parse_str_vec_key(&data[8..]);

    let anno_key = AnnoKey {
        ns: str_vec[0].to_string(),
        name: str_vec[1].to_string(),
    };
    (item, anno_key)
}

/// Creates a key for the `by_anno_qname` tree.
///
/// Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
/// arrays as values, the node ID is part of the key and makes it unique.
///
/// Structure:
/// ```text
/// [Namespace]\0[Name]\0[Value]\0[64 Bits Node ID]
/// ```
fn create_by_anno_qname_key(node: NodeID, anno: &Annotation) -> Vec<u8> {
    // Use the qualified annotation name, the value and the node ID as key for the indexes.

    let mut result: Vec<u8> = create_str_vec_key(&[&anno.key.ns, &anno.key.name, &anno.val]);
    result.extend(&node.to_le_bytes());
    result
}

impl AnnoStorageImpl {
    pub fn new(path: &Path) -> AnnoStorageImpl {
        let db = sled::open(path).expect("Can't create annotation storage");

        let by_container = db
            .open_tree("by_container")
            .expect("Can't create annotation storage");
        let by_anno_qname = db
            .open_tree("by_anno_qname")
            .expect("Can't create annotation storage");

        AnnoStorageImpl {
            by_container,
            by_anno_qname,
            anno_key_sizes: BTreeMap::new(),
            largest_item: None,
            total_number_of_annos: 0,
        }
    }

    fn matching_items<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: Option<&str>,
    ) -> Box<dyn Iterator<Item = (NodeID, Arc<AnnoKey>)> + 'a> {
        let key_ranges: Vec<Arc<AnnoKey>> = if let Some(ns) = namespace {
            vec![Arc::from(AnnoKey {
                ns: ns.to_string(),
                name: name.to_string(),
            })]
        } else {
            self.get_qnames(name)
                .into_iter()
                .map(|key| Arc::from(key))
                .collect()
        };

        let annotation_ranges: Vec<(Arc<AnnoKey>, Vec<u8>, Vec<u8>)> = key_ranges
            .into_iter()
            .map(|key| {
                let lower_bound = Annotation {
                    key: key.as_ref().clone(),
                    val: if let Some(value) = value {
                        value.to_string()
                    } else {
                        "".to_string()
                    },
                };

                let upper_bound = Annotation {
                    key: key.as_ref().clone(),
                    val: if let Some(value) = value {
                        value.to_string()
                    } else {
                        "\0".to_string()
                    },
                };

                let lower_bound = create_by_anno_qname_key(NodeID::min_value(), &lower_bound);
                let upper_bound = create_by_anno_qname_key(NodeID::max_value(), &upper_bound);

                (key, lower_bound, upper_bound)
            })
            .collect();

        let it = annotation_ranges
            .into_iter()
            .flat_map(move |(key, lower_bound, upper_bound)| {
                self.by_anno_qname
                    .range(lower_bound..upper_bound)
                    .map(|data| {
                        // the value is only a marker, use the key to extract the node ID
                        let (data, _) = data.expect(DEFAULT_MSG);
                        let node_id = NodeID::from_le_bytes(
                            data[(data.len() - 8)..]
                                .try_into()
                                .expect("Key data must at least have length 8"),
                        );
                        node_id
                    })
                    .zip(std::iter::repeat(key))
            });

        Box::new(it)
    }
}

impl<'de> AnnotationStorage<NodeID> for AnnoStorageImpl {
    fn insert(&mut self, item: NodeID, anno: Annotation) {
        // insert the value into main tree
        let existing_anno = self
            .by_container
            .insert(
                create_by_container_key(item, &anno.key),
                anno.val.as_bytes(),
            )
            .expect(DEFAULT_MSG);

        // To save some space, only a marker value ([1]) of one byte is actually inserted.
        self.by_anno_qname
            .insert(create_by_anno_qname_key(item, &anno), &[1])
            .expect(DEFAULT_MSG);

        if existing_anno.is_none() {
            // a new annotation entry was inserted and did not replace an existing one
            self.total_number_of_annos += 1;

            if let Some(largest_item) = self.largest_item.clone() {
                if largest_item < item {
                    self.largest_item = Some(item);
                }
            } else {
                self.largest_item = Some(item);
            }

            let anno_key_entry = self.anno_key_sizes.entry(anno.key.clone()).or_insert(0);
            *anno_key_entry += 1;
        }
    }

    fn get_annotations_for_item(&self, item: &NodeID) -> Vec<Annotation> {
        let mut start_key: Vec<u8> = item.to_le_bytes().iter().cloned().collect();
        start_key.extend(&NodeID::min_value().to_le_bytes());

        let mut end_key: Vec<u8> = item.to_le_bytes().iter().cloned().collect();
        end_key.extend(&NodeID::max_value().to_le_bytes());

        let mut result = Vec::default();
        for it_val in self.by_container.range(start_key..=end_key) {
            if let Ok((key, val)) = it_val {
                let parsed_key = parse_by_container_key(&key);
                let anno = Annotation {
                    key: parsed_key.1,
                    val: String::from_utf8_lossy(&val).to_string(),
                };
                result.push(anno);
            }
        }

        result
    }

    fn remove_annotation_for_item(&mut self, _item: &NodeID, key: &AnnoKey) -> Option<Cow<str>> {
        // TODO: remove annotation from disk trees

        // decrease the annotation count for this key
        let new_key_count: usize = if let Some(num_of_keys) = self.anno_key_sizes.get_mut(key) {
            *num_of_keys -= 1;
            *num_of_keys
        } else {
            0
        };
        // if annotation count dropped to zero remove the key
        if new_key_count == 0 {
            self.anno_key_sizes.remove(key);
        }

        unimplemented!();
    }

    fn clear(&mut self) {
        self.by_anno_qname.clear().expect(DEFAULT_MSG);
        self.by_container.clear().expect(DEFAULT_MSG);
        self.largest_item = None;
        self.anno_key_sizes.clear();
    }

    fn get_qnames(&self, name: &str) -> Vec<AnnoKey> {
        let it = self.anno_key_sizes.range(
            AnnoKey {
                name: name.to_owned(),
                ns: String::default(),
            }..,
        );
        let mut result: Vec<AnnoKey> = Vec::default();
        for (k, _) in it {
            if k.name == name {
                result.push(k.clone());
            } else {
                break;
            }
        }
        result
    }

    fn number_of_annotations(&self) -> usize {
        self.by_container.len()
    }

    fn get_value_for_item(&self, item: &NodeID, key: &AnnoKey) -> Option<Cow<str>> {
        let raw = self
            .by_container
            .get(create_by_container_key(*item, key))
            .expect(DEFAULT_MSG);
        if let Some(raw) = raw {
            let val: String = String::from_utf8_lossy(&raw).to_string();
            Some(Cow::Owned(val))
        } else {
            None
        }
    }

    fn get_keys_for_iterator(
        &self,
        ns: Option<&str>,
        name: Option<&str>,
        it: Box<dyn Iterator<Item = NodeID>>,
    ) -> Vec<Match> {
        let result_it = it.flat_map(|item| {
            if let Some(_name) = name {
                if let Some(_ns) = ns {
                    unimplemented!()
                } else {
                    unimplemented!()
                }
            } else {
                // get all annotation keys for this item
                self.by_container
                    .range(item.to_le_bytes()..(item + 1).to_le_bytes())
                    .map(|data| {
                        let (data, _) = data.expect(DEFAULT_MSG);
                        let (node, matched_anno_key) = parse_by_container_key(&data);
                        Match {
                            node,
                            anno_key: Arc::from(matched_anno_key),
                        }
                    })
            }
        });
        result_it.collect()
    }

    fn number_of_annotations_by_name(&self, _ns: Option<&str>, _name: &str) -> usize {
        unimplemented!()
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: ValueSearch<&str>,
    ) -> Box<dyn Iterator<Item = Match> + 'a> {
        match value {
            ValueSearch::Any => {
                let it = self
                    .matching_items(namespace, name, None)
                    .map(move |item| item.into());
                Box::new(it)
            }
            ValueSearch::Some(value) => {
                let it = self
                    .matching_items(namespace, name, Some(value))
                    .map(move |item| item.into());
                Box::new(it)
            }
            ValueSearch::NotSome(value) => {
                let value = value.to_string();
                let it = self
                    .matching_items(namespace, name, None)
                    .filter(move |(node, anno_key)| {
                        if let Some(item_value) = self.get_value_for_item(node, anno_key) {
                            item_value != value
                        } else {
                            false
                        }
                    })
                    .map(move |item| item.into());
                Box::new(it)
            }
        }
    }

    fn regex_anno_search<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        pattern: &str,
        negated: bool,
    ) -> Box<dyn Iterator<Item = Match> + 'a> {
        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if let Ok(re) = compiled_result {
            let it = self
                .matching_items(namespace, name, None)
                .filter(move |(node, anno_key)| {
                    if let Some(val) = self.get_value_for_item(node, anno_key) {
                        if negated {
                            !re.is_match(&val)
                        } else {
                            re.is_match(&val)
                        }
                    } else {
                        false
                    }
                })
                .map(move |item| item.into());
            return Box::new(it);
        } else if negated {
            // return all values
            return self.exact_anno_search(namespace, name, None.into());
        } else {
            // if regular expression pattern is invalid return empty iterator
            return Box::new(std::iter::empty());
        }
    }

    fn get_all_keys_for_item(
        &self,
        _item: &NodeID,
        _ns: Option<&str>,
        _name: Option<&str>,
    ) -> Vec<Arc<AnnoKey>> {
        unimplemented!()
    }

    fn guess_max_count(
        &self,
        _ns: Option<&str>,
        _name: &str,
        _lower_val: &str,
        _upper_val: &str,
    ) -> usize {
        unimplemented!()
    }

    fn guess_max_count_regex(&self, _ns: Option<&str>, _name: &str, _pattern: &str) -> usize {
        unimplemented!()
    }

    fn guess_most_frequent_value(&self, _ns: Option<&str>, _name: &str) -> Option<Cow<str>> {
        unimplemented!()
    }

    fn get_all_values(&self, _key: &AnnoKey, _most_frequent_first: bool) -> Vec<Cow<str>> {
        unimplemented!()
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        self.anno_key_sizes.keys().cloned().collect()
    }

    fn get_largest_item(&self) -> Option<NodeID> {
        self.largest_item.clone()
    }

    fn calculate_statistics(&mut self) {
        unimplemented!()
    }

    fn load_annotations_from(&mut self, _location: &Path) -> Result<()> {
        unimplemented!()
    }

    fn save_annotations_to(&self, _location: &Path) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_same_anno() {
        env_logger::init();

        let test_anno = Annotation {
            key: AnnoKey {
                name: "anno1".to_owned(),
                ns: "annis".to_owned(),
            },
            val: "test".to_owned(),
        };

        let path = tempfile::TempDir::new().unwrap();
        let mut a = AnnoStorageImpl::new(path.path());

        debug!("Inserting annotation for node 1");
        a.insert(1, test_anno.clone());
        debug!("Inserting annotation for node 1 (again)");
        a.insert(1, test_anno.clone());
        debug!("Inserting annotation for node 2");
        a.insert(2, test_anno.clone());
        debug!("Inserting annotation for node 3");
        a.insert(3, test_anno);

        assert_eq!(3, a.number_of_annotations());

        assert_eq!(
            "test",
            a.get_value_for_item(
                &3,
                &AnnoKey {
                    name: "anno1".to_owned(),
                    ns: "annis".to_owned()
                }
            )
            .unwrap()
        );
    }
}
