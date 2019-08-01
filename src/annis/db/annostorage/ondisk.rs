use crate::annis::db::annostorage::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::db::ValueSearch;
use crate::annis::errors::Result;
use crate::annis::types::AnnoKey;
use crate::annis::types::AnnoKeyID;
use crate::annis::types::Annotation;
use crate::malloc_size_of::MallocSizeOf;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use sanakirja::value::UnsafeValue;
use sanakirja::{Commit, Db, Env, Txn, MutTxn, Representable, Transaction};
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;

const BY_CONTAINER_ID: usize = 0;
const BY_ANNO_ID: usize = 0;

type AnnotationsDb = Db<(UnsafeValue, UnsafeValue), UnsafeValue>;
type ByContainerDb<T> = Db<T, AnnotationsDb>;

type ValuesDb<T> = Db<UnsafeValue, T>;
type ByAnnoDb<T> = Db<(UnsafeValue, UnsafeValue), ValuesDb<T>>;

/// An on-disk implementation of an annotation storage
/// 
/// # Error handling
/// In contrast to the main-memory implementation, accessing the disk can fail.
/// This is handled as a fatal error with panic except for specific scenarios where we know how to recover from this error.
/// Panics are used because these errors are unrecoverable 
/// (e.g. if the file is suddenly missing this is like if someone removed the main memory)
/// and there is no way of delivering a correct answer. 
/// Retrying the same query again will also not succeed (we would handle temporary errors internally).
#[derive(MallocSizeOf)]
pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default + Representable> {
    phantom: PhantomData<T>,

    #[ignore_malloc_size_of = "state of environment is negligible compared to the actual maps (which are non disk)"]
    env: Env,

    path: String,

    #[ignore_malloc_size_of = "state of RNG is negligible compared to the actual maps (which are non disk)"]
    rng: SmallRng,
}

impl<T: Ord + Hash + MallocSizeOf + Default + Representable> AnnoStorageImpl<T> {
    pub fn load_from_file(path: &str) -> Result<AnnoStorageImpl<T>> {
        let path = Path::new(path);
        // Use 100 MB (SI standard) as default size
        let env = Env::new(path, 100_000_000)?;
        let mut txn = env.mut_txn_begin()?;

        Self::create_non_existing_roots(&mut txn)?;

        txn.commit()?;

        Ok(AnnoStorageImpl {
            env,
            path: path.to_string_lossy().to_string(),
            rng: SmallRng::from_rng(rand::thread_rng())?,
            phantom: PhantomData::default(),
        })
    }

    fn create_non_existing_roots<A>(txn: &mut MutTxn<A>) -> Result<()> {
        // Map from the item to all its annotations (as a map with the name/namespace as key)
        let by_container: Option<ByContainerDb<T>> = txn.root(BY_CONTAINER_ID);
        if by_container.is_none() {
            let root: ByContainerDb<T> = txn.create_db()?;
            txn.set_root(BY_CONTAINER_ID, root);
        }

        // A map from an annotation key to a map of all its values to the items having this value for the annotation key
        let by_anno: Option<ByAnnoDb<T>> = txn.root(BY_ANNO_ID);
        if by_anno.is_none() {
            let root: ByAnnoDb<T> = txn.create_db()?;
            txn.set_root(BY_ANNO_ID, root);
        }

        Ok(())
    }

    fn unsafe_value_to_string(txn : &Txn, unsafe_value : UnsafeValue) -> String {
        let value = unsafe {sanakirja::value::Value::from_unsafe(&unsafe_value, txn)};
        let value_bytes = value.into_cow();

        std::string::String::from_utf8_lossy(&value_bytes).to_string()

    }


    fn insert_transcational(&mut self, item: T, anno: Annotation) -> std::result::Result<(), sanakirja::Error> {
        let mut txn = self.env.mut_txn_begin()?;

        let by_container: Option<ByContainerDb<T>> = txn.root(BY_CONTAINER_ID);
        let by_anno: Option<ByAnnoDb<T>> = txn.root(BY_ANNO_ID);

        if let (Some(mut by_container), Some(mut by_anno)) = (by_container, by_anno) {
            let name = UnsafeValue::alloc_if_needed(&mut txn, anno.key.name.as_bytes())?;
            let namespace = UnsafeValue::alloc_if_needed(&mut txn, anno.key.ns.as_bytes())?;
            let val = UnsafeValue::alloc_if_needed(&mut txn, anno.val.as_bytes())?;

            // try to get an existing kv for all annotations of this item or create a new one
            let mut annotations: AnnotationsDb =
                if let Some(existing) = txn.get(&by_container, item, None) {
                    existing
                } else {
                    let created_annotations_db = txn.create_db()?;
                    txn.put(
                        &mut self.rng,
                        &mut by_container,
                        item,
                        created_annotations_db,
                    )?;
                    created_annotations_db
                };
            // add the annotation value to the corresponding name/namespace pair
            txn.put(&mut self.rng, &mut annotations, (name, namespace), val)?;

            // add item to the by_anno map and also create the values kv if not yet existing
            let mut values_for_anno: ValuesDb<T> =
                if let Some(existing) = txn.get(&by_anno, (name, namespace), None) {
                    existing
                } else {
                    let created_values_db = txn.create_db()?;
                    txn.put(
                        &mut self.rng,
                        &mut by_anno,
                        (name, namespace),
                        created_values_db,
                    )?;
                    created_values_db
                };
            txn.put(&mut self.rng, &mut values_for_anno, val, item)?;
        }

        txn.commit()?;
        Ok(())
    }

    fn clear_internal(&mut self) -> Result<()> {
        let mut rng = SmallRng::from_rng(rand::thread_rng())?;
        let mut txn = self.env.mut_txn_begin()?;

        // drop the existing DBs
        let by_container: Option<ByContainerDb<T>> = txn.root(BY_CONTAINER_ID);
        if let Some(by_container) = by_container {
            txn.drop(&mut rng, &by_container)?;
        }

        let by_anno: Option<ByAnnoDb<T>> = txn.root(BY_ANNO_ID);
        if let Some(by_anno) = by_anno {
            txn.drop(&mut rng, &by_anno)?;
        }

        // re-create the dropped DBs
        Self::create_non_existing_roots(&mut txn)?;

        txn.commit()?;
        Ok(())
    }

}

impl<'de, T> AnnotationStorage<T> for AnnoStorageImpl<T>
where
    T: Ord + Hash + MallocSizeOf + Default + Clone + Representable + Send + Sync,
    (T, AnnoKeyID): Into<Match>,
{
    fn insert(&mut self, item: T, anno: Annotation) {
        loop {
                            
            match self.insert_transcational(item, anno.clone()) {
                Ok(_) => {
                    return;
                }
                Err(sanakirja::Error::NotEnoughSpace) => {
                    // do nothing, reach end of loop to execute extension code
                }
                Err(e) => panic!("Could not insert item to annotation storage: {:?}", e),
            }
            
            // load a resized memory mapped file
            let old_size = self.env.size();
            let path = Path::new(&self.path);
            self.env = Env::new(path, old_size * 2).expect("Could enlarge file for annotation storage");
        }
    }


    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation> {
        let txn = self.env.txn_begin().expect("Could not create transaction");
        let by_container: Option<ByContainerDb<T>> = txn.root(BY_CONTAINER_ID);
        
        if let Some(by_container) = by_container {
            let annotations : Option<AnnotationsDb> = txn.get(&by_container, item.clone(), None);
            if let Some(annotations) = annotations {
                let mut result = Vec::default();
                for ((name, ns), val) in txn.iter(&annotations, None) {
                    result.push(Annotation {
                        key: AnnoKey {
                            name: Self::unsafe_value_to_string(&txn, name),
                            ns: Self::unsafe_value_to_string(&txn, ns),
                        },
                        val: Self::unsafe_value_to_string(&txn, val),
                    })
                }
                return result;
            }
        }

        return vec![];
    }

    fn remove_annotation_for_item(&mut self, _item: &T, _key: &AnnoKey) -> Option<String> {
        unimplemented!()
    }

    fn clear(&mut self) {
        if let Err(e) = self.clear_internal() {
            error!("Could not clear node annotation storage: {}", e);
        }
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
