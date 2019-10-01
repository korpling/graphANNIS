use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::types::AnnoKey;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::Annotation;
use crate::annis::types::NodeID;
use crate::malloc_size_of::MallocSizeOf;

use std::convert::TryInto;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;
use std::borrow::Cow;

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
pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default> {
    phantom: PhantomData<T>,

    #[ignore_malloc_size_of = "is stored on disk"]
    by_container: sled::Tree,
    #[ignore_malloc_size_of = "is stored on disk"]
    by_anno_name: sled::Tree,
    #[ignore_malloc_size_of = "is stored on disk"]
    by_anno_qname: sled::Tree,
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
    data.split(|b| *b == 0).map(|part| std::str::from_utf8(part).expect(UTF_8_MSG)).collect()
}

/// Creates a key for the `by_container` tree.
///
/// Structure:
/// ```text
/// [64 Bits Node ID][Namespace]\0[Name]\0
/// ```
fn create_by_container_key(node : NodeID, anno_key : &AnnoKey) -> Vec<u8> {
    let mut result: Vec<u8> = node.to_le_bytes().iter().cloned().collect();
    result.extend(create_str_vec_key(&[&anno_key.ns, &anno_key.name]));
    result
}

fn parse_by_container_key(data : &[u8]) -> (NodeID, AnnoKey) {
    let item = NodeID::from_le_bytes(data[0..8].try_into().expect("Key data must at least have length 8"));
    let str_vec = parse_str_vec_key(&data[8..]);

    let anno_key = AnnoKey {
        ns: str_vec[0].to_string(),
        name: str_vec[1].to_string(),
    };
    (item, anno_key)
}

/// Creates a key for the `by_anno_name` tree.
/// 
/// Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
/// arrays as values, the node ID is part of the key and makes it unique.
/// 
/// Structure:
/// ```text
/// [Name]\0[Value]\0[Namespace]\0[8 Bits Node ID]
/// ```
fn create_by_anno_name_key(node : NodeID, anno : &Annotation) -> Vec<u8> {
    // Use the annotation name, the value and the node ID as key for the indexes.
    // Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
    // arrays as values, the node ID is part of the key and makes it unique.
    let mut result : Vec<u8> = create_str_vec_key(&[&anno.key.name, &anno.val, &anno.key.ns]);
    result.extend(&node.to_le_bytes());
    result
}



/// Creates a key for the `by_anno_qname` tree.
///
/// Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
/// arrays as values, the node ID is part of the key and makes it unique.
/// 
/// Structure:
/// ```text
/// [Namespace]\0[Name]\0[Value]\0[8 Bits Node ID]
/// ```
fn create_by_anno_qname_key(node : NodeID, anno : &Annotation) -> Vec<u8> {
    // Use the qualified annotation name, the value and the node ID as key for the indexes.

    let mut result : Vec<u8> = create_str_vec_key(&[&anno.key.ns, &anno.key.name, &anno.val]);
    result.extend(&node.to_le_bytes());
    result
}


impl<T: Ord + Hash + MallocSizeOf + Default> AnnoStorageImpl<T> {
    pub fn new(path: &Path) -> AnnoStorageImpl<T> {
        let db = sled::Db::open(path).expect("Can't create annotation storage");

        let by_container = db
            .open_tree("by_container")
            .expect("Can't create annotation storage");
        let by_anno_name = db
            .open_tree("by_anno_name")
            .expect("Can't create annotation storage");

        let by_anno_qname = db
            .open_tree("by_anno_qname")
            .expect("Can't create annotation storage");

        AnnoStorageImpl {
            phantom: PhantomData::default(),
            by_container,
            by_anno_name,
            by_anno_qname,
        }
    }
}

impl<'de> AnnotationStorage<NodeID> for AnnoStorageImpl<NodeID> {
    fn insert(&mut self, item: NodeID, anno: Annotation) {
        // insert the value into main tree
        self.by_container
            .insert(create_by_container_key(item, &anno.key), anno.val.as_bytes())
            .expect(DEFAULT_MSG);

        
        // To save some space, only a marker value ([1]) of one byte is actually inserted.
        self.by_anno_name.insert(create_by_anno_name_key(item, &anno), &[1]).expect(DEFAULT_MSG);
        self.by_anno_qname.insert(create_by_anno_qname_key(item, &anno), &[1]).expect(DEFAULT_MSG);
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

    fn remove_annotation_for_item(&mut self, _item: &NodeID, _key: &AnnoKey) -> Option<String> {
        unimplemented!()
    }

    fn clear(&mut self) {
        self.by_anno_name.clear().expect(DEFAULT_MSG);
        self.by_anno_qname.clear().expect(DEFAULT_MSG);
        self.by_container.clear().expect(DEFAULT_MSG);
    }

    fn get_qnames(&self, _name: &str) -> Vec<AnnoKey> {
        unimplemented!()
    }

    fn get_key_id(&self, _key: &AnnoKey) -> Option<AnnoKeyID> {
        unimplemented!()
    }

    fn get_key_value(&self, _key_id: AnnoKeyID) -> Option<AnnoKey> {
        unimplemented!()
    }

    fn number_of_annotations(&self) -> usize {
        unimplemented!()
    }

    fn get_value_for_item(&self, item: &NodeID, key: &AnnoKey) -> Option<Cow<str>> {
        let raw = self.by_container.get(create_by_container_key(*item, key)).expect(DEFAULT_MSG);
        if let Some(raw) = raw {
            let val : String = String::from_utf8_lossy(&raw).to_string();
            Some(Cow::Owned(val))
        } else {
            None
        }
    }

    fn get_value_for_item_by_id(&self, _item: &NodeID, _key_id: AnnoKeyID) -> Option<Cow<str>> {
        unimplemented!()
    }

    fn number_of_annotations_by_name(&self, _ns: Option<String>, _name: String) -> usize {
        unimplemented!()
    }

    fn exact_anno_search<'a>(
        &'a self,
        _namespace: Option<String>,
        _name: String,
        _value: ValueSearch<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        unimplemented!()
    }

    fn regex_anno_search<'a>(
        &'a self,
        _namespace: Option<String>,
        _name: String,
        _pattern: &str,
        _negated: bool,
    ) -> Box<Iterator<Item = Match> + 'a> {
        unimplemented!()
    }

    fn find_annotations_for_item(
        &self,
        _item: &NodeID,
        _ns: Option<String>,
        _name: Option<String>,
    ) -> Vec<AnnoKeyID> {
        unimplemented!()
    }

    fn guess_max_count(
        &self,
        _ns: Option<String>,
        _name: String,
        _lower_val: &str,
        _upper_val: &str,
    ) -> usize {
        unimplemented!()
    }

    fn guess_max_count_regex(&self, _ns: Option<String>, _name: String, _pattern: &str) -> usize {
        unimplemented!()
    }

    fn guess_most_frequent_value(&self, _ns: Option<String>, _name: String) -> Option<String> {
        unimplemented!()
    }

    fn get_all_values(&self, _key: &AnnoKey, _most_frequent_first: bool) -> Vec<Cow<str>> {
        unimplemented!()
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        unimplemented!()
    }

    fn get_largest_item(&self) -> Option<NodeID> {
        unimplemented!()
    }

    fn calculate_statistics(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::annis::types::NodeID;

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
        let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new(path.path());

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
