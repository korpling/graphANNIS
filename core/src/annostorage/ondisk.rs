use crate::annostorage::symboltable::SymbolTable;
use crate::annostorage::AnnotationStorage;
use crate::annostorage::{Match, ValueSearch};
use crate::errors::Result;
use crate::serializer::{FixedSizeKeySerializer, KeySerializer};
use crate::types::{AnnoKey, Annotation, NodeID};
use crate::util::disk_collections::{DiskMap, EvictionStrategy, DEFAULT_BLOCK_CACHE_CAPACITY};
use crate::util::{self, memory_estimation};
use core::ops::Bound::*;
use itertools::Itertools;
use rand::seq::IteratorRandom;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use transient_btree_index::BtreeConfig;

use smartstring::alias::String as SmartString;

pub const SUBFOLDER_NAME: &str = "nodes_diskmap_v1";

const UTF_8_MSG: &str = "String must be valid UTF-8 but was corrupted";

const KB: usize = 1 << 10;
const MB: usize = KB * KB;

const EVICTION_STRATEGY: EvictionStrategy = EvictionStrategy::MaximumBytes(512 * MB);

/// An on-disk implementation of an annotation storage.
///
/// # Panics
///
/// In contrast to the main-memory implementation, accessing the disk can fail.
/// This is handled as a fatal error with panic except for specific scenarios where we know how to recover from this error.
/// Panics are used because these errors are unrecoverable
/// (e.g. if the file is suddenly missing this is like if someone removed the main memory)
/// and there is no way of delivering a correct answer.
/// Retrying the same query again will also not succeed since temporary errors are already handled internally.
#[derive(MallocSizeOf)]
pub struct AnnoStorageImpl<T>
where
    T: FixedSizeKeySerializer
        + Send
        + Sync
        + malloc_size_of::MallocSizeOf
        + Clone
        + serde::ser::Serialize
        + serde::de::DeserializeOwned,
{
    #[ignore_malloc_size_of = "is stored on disk"]
    by_container: DiskMap<Vec<u8>, String>,
    #[ignore_malloc_size_of = "is stored on disk"]
    by_anno_qname: DiskMap<Vec<u8>, bool>,
    #[with_malloc_size_of_func = "memory_estimation::size_of_pathbuf"]
    location: PathBuf,
    /// A handle to a temporary directory. This must be part of the struct because the temporary directory will
    /// be deleted when this handle is dropped.
    #[with_malloc_size_of_func = "memory_estimation::size_of_option_tempdir"]
    temp_dir: Option<tempfile::TempDir>,

    anno_key_symbols: SymbolTable<AnnoKey>,

    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    anno_key_sizes: BTreeMap<AnnoKey, usize>,

    /// additional statistical information
    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    histogram_bounds: BTreeMap<AnnoKey, Vec<String>>,
    largest_item: Option<T>,

    phantom: std::marker::PhantomData<T>,
}

/// Creates a key for the `by_container` tree.
///
/// Structure:
/// ```text
/// [x Bits item ID][64 Bits symbol ID]
/// ```
fn create_by_container_key<T: FixedSizeKeySerializer>(item: T, anno_key_symbol: usize) -> Vec<u8> {
    let mut result: Vec<u8> = item.create_key().to_vec();
    result.extend(anno_key_symbol.create_key());
    result
}

/// Creates a key for the `by_anno_qname` tree.
///
/// Since the same (name, ns, value) triple can be used by multiple nodes and we want to avoid
/// arrays as values, the node ID is part of the key and makes it unique.
///
/// Structure:
/// ```text
/// [64 Bits Annotation Key Symbol][Value]\0[x Bits item ID]
/// ```
fn create_by_anno_qname_key<T: FixedSizeKeySerializer>(
    item: T,
    anno_key_symbol: usize,
    anno_value: &str,
) -> Vec<u8> {
    // Use the qualified annotation name, the value and the node ID as key for the indexes.
    let mut result: Vec<u8> = anno_key_symbol.create_key().to_vec();
    for b in anno_value.as_bytes() {
        result.push(*b);
    }
    result.push(0);
    let item_key: &[u8] = &item.create_key();
    result.extend(item_key);
    result
}

impl<T> AnnoStorageImpl<T>
where
    T: FixedSizeKeySerializer
        + Send
        + Sync
        + malloc_size_of::MallocSizeOf
        + Clone
        + Default
        + serde::ser::Serialize
        + serde::de::DeserializeOwned,
    (T, Arc<AnnoKey>): Into<Match>,
{
    pub fn new(path: Option<PathBuf>) -> Result<AnnoStorageImpl<T>> {
        if let Some(path) = path {
            let path_by_container = path.join("by_container.bin");
            let path_by_anno_qname = path.join("by_anno_qname.bin");

            let mut result = AnnoStorageImpl {
                by_container: DiskMap::new(
                    Some(&path_by_container),
                    EVICTION_STRATEGY,
                    DEFAULT_BLOCK_CACHE_CAPACITY,
                    BtreeConfig::default().fixed_key_size(T::key_size() + 16),
                )?,
                by_anno_qname: DiskMap::new(
                    Some(&path_by_anno_qname),
                    EVICTION_STRATEGY,
                    DEFAULT_BLOCK_CACHE_CAPACITY,
                    BtreeConfig::default(),
                )?,
                anno_key_symbols: SymbolTable::default(),
                anno_key_sizes: BTreeMap::new(),
                largest_item: None,
                histogram_bounds: BTreeMap::new(),
                location: path.clone(),
                temp_dir: None,
                phantom: std::marker::PhantomData,
            };

            // load internal helper fields
            let custom_path = path.join("custom.bin");
            let f = std::fs::File::open(custom_path)?;
            let mut reader = std::io::BufReader::new(f);
            result.largest_item = bincode::deserialize_from(&mut reader)?;
            result.anno_key_sizes = bincode::deserialize_from(&mut reader)?;
            result.histogram_bounds = bincode::deserialize_from(&mut reader)?;
            result.anno_key_symbols = bincode::deserialize_from(&mut reader)?;
            result.anno_key_symbols.after_deserialization();

            Ok(result)
        } else {
            let tmp_dir = tempfile::Builder::new()
                .prefix("graphannis-ondisk-nodeanno-")
                .tempdir()?;
            Ok(AnnoStorageImpl {
                by_container: DiskMap::new_temporary(
                    EVICTION_STRATEGY,
                    DEFAULT_BLOCK_CACHE_CAPACITY,
                    BtreeConfig::default().fixed_key_size(T::key_size() + 16),
                ),
                by_anno_qname: DiskMap::new_temporary(
                    EVICTION_STRATEGY,
                    DEFAULT_BLOCK_CACHE_CAPACITY,
                    BtreeConfig::default(),
                ),
                anno_key_symbols: SymbolTable::default(),
                anno_key_sizes: BTreeMap::new(),
                largest_item: None,
                histogram_bounds: BTreeMap::new(),
                location: tmp_dir.as_ref().to_path_buf(),
                temp_dir: Some(tmp_dir),
                phantom: std::marker::PhantomData,
            })
        }
    }

    fn matching_items<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: Option<&str>,
    ) -> Box<dyn Iterator<Item = Result<(T, Arc<AnnoKey>)>> + 'a>
    where
        T: FixedSizeKeySerializer + Send + Sync + malloc_size_of::MallocSizeOf + PartialOrd,
    {
        let key_ranges: Vec<Arc<AnnoKey>> = if let Some(ns) = namespace {
            vec![Arc::from(AnnoKey {
                ns: ns.into(),
                name: name.into(),
            })]
        } else {
            self.get_qnames(name).into_iter().map(Arc::from).collect()
        };

        let value = value.map(|v| v.to_string());

        let it = key_ranges
            .into_iter()
            .filter_map(move |k| self.anno_key_symbols.get_symbol(&k))
            .flat_map(move |anno_key_symbol| {
                let lower_bound_value = if let Some(value) = &value { value } else { "" };
                let lower_bound = create_by_anno_qname_key(
                    NodeID::min_value(),
                    anno_key_symbol,
                    lower_bound_value,
                );

                let upper_bound_value = if let Some(value) = &value {
                    Cow::Borrowed(value)
                } else {
                    Cow::Owned(std::char::MAX.to_string())
                };

                let upper_bound = create_by_anno_qname_key(
                    NodeID::max_value(),
                    anno_key_symbol,
                    &upper_bound_value,
                );
                self.by_anno_qname.range(lower_bound..upper_bound)
            })
            .fuse()
            .map_ok(move |(data, _)| {
                // get the item ID at the end
                let item_id = T::parse_key(&data[data.len() - T::key_size()..]);
                let anno_key_symbol = usize::parse_key(&data[0..std::mem::size_of::<usize>()]);
                let key = self
                    .anno_key_symbols
                    .get_value(anno_key_symbol)
                    .unwrap_or_default();
                (item_id, key)
            });

        Box::new(it)
    }

    /// Parse the raw data and extract the item ID and the annotation key.
    ///
    /// # Panics
    /// Panics if the raw data is smaller than the length of a item ID bit-representation.
    fn parse_by_container_key(&self, data: Vec<u8>) -> (T, Arc<AnnoKey>) {
        let item = T::parse_key(&data[0..T::key_size()]);
        let anno_key_symbol = usize::parse_key(&data[T::key_size()..]);

        (
            item,
            self.anno_key_symbols
                .get_value(anno_key_symbol)
                .unwrap_or_default(),
        )
    }

    /// Parse the raw data and extract the node ID and the annotation.
    ///
    /// # Panics
    /// Panics if the raw data is smaller than the length of a node ID bit-representation or if the strings are not valid
    /// UTF-8.
    fn parse_by_anno_qname_key(&self, mut data: Vec<u8>) -> (T, Arc<AnnoKey>, String) {
        // get the item ID at the end
        let item_id_raw = data.split_off(data.len() - T::key_size());
        let item_id = T::parse_key(&item_id_raw);

        // remove the trailing '\0' character
        data.pop();

        // split off the annotation value string
        let anno_val_raw = data.split_off(std::mem::size_of::<usize>());
        let anno_val = String::from_utf8(anno_val_raw).expect(UTF_8_MSG);

        // parse the remaining annotation key symbol
        let anno_key_symbol = usize::parse_key(&data);

        (
            item_id,
            self.anno_key_symbols
                .get_value(anno_key_symbol)
                .unwrap_or_default(),
            anno_val,
        )
    }

    fn get_by_anno_qname_range<'a>(
        &'a self,
        anno_key: &AnnoKey,
    ) -> Box<dyn Iterator<Item = Result<(Vec<u8>, bool)>> + 'a> {
        if let Some(anno_key_symbol) = self.anno_key_symbols.get_symbol(anno_key) {
            let lower_bound = create_by_anno_qname_key(NodeID::min_value(), anno_key_symbol, "");

            let upper_bound = create_by_anno_qname_key(
                NodeID::max_value(),
                anno_key_symbol,
                &std::char::MAX.to_string(),
            );

            Box::new(self.by_anno_qname.range(lower_bound..upper_bound))
        } else {
            Box::from(std::iter::empty())
        }
    }
}

impl<'de, T> AnnotationStorage<T> for AnnoStorageImpl<T>
where
    T: FixedSizeKeySerializer
        + Send
        + Sync
        + malloc_size_of::MallocSizeOf
        + PartialOrd
        + Clone
        + Default
        + serde::ser::Serialize
        + serde::de::DeserializeOwned,
    (T, Arc<AnnoKey>): Into<Match>,
{
    fn insert(&mut self, item: T, anno: Annotation) -> Result<()> {
        // make sure the symbol ID for this annotation key is created
        let anno_key_symbol = self.anno_key_symbols.insert(anno.key.clone());

        // insert the value into main tree
        let by_container_key = create_by_container_key(item.clone(), anno_key_symbol);

        // Check if the item already exists. This needs to access the disk tables,
        // so avoid the check if we already know the new item is larger than the previous largest
        // item and thus can't exist yet.
        let item_smaller_than_largest = self
            .largest_item
            .as_ref()
            .map_or(true, |largest_item| item <= *largest_item);
        let already_existed =
            item_smaller_than_largest && self.by_container.contains_key(&by_container_key)?;
        self.by_container
            .insert(by_container_key, anno.val.clone().into())?;

        // To save some space, insert an boolean value as a marker value
        // (all information is part of the key already)
        self.by_anno_qname.insert(
            create_by_anno_qname_key(item.clone(), anno_key_symbol, &anno.val),
            true,
        )?;

        if !already_existed {
            // a new annotation entry was inserted and did not replace an existing one
            if let Some(largest_item) = self.largest_item.clone() {
                if largest_item < item {
                    self.largest_item = Some(item);
                }
            } else {
                self.largest_item = Some(item);
            }

            let anno_key_entry = self.anno_key_sizes.entry(anno.key).or_insert(0);
            *anno_key_entry += 1;
        }

        Ok(())
    }

    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation> {
        let mut result = Vec::default();
        let start = create_by_container_key(item.clone(), usize::min_value());
        let end = create_by_container_key(item.clone(), usize::max_value());
        for anno in self.by_container.range(start..=end) {
            let (key, val) = anno.expect("Iterator over annotations returned error");
            let parsed_key = self.parse_by_container_key(key);
            let anno = Annotation {
                key: parsed_key.1.as_ref().clone(),
                val: val.into(),
            };
            result.push(anno);
        }

        result
    }

    fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Result<Option<Cow<str>>> {
        // remove annotation from by_container
        if let Some(symbol_id) = self.anno_key_symbols.get_symbol(key) {
            let by_container_key = create_by_container_key(item.clone(), symbol_id);
            if let Some(val) = self.by_container.remove(&by_container_key)? {
                // remove annotation from by_anno_qname
                let anno = Annotation {
                    key: key.clone(),
                    val: val.into(),
                };

                self.by_anno_qname.remove(&create_by_anno_qname_key(
                    item.clone(),
                    symbol_id,
                    &anno.val,
                ))?;
                // decrease the annotation count for this key
                let new_key_count: usize =
                    if let Some(num_of_keys) = self.anno_key_sizes.get_mut(key) {
                        *num_of_keys -= 1;
                        *num_of_keys
                    } else {
                        0
                    };
                // if annotation count dropped to zero remove the key
                if new_key_count == 0 {
                    self.anno_key_sizes.remove(key);
                    if let Some(id) = self.anno_key_symbols.get_symbol(key) {
                        self.anno_key_symbols.remove(id);
                    }
                }

                return Ok(Some(Cow::Owned(anno.val.into())));
            }
        }
        Ok(None)
    }

    fn clear(&mut self) -> Result<()> {
        self.by_container.clear();
        self.by_anno_qname.clear();

        self.largest_item = None;
        self.anno_key_sizes.clear();
        self.histogram_bounds.clear();

        Ok(())
    }

    fn get_qnames(&self, name: &str) -> Vec<AnnoKey> {
        let it = self.anno_key_sizes.range(
            AnnoKey {
                name: name.into(),
                ns: SmartString::default(),
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

    fn number_of_annotations(&self) -> Result<usize> {
        let result = self.by_container.iter()?.count();
        Ok(result)
    }

    fn is_empty(&self) -> Result<bool> {
        self.by_container.is_empty()
    }

    fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Result<Option<Cow<str>>> {
        if let Some(symbol_id) = self.anno_key_symbols.get_symbol(key) {
            let raw = self
                .by_container
                .get(&create_by_container_key(item.clone(), symbol_id))?;
            if let Some(val) = raw {
                return match val {
                    Cow::Borrowed(val) => Ok(Some(Cow::Borrowed(val.as_str()))),
                    Cow::Owned(val) => Ok(Some(Cow::Owned(val))),
                };
            }
        }
        Ok(None)
    }

    fn has_value_for_item(&self, item: &T, key: &AnnoKey) -> Result<bool> {
        if let Some(symbol_id) = self.anno_key_symbols.get_symbol(key) {
            let result = self
                .by_container
                .contains_key(&create_by_container_key(item.clone(), symbol_id))?;
            Ok(result)
        } else {
            Ok(false)
        }
    }

    fn get_keys_for_iterator<'b>(
        &'b self,
        ns: Option<&str>,
        name: Option<&str>,
        it: Box<
            dyn Iterator<Item = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>>
                + 'b,
        >,
    ) -> Result<Vec<Match>> {
        if let Some(name) = name {
            if let Some(ns) = ns {
                // return the only possible annotation for each node
                let key = Arc::from(AnnoKey {
                    ns: ns.into(),
                    name: name.into(),
                });
                let mut matches = Vec::new();
                if let Some(symbol_id) = self.anno_key_symbols.get_symbol(&key) {
                    // create a template key
                    let mut container_key = create_by_container_key(T::default(), symbol_id);
                    for item in it {
                        let item = item?;
                        // Set the first bytes to the ID of the item.
                        // This saves the repeated expensive construction of the annotation key part.
                        container_key[0..T::key_size()].copy_from_slice(&item.create_key());
                        let does_contain_key = self.by_container.contains_key(&container_key)?;
                        if does_contain_key {
                            matches.push((item, key.clone()).into());
                        }
                    }
                }
                Ok(matches)
            } else {
                let mut matching_qnames: Vec<(Vec<u8>, Arc<AnnoKey>)> = self
                    .get_qnames(name)
                    .into_iter()
                    .filter_map(|key| {
                        if let Some(symbol_id) = self.anno_key_symbols.get_symbol(&key) {
                            let serialized_key = create_by_container_key(T::default(), symbol_id);
                            Some((serialized_key, Arc::from(key)))
                        } else {
                            None
                        }
                    })
                    .collect();
                // return all annotations with the correct name for each node
                let mut matches = Vec::new();
                for item in it {
                    let item = item?;
                    for (container_key, anno_key) in matching_qnames.iter_mut() {
                        // Set the first bytes to the ID of the item.
                        // This saves the repeated expensive construction of the annotation key part.
                        container_key[0..T::key_size()].copy_from_slice(&item.create_key());
                        let does_contain_key = self.by_container.contains_key(container_key)?;
                        if does_contain_key {
                            matches.push((item.clone(), anno_key.clone()).into());
                        }
                    }
                }
                Ok(matches)
            }
        } else {
            // get all annotation keys for this item
            let matches: Result<Vec<_>> = it
                .map_ok(|item| {
                    let start = create_by_container_key(item.clone(), usize::min_value());
                    let end = create_by_container_key(item, usize::max_value());

                    self.by_container
                        .range(start..=end)
                        .map(|anno| anno.expect("Iterator over annotations returned error"))
                        .map(|(data, _)| self.parse_by_container_key(data).into())
                })
                .flatten_ok()
                .map(|item| item.map_err(|e| e.into()))
                .collect();
            matches
        }
    }

    fn number_of_annotations_by_name(&self, ns: Option<&str>, name: &str) -> Result<usize> {
        let qualified_keys = match ns {
            Some(ns) => self.anno_key_sizes.range((
                Included(AnnoKey {
                    name: name.into(),
                    ns: ns.into(),
                }),
                Included(AnnoKey {
                    name: name.into(),
                    ns: ns.into(),
                }),
            )),
            None => self.anno_key_sizes.range(
                AnnoKey {
                    name: name.into(),
                    ns: SmartString::default(),
                }..AnnoKey {
                    name: name.into(),
                    ns: std::char::MAX.to_string().into(),
                },
            ),
        };
        let mut result = 0;
        for (_anno_key, anno_size) in qualified_keys {
            result += anno_size;
        }
        Ok(result)
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: ValueSearch<&str>,
    ) -> Box<dyn Iterator<Item = Result<Match>> + 'a> {
        match value {
            ValueSearch::Any => {
                let it = self
                    .matching_items(namespace, name, None)
                    .map_ok(|item| item.into());
                Box::new(it)
            }
            ValueSearch::Some(value) => {
                let it = self
                    .matching_items(namespace, name, Some(value))
                    .map_ok(|item| item.into());
                Box::new(it)
            }
            ValueSearch::NotSome(value) => {
                let value = value.to_string();
                let it = self
                    .matching_items(namespace, name, None)
                    .map(move |item| match item {
                        Ok((node, anno_key)) => {
                            let value = self.get_value_for_item(&node, &anno_key)?;
                            Ok((node, anno_key, value))
                        }
                        Err(e) => Err(e),
                    })
                    .filter_ok(move |(_, _, item_value)| {
                        if let Some(item_value) = item_value {
                            item_value != &value
                        } else {
                            false
                        }
                    })
                    .map_ok(move |(node, anno_key, _)| (node, anno_key).into());
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
    ) -> Box<dyn Iterator<Item = Result<Match>> + 'a> {
        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if let Ok(re) = compiled_result {
            let it = self
                .matching_items(namespace, name, None)
                .map(move |item| match item {
                    Ok((node, anno_key)) => {
                        let value = self.get_value_for_item(&node, &anno_key)?;
                        Ok((node, anno_key, value))
                    }
                    Err(e) => Err(e),
                })
                .filter_ok(move |(_, _, val)| {
                    if let Some(val) = val {
                        if negated {
                            !re.is_match(&val)
                        } else {
                            re.is_match(&val)
                        }
                    } else {
                        false
                    }
                })
                .map_ok(move |(node, anno_key, _val)| (node, anno_key).into());
            Box::new(it)
        } else if negated {
            // return all values
            self.exact_anno_search(namespace, name, None.into())
        } else {
            // if regular expression pattern is invalid return empty iterator
            Box::new(std::iter::empty())
        }
    }

    fn get_all_keys_for_item(
        &self,
        item: &T,
        ns: Option<&str>,
        name: Option<&str>,
    ) -> Result<Vec<Arc<AnnoKey>>> {
        if let Some(name) = name {
            if let Some(ns) = ns {
                let key = Arc::from(AnnoKey {
                    ns: ns.into(),
                    name: name.into(),
                });
                if let Some(symbol_id) = self.anno_key_symbols.get_symbol(&key) {
                    let does_contain_key = self
                        .by_container
                        .contains_key(&create_by_container_key(item.clone(), symbol_id))?;
                    if does_contain_key {
                        return Ok(vec![key.clone()]);
                    }
                }
                Ok(vec![])
            } else {
                // get all qualified names for the given annotation name
                let res: Result<Vec<Arc<AnnoKey>>> = self
                    .get_qnames(name)
                    .into_iter()
                    .map(|key| {
                        if let Some(symbol_id) = self.anno_key_symbols.get_symbol(&key) {
                            let does_contain_key = self
                                .by_container
                                .contains_key(&create_by_container_key(item.clone(), symbol_id))?;
                            Ok((does_contain_key, key))
                        } else {
                            Ok((false, key))
                        }
                    })
                    .filter_ok(|(does_contain_key, _)| *does_contain_key)
                    .map_ok(|(_, key)| Arc::from(key))
                    .collect();
                let res = res?;
                Ok(res)
            }
        } else {
            // no annotation name given, return all
            let result = self
                .get_annotations_for_item(item)
                .into_iter()
                .map(|anno| Arc::from(anno.key))
                .collect();
            Ok(result)
        }
    }

    fn guess_max_count(
        &self,
        ns: Option<&str>,
        name: &str,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey {
                name: name.into(),
                ns: ns.into(),
            }],
            None => self.get_qnames(name),
        };

        let mut universe_size: usize = 0;
        let mut sum_histogram_buckets: usize = 0;
        let mut count_matches: usize = 0;

        // guess for each fully qualified annotation key and return the sum of all guesses
        for anno_key in qualified_keys {
            if let Some(anno_size) = self.anno_key_sizes.get(&anno_key) {
                universe_size += *anno_size;

                if let Some(histo) = self.histogram_bounds.get(&anno_key) {
                    // find the range in which the value is contained

                    // we need to make sure the histogram is not empty -> should have at least two bounds
                    if histo.len() >= 2 {
                        sum_histogram_buckets += histo.len() - 1;

                        for i in 0..histo.len() - 1 {
                            let bucket_begin = &histo[i];
                            let bucket_end = &histo[i + 1];
                            // check if the range overlaps with the search range
                            if bucket_begin.as_str() <= upper_val
                                && lower_val <= bucket_end.as_str()
                            {
                                count_matches += 1;
                            }
                        }
                    }
                }
            }
        }

        if sum_histogram_buckets > 0 {
            let selectivity: f64 = (count_matches as f64) / (sum_histogram_buckets as f64);
            (selectivity * (universe_size as f64)).round() as usize
        } else {
            0
        }
    }

    fn guess_max_count_regex(&self, ns: Option<&str>, name: &str, pattern: &str) -> usize {
        let full_match_pattern = util::regex_full_match(pattern);

        let parsed = regex_syntax::Parser::new().parse(&full_match_pattern);
        if let Ok(parsed) = parsed {
            let expr: regex_syntax::hir::Hir = parsed;

            let prefix_set = regex_syntax::hir::literal::Literals::prefixes(&expr);
            let val_prefix = std::str::from_utf8(prefix_set.longest_common_prefix());

            if let Ok(lower_val) = val_prefix {
                let mut upper_val = String::from(lower_val);
                upper_val.push(std::char::MAX);
                return self.guess_max_count(ns, name, lower_val, &upper_val);
            }
        }

        0
    }

    fn guess_most_frequent_value(&self, ns: Option<&str>, name: &str) -> Option<Cow<str>> {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey {
                name: name.into(),
                ns: ns.into(),
            }],
            None => self.get_qnames(name),
        };

        let mut sampled_values: HashMap<&str, usize> = HashMap::default();

        // guess for each fully qualified annotation key
        for anno_key in qualified_keys {
            if let Some(histo) = self.histogram_bounds.get(&anno_key) {
                for v in histo.iter() {
                    let count: &mut usize = sampled_values.entry(v).or_insert(0);
                    *count += 1;
                }
            }
        }
        // find the value which is most frequent
        if !sampled_values.is_empty() {
            let mut max_count = 0;
            let mut max_value = Cow::Borrowed("");
            for (v, count) in sampled_values.into_iter() {
                if count >= max_count {
                    max_value = Cow::Borrowed(v);
                    max_count = count;
                }
            }
            Some(max_value)
        } else {
            None
        }
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<Cow<str>> {
        if most_frequent_first {
            let mut values_with_count: HashMap<String, usize> = HashMap::default();
            for (data, _) in self
                .get_by_anno_qname_range(key)
                .map(|item| item.expect("Iterator over items for annotation key returned error"))
            {
                let (_, _, val) = self.parse_by_anno_qname_key(data);

                let count = values_with_count.entry(val).or_insert(0);
                *count += 1;
            }
            let mut values_with_count: Vec<(usize, Cow<str>)> = values_with_count
                .into_iter()
                .map(|(val, count)| (count, Cow::Owned(val)))
                .collect();
            values_with_count.sort();
            values_with_count
                .into_iter()
                .map(|(_count, val)| val)
                .collect()
        } else {
            let values_unique: HashSet<Cow<str>> = self
                .get_by_anno_qname_range(key)
                .map(|item| item.expect("Iterator over items for annotation key returned error"))
                .map(|(data, _)| {
                    let (_, _, val) = self.parse_by_anno_qname_key(data);
                    Cow::Owned(val)
                })
                .collect();
            values_unique.into_iter().collect()
        }
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        self.anno_key_sizes.keys().cloned().collect()
    }

    fn get_largest_item(&self) -> Option<T> {
        self.largest_item.clone()
    }

    fn calculate_statistics(&mut self) {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separately
        for anno_key in self.anno_key_sizes.keys() {
            // sample a maximal number of annotation values
            let mut rng = rand::thread_rng();

            let all_values_for_key = self.get_by_anno_qname_range(anno_key);

            let mut sampled_anno_values: Vec<String> = all_values_for_key
                .choose_multiple(&mut rng, max_sampled_annotations)
                .into_iter()
                .map(|data| {
                    let (data, _) =
                        data.expect("Iterator over items for annotation key returned error");
                    let (_, _, val) = self.parse_by_anno_qname_key(data);
                    val
                })
                .collect();

            // create uniformly distributed histogram bounds
            sampled_anno_values.sort();

            let num_hist_bounds = if sampled_anno_values.len() < (max_histogram_buckets + 1) {
                sampled_anno_values.len()
            } else {
                max_histogram_buckets + 1
            };

            let hist = self
                .histogram_bounds
                .entry(anno_key.clone())
                .or_insert_with(std::vec::Vec::new);

            if num_hist_bounds >= 2 {
                hist.resize(num_hist_bounds, String::from(""));

                let delta: usize = (sampled_anno_values.len() - 1) / (num_hist_bounds - 1);
                let delta_fraction: usize = (sampled_anno_values.len() - 1) % (num_hist_bounds - 1);

                let mut pos = 0;
                let mut pos_fraction = 0;
                for hist_item in hist.iter_mut() {
                    *hist_item = sampled_anno_values[pos].clone();
                    pos += delta;
                    pos_fraction += delta_fraction;

                    if pos_fraction >= (num_hist_bounds - 1) {
                        pos += 1;
                        pos_fraction -= num_hist_bounds - 1;
                    }
                }
            }
        }
    }

    fn load_annotations_from(&mut self, location: &Path) -> Result<()> {
        let location = location.join(SUBFOLDER_NAME);

        if !self.location.eq(&location) {
            self.by_container = DiskMap::new(
                Some(&location.join("by_container.bin")),
                EVICTION_STRATEGY,
                DEFAULT_BLOCK_CACHE_CAPACITY,
                BtreeConfig::default().fixed_value_size(T::key_size() + 9),
            )?;
            self.by_anno_qname = DiskMap::new(
                Some(&location.join("by_anno_qname.bin")),
                EVICTION_STRATEGY,
                DEFAULT_BLOCK_CACHE_CAPACITY,
                BtreeConfig::default(),
            )?;
        }

        // load internal helper fields
        let f = std::fs::File::open(location.join("custom.bin"))?;
        let mut reader = std::io::BufReader::new(f);
        self.largest_item = bincode::deserialize_from(&mut reader)?;
        self.anno_key_sizes = bincode::deserialize_from(&mut reader)?;
        self.histogram_bounds = bincode::deserialize_from(&mut reader)?;
        self.anno_key_symbols = bincode::deserialize_from(&mut reader)?;
        self.anno_key_symbols.after_deserialization();

        Ok(())
    }

    fn save_annotations_to(&self, location: &Path) -> Result<()> {
        let location = location.join(SUBFOLDER_NAME);

        // write out the disk maps to a single sorted string table
        self.by_container
            .write_to(&location.join("by_container.bin"))?;
        self.by_anno_qname
            .write_to(&location.join("by_anno_qname.bin"))?;

        // save the other custom fields
        let f = std::fs::File::create(location.join("custom.bin"))?;
        let mut writer = std::io::BufWriter::new(f);
        bincode::serialize_into(&mut writer, &self.largest_item)?;
        bincode::serialize_into(&mut writer, &self.anno_key_sizes)?;
        bincode::serialize_into(&mut writer, &self.histogram_bounds)?;
        bincode::serialize_into(&mut writer, &self.anno_key_symbols)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Once;
    static LOGGER_INIT: Once = Once::new();

    #[test]
    fn insert_same_anno() {
        LOGGER_INIT.call_once(env_logger::init);

        let test_anno = Annotation {
            key: AnnoKey {
                name: "anno1".into(),
                ns: "annis".into(),
            },
            val: "test".into(),
        };

        let mut a = AnnoStorageImpl::new(None).unwrap();

        debug!("Inserting annotation for node 1");
        a.insert(1, test_anno.clone()).unwrap();
        debug!("Inserting annotation for node 1 (again)");
        a.insert(1, test_anno.clone()).unwrap();
        debug!("Inserting annotation for node 2");
        a.insert(2, test_anno.clone()).unwrap();
        debug!("Inserting annotation for node 3");
        a.insert(3, test_anno).unwrap();

        assert_eq!(3, a.number_of_annotations().unwrap());

        assert_eq!(
            "test",
            a.get_value_for_item(
                &3,
                &AnnoKey {
                    name: "anno1".into(),
                    ns: "annis".into()
                }
            )
            .unwrap()
            .unwrap()
        );
    }

    #[test]
    fn get_all_for_node() {
        LOGGER_INIT.call_once(env_logger::init);

        let test_anno1 = Annotation {
            key: AnnoKey {
                name: "anno1".into(),
                ns: "annis1".into(),
            },
            val: "test".into(),
        };
        let test_anno2 = Annotation {
            key: AnnoKey {
                name: "anno2".into(),
                ns: "annis2".into(),
            },
            val: "test".into(),
        };
        let test_anno3 = Annotation {
            key: AnnoKey {
                name: "anno3".into(),
                ns: "annis1".into(),
            },
            val: "test".into(),
        };

        let mut a = AnnoStorageImpl::new(None).unwrap();

        a.insert(1, test_anno1.clone()).unwrap();
        a.insert(1, test_anno2.clone()).unwrap();
        a.insert(1, test_anno3.clone()).unwrap();

        assert_eq!(3, a.number_of_annotations().unwrap());

        let mut all = a.get_annotations_for_item(&1);
        assert_eq!(3, all.len());

        all.sort_by(|a, b| a.key.partial_cmp(&b.key).unwrap());

        assert_eq!(test_anno1, all[0]);
        assert_eq!(test_anno2, all[1]);
        assert_eq!(test_anno3, all[2]);
    }

    #[test]
    fn remove() {
        LOGGER_INIT.call_once(env_logger::init);
        let test_anno = Annotation {
            key: AnnoKey {
                name: "anno1".into(),
                ns: "annis1".into(),
            },
            val: "test".into(),
        };

        let mut a = AnnoStorageImpl::new(None).unwrap();
        a.insert(1, test_anno.clone()).unwrap();

        assert_eq!(1, a.number_of_annotations().unwrap());
        assert_eq!(1, a.anno_key_sizes.len());
        assert_eq!(&1, a.anno_key_sizes.get(&test_anno.key).unwrap());

        a.remove_annotation_for_item(&1, &test_anno.key).unwrap();

        assert_eq!(0, a.number_of_annotations().unwrap());
        assert_eq!(&0, a.anno_key_sizes.get(&test_anno.key).unwrap_or(&0));
    }
}
