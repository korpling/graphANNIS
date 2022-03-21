use super::{AnnotationStorage, Match};
use crate::annostorage::ValueSearch;
use crate::errors::Result;
use crate::malloc_size_of::MallocSizeOf;
use crate::types::{AnnoKey, Annotation, Edge};
use crate::util::{self, memory_estimation};
use crate::{annostorage::symboltable::SymbolTable, errors::GraphAnnisCoreError};
use core::ops::Bound::*;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use smartstring::alias::String;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug, Default, MallocSizeOf, Copy)]
struct SparseAnnotation {
    key: usize,
    val: usize,
}

type ValueItemMap<T> = FxHashMap<usize, Vec<T>>;

#[derive(Serialize, Deserialize, Clone, Default, MallocSizeOf)]
pub struct AnnoStorageImpl<T: Ord + Hash + MallocSizeOf + Default> {
    by_container: FxHashMap<T, Vec<SparseAnnotation>>,
    /// A map from an annotation key symbol to a map of all its values to the items having this value for the annotation key
    by_anno: FxHashMap<usize, ValueItemMap<T>>,
    /// Maps a distinct annotation key to the number of elements having this annotation key.
    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    anno_key_sizes: BTreeMap<AnnoKey, usize>,
    anno_keys: SymbolTable<AnnoKey>,
    anno_values: SymbolTable<smartstring::alias::String>,

    /// additional statistical information
    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    histogram_bounds: BTreeMap<usize, Vec<smartstring::alias::String>>,
    largest_item: Option<T>,
    total_number_of_annos: usize,
}

impl<
        'de_impl,
        T: Ord
            + Hash
            + Clone
            + serde::Serialize
            + serde::de::DeserializeOwned
            + MallocSizeOf
            + Default,
    > AnnoStorageImpl<T>
{
    pub fn new() -> AnnoStorageImpl<T> {
        AnnoStorageImpl {
            by_container: FxHashMap::default(),
            by_anno: FxHashMap::default(),
            anno_keys: SymbolTable::new(),
            anno_values: SymbolTable::new(),
            anno_key_sizes: BTreeMap::new(),
            histogram_bounds: BTreeMap::new(),
            largest_item: None,
            total_number_of_annos: 0,
        }
    }

    fn clear_internal(&mut self) {
        self.by_container.clear();
        self.by_anno.clear();
        self.anno_keys.clear();
        self.anno_key_sizes.clear();
        self.histogram_bounds.clear();
        self.largest_item = None;
        self.anno_values.clear();
    }

    fn create_sparse_anno(&mut self, orig: Annotation) -> Result<SparseAnnotation> {
        let key = self.anno_keys.insert(orig.key)?;
        let val = self.anno_values.insert(orig.val)?;

        Ok(SparseAnnotation { key, val })
    }

    fn create_annotation_from_sparse(&self, orig: &SparseAnnotation) -> Option<Annotation> {
        let key = self.anno_keys.get_value_ref(orig.key)?;
        let val = self.anno_values.get_value_ref(orig.val)?;

        Some(Annotation {
            key: key.clone(),
            val: val.clone(),
        })
    }

    fn remove_element_from_by_anno(&mut self, anno: &SparseAnnotation, item: &T) {
        let remove_anno_key = if let Some(annos_for_key) = self.by_anno.get_mut(&anno.key) {
            let remove_anno_val = if let Some(items_for_anno) = annos_for_key.get_mut(&anno.val) {
                items_for_anno.retain(|i| i != item);
                items_for_anno.is_empty()
            } else {
                false
            };
            // remove the hash set of items for the original annotation if it empty
            if remove_anno_val {
                annos_for_key.remove(&anno.val);
                annos_for_key.is_empty()
            } else {
                false
            }
        } else {
            false
        };
        if remove_anno_key {
            self.by_anno.remove(&anno.key);
            // TODO: remove from symbol table?
        }
    }

    fn check_and_remove_value_symbol(&mut self, value_id: usize) {
        let mut still_used = false;
        for values in self.by_anno.values() {
            if values.contains_key(&value_id) {
                still_used = true;
                break;
            }
        }
        if !still_used {
            self.anno_values.remove(value_id);
        }
    }
}

impl<'de_impl, T> AnnoStorageImpl<T>
where
    T: Ord
        + Hash
        + MallocSizeOf
        + Default
        + Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + Send
        + Sync,
    (T, Arc<AnnoKey>): Into<Match>,
{
    fn matching_items<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: Option<&str>,
    ) -> Box<dyn Iterator<Item = Result<(T, Arc<AnnoKey>)>> + 'a> {
        let key_ranges: Vec<Arc<AnnoKey>> = if let Some(ns) = namespace {
            vec![Arc::from(AnnoKey {
                ns: ns.into(),
                name: name.into(),
            })]
        } else {
            let qnames = match self.get_qnames(name) {
                Ok(qnames) => qnames,
                Err(e) => return Box::new(std::iter::once(Err(e))),
            };
            qnames.into_iter().map(Arc::from).collect()
        };
        // Create a vector fore each matching AnnoKey to the value map containing all items and their annotation values
        // for this key.
        let value_maps: Vec<(Arc<AnnoKey>, &ValueItemMap<T>)> = key_ranges
            .into_iter()
            .filter_map(|key| {
                let key_id = self.anno_keys.get_symbol(&key)?;
                self.by_anno
                    .get(&key_id)
                    .map(|values_for_key| (key, values_for_key))
            })
            .collect();

        if let Some(value) = value {
            let target_value_symbol = self.anno_values.get_symbol(&value.into());

            if let Some(target_value_symbol) = target_value_symbol {
                let it = value_maps
                    .into_iter()
                    // find the items with the correct value
                    .filter_map(move |(key, values)| {
                        values.get(&target_value_symbol).map(|items| (items, key))
                    })
                    // flatten the hash set of all items, returns all items for the condition
                    .flat_map(|(items, key)| items.iter().cloned().zip(std::iter::repeat(key)))
                    .map(Ok);
                Box::new(it)
            } else {
                // value is not known, return empty result
                Box::new(std::iter::empty())
            }
        } else {
            let it = value_maps
                .into_iter()
                // flatten the hash set of all items of the value map
                .flat_map(|(key, values)| {
                    values
                        .iter()
                        .flat_map(|(_, items)| items.iter().cloned())
                        .zip(std::iter::repeat(key))
                })
                .map(Ok);
            Box::new(it)
        }
    }
}

impl<T> AnnotationStorage<T> for AnnoStorageImpl<T>
where
    T: Ord
        + Hash
        + MallocSizeOf
        + Default
        + Clone
        + Send
        + Sync
        + serde::Serialize
        + serde::de::DeserializeOwned,
    (T, Arc<AnnoKey>): Into<Match>,
{
    fn insert(&mut self, item: T, anno: Annotation) -> Result<()> {
        let orig_anno_key = anno.key.clone();
        let anno = self.create_sparse_anno(anno)?;

        let existing_anno = {
            let existing_item_entry = self
                .by_container
                .entry(item.clone())
                .or_insert_with(Vec::new);

            // check if there is already an item with the same annotation key
            let existing_entry_idx = existing_item_entry.binary_search_by_key(&anno.key, |a| a.key);

            if let Ok(existing_entry_idx) = existing_entry_idx {
                let orig_anno = existing_item_entry[existing_entry_idx];
                // abort if the same annotation key with the same value already exist
                if orig_anno.val == anno.val {
                    return Ok(());
                }
                // insert annotation for item at existing position
                existing_item_entry[existing_entry_idx] = anno;
                Some(orig_anno)
            } else if let Err(insertion_idx) = existing_entry_idx {
                // insert at sorted position -> the result will still be a sorted vector
                existing_item_entry.insert(insertion_idx, anno);
                None
            } else {
                None
            }
        };

        if let Some(ref existing_anno) = existing_anno {
            // remove the relation from the original annotation to this item
            self.remove_element_from_by_anno(existing_anno, &item);
        }

        // inserts a new relation between the annotation and the item
        // if set is not existing yet it is created
        self.by_anno
            .entry(anno.key)
            .or_insert_with(FxHashMap::default)
            .entry(anno.val)
            .or_insert_with(Vec::default)
            .push(item.clone());

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

            let anno_key_entry = self.anno_key_sizes.entry(orig_anno_key).or_insert(0);
            *anno_key_entry += 1;
        }

        Ok(())
    }

    fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Result<Option<Cow<str>>> {
        let mut result = None;

        let orig_key = key;
        if let Some(key) = self.anno_keys.get_symbol(key) {
            if let Some(mut all_annos) = self.by_container.remove(item) {
                // find the specific annotation key from the sorted vector of all annotations of this item
                let anno_idx = all_annos.binary_search_by_key(&key, |a| a.key);

                if let Ok(anno_idx) = anno_idx {
                    // since value was found, also remove the item from the other containers
                    self.remove_element_from_by_anno(&all_annos[anno_idx], item);

                    let old_value = all_annos[anno_idx].val;

                    // remove the specific annotation key from the entry
                    all_annos.remove(anno_idx);

                    // decrease the annotation count for this key
                    let new_key_count: usize =
                        if let Some(num_of_keys) = self.anno_key_sizes.get_mut(orig_key) {
                            *num_of_keys -= 1;
                            *num_of_keys
                        } else {
                            0
                        };
                    // if annotation count dropped to zero remove the key
                    if new_key_count == 0 {
                        self.by_anno.remove(&key);
                        self.anno_key_sizes.remove(orig_key);
                        self.anno_keys.remove(key);
                    }

                    result = self
                        .anno_values
                        .get_value_ref(old_value)
                        .map(|v| Cow::Owned(v.clone().into()));

                    self.check_and_remove_value_symbol(old_value);
                    self.total_number_of_annos -= 1;
                }
                // if there are more annotations for this item, re-insert them
                if !all_annos.is_empty() {
                    self.by_container.insert(item.clone(), all_annos);
                }
            }
        }

        Ok(result)
    }

    fn clear(&mut self) -> Result<()> {
        self.clear_internal();
        Ok(())
    }

    fn get_qnames(&self, name: &str) -> Result<Vec<AnnoKey>> {
        let it = self.anno_key_sizes.range(
            AnnoKey {
                name: name.into(),
                ns: smartstring::alias::String::default(),
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
        Ok(result)
    }

    fn get_annotations_for_item(&self, item: &T) -> Result<Vec<Annotation>> {
        if let Some(all_annos) = self.by_container.get(item) {
            let mut result: Vec<Annotation> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                if let Some(a) = self.create_annotation_from_sparse(a) {
                    result.push(a);
                }
            }
            return Ok(result);
        }
        // return empty result if not found
        Ok(Vec::new())
    }

    fn number_of_annotations(&self) -> Result<usize> {
        Ok(self.total_number_of_annos)
    }

    fn is_empty(&self) -> Result<bool> {
        Ok(self.total_number_of_annos == 0)
    }

    fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Result<Option<Cow<str>>> {
        if let (Some(key_symbol), Some(all_annos)) =
            (self.anno_keys.get_symbol(key), self.by_container.get(item))
        {
            let idx = all_annos.binary_search_by_key(&key_symbol, |a| a.key);
            if let Ok(idx) = idx {
                if let Some(val) = self.anno_values.get_value_ref(all_annos[idx].val) {
                    return Ok(Some(Cow::Borrowed(val)));
                }
            }
        }
        Ok(None)
    }

    fn has_value_for_item(&self, item: &T, key: &AnnoKey) -> Result<bool> {
        if let Some(key_symbol) = self.anno_keys.get_symbol(key) {
            if let Some(all_annos) = self.by_container.get(item) {
                if all_annos
                    .binary_search_by_key(&key_symbol, |a| a.key)
                    .is_ok()
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
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
                let mut matches = Vec::new();
                let key = Arc::from(AnnoKey {
                    ns: ns.into(),
                    name: name.into(),
                });

                if let Some(key_symbol) = self.anno_keys.get_symbol(&key) {
                    for item in it {
                        let item = item?;
                        if let Some(all_annos) = self.by_container.get(&item) {
                            if all_annos
                                .binary_search_by_key(&key_symbol, |a| a.key)
                                .is_ok()
                            {
                                matches.push((item, key.clone()).into());
                            }
                        }
                    }
                }
                Ok(matches)
            } else {
                let matching_key_symbols: Vec<(usize, Arc<AnnoKey>)> = self
                    .get_qnames(name)?
                    .into_iter()
                    .filter_map(|key| {
                        self.anno_keys
                            .get_symbol(&key)
                            .map(|key_symbol| (key_symbol, Arc::from(key)))
                    })
                    .collect();
                // return all annotations with the correct name for each node
                let mut matches = Vec::new();
                for item in it {
                    let item = item?;
                    for (key_symbol, key) in matching_key_symbols.iter() {
                        if let Some(all_annos) = self.by_container.get(&item) {
                            if all_annos
                                .binary_search_by_key(&key_symbol, |a| &a.key)
                                .is_ok()
                            {
                                matches.push((item.clone(), key.clone()).into());
                            }
                        }
                    }
                }
                Ok(matches)
            }
        } else {
            // return all annotations for each node
            let mut matches = Vec::new();
            for item in it {
                let item = item?;
                let all_keys = self.get_all_keys_for_item(&item, None, None)?;
                for anno_key in all_keys {
                    matches.push((item.clone(), anno_key).into());
                }
            }
            Ok(matches)
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
                    ns: String::default(),
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
        let key_ranges: Vec<Arc<AnnoKey>> = if let Some(ns) = namespace {
            vec![Arc::from(AnnoKey {
                ns: ns.into(),
                name: name.into(),
            })]
        } else {
            let qnames = match self.get_qnames(name) {
                Ok(qnames) => qnames,
                Err(e) => return Box::new(std::iter::once(Err(e))),
            };
            qnames.into_iter().map(Arc::from).collect()
        };
        // Create a vector for each matching AnnoKey to the value map containing all items and their annotation values
        // for this key.
        let value_maps: Vec<(Arc<AnnoKey>, &ValueItemMap<T>)> = key_ranges
            .into_iter()
            .filter_map(|key| {
                let key_id = self.anno_keys.get_symbol(&key)?;
                self.by_anno
                    .get(&key_id)
                    .map(|values_for_key| (key, values_for_key))
            })
            .collect();

        if let ValueSearch::Some(value) = value {
            let target_value_symbol = self.anno_values.get_symbol(&value.into());

            if let Some(target_value_symbol) = target_value_symbol {
                let it = value_maps
                    .into_iter()
                    // find the items with the correct value
                    .filter_map(move |(key, values)| {
                        values.get(&target_value_symbol).map(|items| (items, key))
                    })
                    // flatten the hash set of all items, returns all items for the condition
                    .flat_map(|(items, key)| items.iter().cloned().zip(std::iter::repeat(key)))
                    .map(move |item| Ok(item.into()));
                Box::new(it)
            } else {
                // value is not known, return empty result
                Box::new(std::iter::empty())
            }
        } else {
            // Search for all annotations having a matching qualified name, regardless of the value
            let matching_qname_annos = value_maps
                .into_iter()
                // flatten the hash set of all items of the value map
                .flat_map(|(key, values)| {
                    values
                        .iter()
                        .flat_map(|(_, items)| items.iter().cloned())
                        .zip(std::iter::repeat(key))
                });

            if let ValueSearch::NotSome(value) = value {
                let value = value.to_string();
                let it = matching_qname_annos
                    .map(move |(item, anno_key)| {
                        let value = self.get_value_for_item(&item, &anno_key)?;
                        Ok((item, anno_key, value))
                    })
                    .filter_ok(move |(_, _, item_value)| {
                        if let Some(item_value) = item_value {
                            item_value != &value
                        } else {
                            false
                        }
                    })
                    .map_ok(|(item, anno_key, _)| (item, anno_key).into());
                Box::new(it)
            } else {
                Box::new(matching_qname_annos.map(move |item| Ok(item.into())))
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
                .map(move |item| {
                    let (item, anno_key) = item?;
                    let value = self.get_value_for_item(&item, &anno_key)?;
                    Ok((item, anno_key, value))
                })
                .filter_ok(move |(_, _, value)| {
                    if let Some(val) = value {
                        if negated {
                            !re.is_match(val)
                        } else {
                            re.is_match(val)
                        }
                    } else {
                        false
                    }
                })
                .map_ok(move |(item, anno_key, _)| (item, anno_key).into());
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
                // fully qualified search
                let key = AnnoKey {
                    ns: ns.into(),
                    name: name.into(),
                };
                if let Some(key_symbol) = self.anno_keys.get_symbol(&key) {
                    if let Some(all_annos) = self.by_container.get(item) {
                        if all_annos
                            .binary_search_by_key(&key_symbol, |a| a.key)
                            .is_ok()
                        {
                            return Ok(vec![Arc::from(key)]);
                        }
                    }
                }

                Ok(vec![])
            } else {
                // get all qualified names for the given annotation name
                let res: Result<Vec<Arc<AnnoKey>>> = self
                    .get_qnames(name)?
                    .into_iter()
                    .map(|anno_key| {
                        let value = self.get_value_for_item(item, &anno_key)?;
                        Ok((anno_key, value))
                    })
                    .filter_ok(|(_key, value)| value.is_some())
                    .map_ok(|(key, _)| Arc::from(key))
                    .collect();
                res
            }
        } else if let Some(all_annos) = self.by_container.get(item) {
            // no annotation name given, return all
            let mut result: Vec<Arc<AnnoKey>> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                if let Some(key) = self.anno_keys.get_value(a.key) {
                    result.push(key);
                }
            }
            Ok(result)
        } else {
            // return empty result if not found
            Ok(vec![])
        }
    }

    fn guess_max_count(
        &self,
        ns: Option<&str>,
        name: &str,
        lower_val: &str,
        upper_val: &str,
    ) -> Result<usize> {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey {
                name: name.into(),
                ns: ns.into(),
            }],
            None => self.get_qnames(name)?,
        };

        let mut universe_size: usize = 0;
        let mut sum_histogram_buckets: usize = 0;
        let mut count_matches: usize = 0;

        // guess for each fully qualified annotation key and return the sum of all guesses
        for anno_key in qualified_keys {
            if let Some(anno_size) = self.anno_key_sizes.get(&anno_key) {
                universe_size += *anno_size;

                if let Some(anno_key) = self.anno_keys.get_symbol(&anno_key) {
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
        }

        if sum_histogram_buckets > 0 {
            let selectivity: f64 = (count_matches as f64) / (sum_histogram_buckets as f64);
            Ok((selectivity * (universe_size as f64)).round() as usize)
        } else {
            Ok(0)
        }
    }

    fn guess_max_count_regex(&self, ns: Option<&str>, name: &str, pattern: &str) -> Result<usize> {
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

        Ok(0)
    }

    fn guess_most_frequent_value(&self, ns: Option<&str>, name: &str) -> Result<Option<Cow<str>>> {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey {
                name: name.into(),
                ns: ns.into(),
            }],
            None => self.get_qnames(name)?,
        };

        let mut sampled_values: HashMap<&str, usize> = HashMap::default();

        // guess for each fully qualified annotation key
        for anno_key in qualified_keys {
            if let Some(anno_key) = self.anno_keys.get_symbol(&anno_key) {
                if let Some(histo) = self.histogram_bounds.get(&anno_key) {
                    for v in histo.iter() {
                        let count: &mut usize = sampled_values.entry(v).or_insert(0);
                        *count += 1;
                    }
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
            Ok(Some(max_value))
        } else {
            Ok(None)
        }
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Result<Vec<Cow<str>>> {
        if let Some(key) = self.anno_keys.get_symbol(key) {
            if let Some(values_for_key) = self.by_anno.get(&key) {
                if most_frequent_first {
                    let result = values_for_key
                        .iter()
                        .filter_map(|(val, items)| {
                            let val = self.anno_values.get_value_ref(*val)?;
                            Some((items.len(), val))
                        })
                        .sorted()
                        .rev()
                        .map(|(_, val)| Cow::Borrowed(&val[..]))
                        .collect();
                    return Ok(result);
                } else {
                    let result = values_for_key
                        .iter()
                        .filter_map(|(val, _items)| self.anno_values.get_value_ref(*val))
                        .map(|val| Cow::Borrowed(&val[..]))
                        .collect();
                    return Ok(result);
                }
            }
        }
        Ok(vec![])
    }

    fn annotation_keys(&self) -> Result<Vec<AnnoKey>> {
        Ok(self.anno_key_sizes.keys().cloned().collect())
    }

    fn get_largest_item(&self) -> Result<Option<T>> {
        Ok(self.largest_item.clone())
    }

    fn calculate_statistics(&mut self) -> Result<()> {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separately
        for anno_key in self.anno_key_sizes.keys() {
            if let Some(anno_key) = self.anno_keys.get_symbol(anno_key) {
                // sample a maximal number of annotation values
                let mut rng = rand::thread_rng();
                if let Some(values_for_key) = self.by_anno.get(&anno_key) {
                    let sampled_anno_values: Vec<usize> = values_for_key
                        .iter()
                        .flat_map(|(val, items)| {
                            // repeat value corresponding to the number of nodes with this annotation
                            let v = vec![*val; items.len()];
                            v.into_iter()
                        })
                        .collect();
                    let sampled_anno_indexes: FxHashSet<usize> = rand::seq::index::sample(
                        &mut rng,
                        sampled_anno_values.len(),
                        std::cmp::min(sampled_anno_values.len(), max_sampled_annotations),
                    )
                    .into_iter()
                    .collect();

                    let mut sampled_anno_values: Vec<String> = sampled_anno_values
                        .into_iter()
                        .enumerate()
                        .filter(|x| sampled_anno_indexes.contains(&x.0))
                        .filter_map(|x| self.anno_values.get_value_ref(x.1).cloned())
                        .collect();
                    // create uniformly distributed histogram bounds
                    sampled_anno_values.sort();

                    let num_hist_bounds = if sampled_anno_values.len() < (max_histogram_buckets + 1)
                    {
                        sampled_anno_values.len()
                    } else {
                        max_histogram_buckets + 1
                    };

                    let hist = self
                        .histogram_bounds
                        .entry(anno_key)
                        .or_insert_with(std::vec::Vec::new);

                    if num_hist_bounds >= 2 {
                        hist.resize(num_hist_bounds, String::from(""));

                        let delta: usize = (sampled_anno_values.len() - 1) / (num_hist_bounds - 1);
                        let delta_fraction: usize =
                            (sampled_anno_values.len() - 1) % (num_hist_bounds - 1);

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
        }
        Ok(())
    }

    fn load_annotations_from(&mut self, location: &Path) -> Result<()> {
        // always remove all entries first, so even if there is an error the anno storage is empty
        self.clear_internal();

        let path = location.join("nodes_v1.bin");
        let f = std::fs::File::open(path.clone()).map_err(|e| {
            GraphAnnisCoreError::LoadingAnnotationStorage {
                path: path.to_string_lossy().to_string(),
                source: e,
            }
        })?;
        let mut reader = std::io::BufReader::new(f);
        *self = bincode::deserialize_from(&mut reader)?;

        self.anno_keys.after_deserialization();
        self.anno_values.after_deserialization();

        Ok(())
    }

    fn save_annotations_to(&self, location: &Path) -> Result<()> {
        let f = std::fs::File::create(location.join("nodes_v1.bin"))?;
        let mut writer = std::io::BufWriter::new(f);
        bincode::serialize_into(&mut writer, self)?;

        Ok(())
    }
}

impl AnnoStorageImpl<Edge> {
    pub fn after_deserialization(&mut self) {
        self.anno_keys.after_deserialization();
        self.anno_values.after_deserialization();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::NodeID;

    #[test]
    fn insert_same_anno() {
        let test_anno = Annotation {
            key: AnnoKey {
                name: "anno1".into(),
                ns: "annis".into(),
            },
            val: "test".into(),
        };
        let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
        a.insert(1, test_anno.clone()).unwrap();
        a.insert(1, test_anno.clone()).unwrap();
        a.insert(2, test_anno.clone()).unwrap();
        a.insert(3, test_anno).unwrap();

        assert_eq!(3, a.number_of_annotations().unwrap());
        assert_eq!(3, a.by_container.len());
        assert_eq!(1, a.by_anno.len());
        assert_eq!(1, a.anno_keys.len());

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

        let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
        a.insert(1, test_anno1.clone()).unwrap();
        a.insert(1, test_anno2.clone()).unwrap();
        a.insert(1, test_anno3.clone()).unwrap();

        assert_eq!(3, a.number_of_annotations().unwrap());

        let all = a.get_annotations_for_item(&1).unwrap();
        assert_eq!(3, all.len());

        assert_eq!(test_anno1, all[0]);
        assert_eq!(test_anno2, all[1]);
        assert_eq!(test_anno3, all[2]);
    }

    #[test]
    fn remove() {
        let test_anno = Annotation {
            key: AnnoKey {
                name: "anno1".into(),
                ns: "annis1".into(),
            },
            val: "test".into(),
        };
        let mut a: AnnoStorageImpl<NodeID> = AnnoStorageImpl::new();
        a.insert(1, test_anno.clone()).unwrap();

        assert_eq!(1, a.number_of_annotations().unwrap());
        assert_eq!(1, a.by_container.len());
        assert_eq!(1, a.by_anno.len());
        assert_eq!(1, a.anno_key_sizes.len());
        assert_eq!(&1, a.anno_key_sizes.get(&test_anno.key).unwrap());

        a.remove_annotation_for_item(&1, &test_anno.key).unwrap();

        assert_eq!(0, a.number_of_annotations().unwrap());
        assert_eq!(0, a.by_container.len());
        assert_eq!(0, a.by_anno.len());
        assert_eq!(&0, a.anno_key_sizes.get(&test_anno.key).unwrap_or(&0));
    }
}
