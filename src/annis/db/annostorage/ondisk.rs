use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::errors::Result;
use crate::annis::types::AnnoKey;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::Annotation;
use crate::malloc_size_of::MallocSizeOf;
use sanakirja::{Env, Representable, Transaction, Db};
use sanakirja::value::UnsafeValue;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;
use super::SparseAnnotation;


#[derive(MallocSizeOf)]
pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default + Representable> {
    phantom: PhantomData<T>,

    //#[ignore_malloc_size_of = "state of environment is neglectable compared to the actual maps (which are non disk)"]
    //env: Env,
    
    //by_container: Db<T, UnsafeValue>,
}

impl<T: Ord + Hash + MallocSizeOf + Default + Representable> AnnoStorageImpl<T> {
    pub fn load_from_file(path: &str) -> Result<AnnoStorageImpl<T>> {
        let path = Path::new(path);
        // Use 100 MB (SI standard) as default size
        let env = Env::new(path, 100_000_000)?;
        let mut txn = env.mut_txn_begin()?;
        let by_container : Db<T, UnsafeValue> = txn.create_db()?;
        Ok(AnnoStorageImpl {
           // env,
           //by_container: env
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
        // if let Ok(txn) = self.env.mut_txn_begin() {
        
        // }
        unimplemented!()
    }

    fn get_all_keys_for_item(&self, _item: &T) -> Vec<AnnoKey> {
        unimplemented!()
    }

    fn remove_annotation_for_item(&mut self, _item: &T, _key: &AnnoKey) -> Option<String> {
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

    fn get_annotations_for_item(&self, _item: &T) -> Vec<Annotation> {
        unimplemented!()
    }

    fn number_of_annotations(&self) -> usize {
        unimplemented!()
    }

    fn get_value_for_item(&self, _item: &T, _key: &AnnoKey) -> Option<&str> {
        unimplemented!()
    }

    fn get_value_for_item_by_id(&self, _item: &T, _key_id: AnnoKeyID) -> Option<&str> {
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
        _item: &T,
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

    fn get_largest_item(&self) -> Option<T> {
        unimplemented!()
    }

    fn calculate_statistics(&mut self) {
        unimplemented!()
    }
}
