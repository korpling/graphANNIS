use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::errors::Result;
use crate::annis::types::AnnoKey;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::Annotation;
use crate::malloc_size_of::MallocSizeOf;
use sanakirja::{Env, Representable};
use std::hash::Hash;
use std::path::Path;
use std::marker::PhantomData;

#[derive(MallocSizeOf)]
pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default> {
    phantom: PhantomData<T>,

    #[ignore_malloc_size_of = "state of environment is neglectable compared to the actual maps (which are non disk)"]
    env : Env,

}

impl<T: Ord + Hash + MallocSizeOf + Default> AnnoStorageImpl<T> {
    pub fn load_from_file(path: &str) -> Result<AnnoStorageImpl<T>> {
        let path = Path::new(path);
        // Use 1 GB (SI standard) as default size
        let env = Env::new(path, 1_000_000_000)?;
        
        Ok(AnnoStorageImpl {
            env,
            phantom: PhantomData::default(),
        })
    }
}

impl<'de, T> AnnotationStorage<T> for AnnoStorageImpl<T>
where
    T: Ord + Hash + MallocSizeOf + Default + Clone + Representable + Send + Sync,
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
