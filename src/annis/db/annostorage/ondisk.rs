use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::types::AnnoKey;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::Annotation;
use crate::annis::types::NodeID;
use crate::malloc_size_of::MallocSizeOf;

use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;

const DEFAULT_MSG : &str = "Accessing the disk-database failed. This is a non-recoverable error since it means something serious is wrong with the disk or file system.";

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

fn str_vec_key(val: &[&str]) -> Vec<u8> {
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

#[derive(Serialize, Deserialize)]
struct ByAnnoValue {
    items: Vec<NodeID>,
}

impl Into<Vec<u8>> for ByAnnoValue {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

impl From<&[u8]> for ByAnnoValue {
    fn from(val: &[u8]) -> ByAnnoValue {
        bincode::deserialize(val).unwrap()
    }
}

impl<'de> AnnotationStorage<NodeID> for AnnoStorageImpl<NodeID> {
    fn insert(&mut self, item: NodeID, anno: Annotation) {
        // create a key from the node ID and the annotation key
        let mut by_container_key: Vec<u8> = item.to_le_bytes().iter().cloned().collect();
        by_container_key.extend(str_vec_key(&[&anno.key.ns, &anno.key.name]));

        // insert the value into main tree
        self.by_container
            .insert(by_container_key, anno.val.as_bytes())
            .expect(DEFAULT_MSG);

        // Use the (qualified) annotation name, the value and the node ID as key for the indexes.
        // Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
        // arrays as values, the node ID is part of the key and makes it unique.
        let mut by_anno_name_key : Vec<u8> = str_vec_key(&[&anno.key.name, &anno.val, &anno.key.ns]);
        by_anno_name_key.extend(&item.to_le_bytes());
        self.by_anno_name.insert(by_anno_name_key, &[1]).expect(DEFAULT_MSG);

        let mut by_anno_qname_key : Vec<u8> = str_vec_key(&[&anno.key.ns, &anno.key.name, &anno.val]);
        by_anno_qname_key.extend(&item.to_le_bytes());
        self.by_anno_qname.insert(by_anno_qname_key, &[1]).expect(DEFAULT_MSG);
    }

    fn get_annotations_for_item(&self, item: &NodeID) -> Vec<Annotation> {
        unimplemented!()
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

    fn get_value_for_item(&self, _item: &NodeID, _key: &AnnoKey) -> Option<&str> {
        unimplemented!()
    }

    fn get_value_for_item_by_id(&self, _item: &NodeID, _key_id: AnnoKeyID) -> Option<&str> {
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

    fn get_all_values(&self, _key: &AnnoKey, _most_frequent_first: bool) -> Vec<&str> {
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
