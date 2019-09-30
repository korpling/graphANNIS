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
    by_anno_ns: sled::Tree,
}

fn str_vec_key(val : &[&str]) -> Vec<u8> {
    let mut result : Vec<u8> = Vec::default();
    for v in val {
        // append null-terminated string to result
        for b in v.as_bytes() {
            result.push(*b)
        }
        result.push(0);
    }
    result
}

// fn remove_element_from_sorted_vector(tree : &tree, key : &[u8], val : &[u8]) {
//     if let Some(elements) = tree.get(key).expect("Database should work") {
//         let mut elements = ByAnnoValue::from(items_for_anno.as_ref());
//         if let Ok(item_idx) = value.items.binary_search(&item) {
//             value.items.remove(item_idx);
//             // store back item vector
//             let value : Vec<u8> = value.into();
//             self.by_anno_ns.insert(&key_ns, value).expect("Database should work");
//         }
//     }
// }


impl<T: Ord + Hash + MallocSizeOf + Default> AnnoStorageImpl<T> {
    pub fn new(path: &Path) -> AnnoStorageImpl<T> {
        let db = sled::Db::open(path).expect("Can't create annotation storage");

        let by_container = db
            .open_tree("by_container")
            .expect("Can't create annotation storage");
        let by_anno_name = db
            .open_tree("by_anno_name")
            .expect("Can't create annotation storage");

        let by_anno_ns = db
            .open_tree("by_anno_ns")
            .expect("Can't create annotation storage");

        AnnoStorageImpl {
            phantom: PhantomData::default(),
            by_container,
            by_anno_name,
            by_anno_ns,
        }
    }



    fn remove_element_from_by_anno(&mut self, anno: &Annotation, item: NodeID) {
        
        let key_ns : Vec<u8> = str_vec_key(&[&anno.key.name, &anno.key.ns, &anno.val]);
        if let Some(items_for_anno) = self.by_anno_ns.get(&key_ns).expect(DEFAULT_MSG) {
            let mut value = ByAnnoValue::from(items_for_anno.as_ref());
            if let Ok(item_idx) = value.items.binary_search(&item) {
                value.items.remove(item_idx);
                // store back item vector
                let value : Vec<u8> = value.into();
                self.by_anno_ns.insert(&key_ns, value).expect(DEFAULT_MSG);
            }
        }

        let key_name : Vec<u8> = str_vec_key(&[&anno.key.name, &anno.val]);
        if let Some(items_for_anno) = self.by_anno_ns.get(&key_name).expect(DEFAULT_MSG) {
            let mut value = ByAnnoValue::from(items_for_anno.as_ref());
            if let Ok(item_idx) = value.items.binary_search(&item) {
                value.items.remove(item_idx);
                // store back item vector
                let value : Vec<u8> = value.into();
                self.by_anno_ns.insert(&key_name, value).expect(DEFAULT_MSG);
            }
        }
        

    }
}

#[derive(Serialize, Deserialize)]
struct ByContainerValue {
    annotations: Vec<Annotation>,
}

impl Into<Vec<u8>> for ByContainerValue {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

impl From<&[u8]> for ByContainerValue {
    fn from(val: &[u8]) -> ByContainerValue {
        bincode::deserialize(val).unwrap()
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
        let mut by_container_value: ByContainerValue = if let Some(existing) = self
            .by_container
            .get(item.to_le_bytes())
            .expect(DEFAULT_MSG)
        {
            ByContainerValue::from(existing.as_ref())
        } else {
            ByContainerValue {
                annotations: vec![],
            }
        };

        // check if there is already an item with the same annotation key
        let existing_entry_idx = by_container_value
            .annotations
            .binary_search_by_key(&anno.key, |a| a.key.clone());

        let existing_anno = {
            if let Ok(existing_entry_idx) = existing_entry_idx {
                let orig_anno = by_container_value.annotations[existing_entry_idx].clone();
                // abort if the same annotation key with the same value already exist
                if orig_anno.val == anno.val {
                    return;
                }
                // insert annotation for item at existing position
                by_container_value.annotations[existing_entry_idx] = anno.clone();
                Some(orig_anno)
            } else if let Err(insertion_idx) = existing_entry_idx {
                // insert at sorted position -> the result will still be a sorted vector
                by_container_value
                    .annotations
                    .insert(insertion_idx, anno.clone());
                None
            } else {
                None
            }
        };

        // write back possibly updated by_container value
        let by_container_value: Vec<u8> = by_container_value.into();
        self.by_container
            .insert(item.to_le_bytes(), by_container_value)
            .expect(DEFAULT_MSG);

        if let Some(ref existing_anno) = existing_anno {
            self.remove_element_from_by_anno(existing_anno, item);
        }

        // inserts a new relation between the annotation and the item
        // if set is not existing yet it is created
        let by_anno_name_key = str_vec_key(&[&anno.key.name, &anno.val]);
//        self.by_anno_name.insert(by_anno_name_key, value: V)

    }

    

    fn get_annotations_for_item(&self, item: &NodeID) -> Vec<Annotation> {
        unimplemented!()
    }

    fn remove_annotation_for_item(&mut self, _item: &NodeID, _key: &AnnoKey) -> Option<String> {
        unimplemented!()
    }

    fn clear(&mut self) {
        unimplemented!()
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
