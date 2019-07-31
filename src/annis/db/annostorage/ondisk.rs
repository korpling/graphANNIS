use crate::annis::types::Annotation;
use crate::annis::db::Match;
use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::AnnoKey;
use crate::malloc_size_of::MallocSizeOf;
use crate::annis::db::ValueSearch;
use std::hash::Hash;

pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default> {
    phantom: std::marker::PhantomData<T>,
}

impl<'de, T> AnnotationStorage<T> for AnnoStorageImpl<T>
where
    T: Ord
        + Hash
        + MallocSizeOf
        + Default
        + Clone
        + serde::Serialize
        + serde::Deserialize<'de>
        + Send
        + Sync,
    (T, AnnoKeyID): Into<Match>,
{
    fn insert(&mut self, item: T, anno: Annotation) {
        unimplemented!()
    }

    fn get_all_keys_for_item(&self, item: &T) -> Vec<AnnoKey> {
        unimplemented!()
    }

    fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Option<String> {
       unimplemented!()
    }

    fn clear(&mut self) {
       unimplemented!()
    }

    fn get_qnames(&self, name: &str) -> Vec<AnnoKey> {
       unimplemented!()
    }

    fn get_key_id(&self, key: &AnnoKey) -> Option<AnnoKeyID> {
        unimplemented!()
    }

    fn get_key_value(&self, key_id: AnnoKeyID) -> Option<AnnoKey> {
        unimplemented!()
    }

    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation> {
       unimplemented!()
    }

    fn number_of_annotations(&self) -> usize {
        unimplemented!()
    }

    fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Option<&str> {
        unimplemented!()
    }

    fn get_value_for_item_by_id(&self, item: &T, key_id: AnnoKeyID) -> Option<&str> {
        unimplemented!()
    }

    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize {
        unimplemented!()
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: ValueSearch<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        unimplemented!()
    }

    fn regex_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        pattern: &str,
        negated: bool,
    ) -> Box<Iterator<Item = Match> + 'a> {
        unimplemented!()
    }

    fn find_annotations_for_item(
        &self,
        item: &T,
        ns: Option<String>,
        name: Option<String>,
    ) -> Vec<AnnoKeyID> {
        unimplemented!()
    }

    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        unimplemented!()
    }

    fn guess_max_count_regex(&self, ns: Option<String>, name: String, pattern: &str) -> usize {
        unimplemented!()
    }

    fn guess_most_frequent_value(&self, ns: Option<String>, name: String) -> Option<String> {
        unimplemented!()
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str> {
        unimplemented!()
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        unimplemented!()
    }

    fn get_largest_item(&self) -> Option<T> {
        unimplemented!()
    }

    fn calculate_statistics(&mut self) {
        unimplemented!()
    }
}