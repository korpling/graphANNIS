use self::symboltable::SymbolTable;
use crate::annis::db::AnnotationStorage;
use crate::annis::db::Match;
use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, AnnoKeyID, Annotation};
use crate::annis::types::{Edge, NodeID};
use crate::annis::util;
use crate::annis::util::memory_estimation;
use bincode;
use itertools::Itertools;
use crate::malloc_size_of::MallocSizeOf;
use rand;
use regex;
use regex_syntax;
use rustc_hash::{FxHashMap, FxHashSet};
use serde;
use serde::de::DeserializeOwned;
use std;
use std::collections::BTreeMap;
use std::collections::Bound::*;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, Default, MallocSizeOf)]
struct SparseAnnotation {
    key: usize,
    val: usize,
}

#[derive(Serialize, Deserialize, Clone, Default, MallocSizeOf)]
pub struct AnnoStorage<T: Ord + Hash + MallocSizeOf + Default> {
    by_container: FxHashMap<T, Vec<SparseAnnotation>>,
    /// A map from an annotation key symbol to a map of all its values to the items having this value for the annotation key
    by_anno: FxHashMap<usize, FxHashMap<usize, Vec<T>>>,
    /// Maps a distinct annotation key to the number of elements having this annotation key.
    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    anno_key_sizes: BTreeMap<AnnoKey, usize>,
    anno_keys: SymbolTable<AnnoKey>,
    anno_values: SymbolTable<String>,

    /// additional statistical information
    #[with_malloc_size_of_func = "memory_estimation::size_of_btreemap"]
    histogram_bounds: BTreeMap<usize, Vec<String>>,
    largest_item: Option<T>,
    total_number_of_annos: usize,
}

impl<T: Ord + Hash + Clone + serde::Serialize + DeserializeOwned + MallocSizeOf + Default>
    AnnoStorage<T>
{
    pub fn new() -> AnnoStorage<T> {
        AnnoStorage {
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

    fn create_sparse_anno(&mut self, orig: Annotation) -> SparseAnnotation {
        SparseAnnotation {
            key: self.anno_keys.insert(orig.key),
            val: self.anno_values.insert(orig.val),
        }
    }

    fn create_annotation_from_sparse(&self, orig: &SparseAnnotation) -> Option<Annotation> {
        let key = self.anno_keys.get_value(orig.key)?;
        let val = self.anno_values.get_value(orig.val)?;

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

    pub fn insert(&mut self, item: T, anno: Annotation) {
        let orig_anno_key = anno.key.clone();
        let anno = self.create_sparse_anno(anno);

        let existing_anno = {
            let existing_item_entry = self
                .by_container
                .entry(item.clone())
                .or_insert_with(Vec::new);

            // check if there is already an item with the same annotation key
            let existing_entry_idx = existing_item_entry.binary_search_by_key(&anno.key, |a| a.key);

            if let Ok(existing_entry_idx) = existing_entry_idx {
                let orig_anno = existing_item_entry[existing_entry_idx].clone();
                // abort if the same annotation key with the same value already exist
                if orig_anno.val == anno.val {
                    return;
                }
                // insert annotation for item at existing position
                existing_item_entry[existing_entry_idx] = anno.clone();
                Some(orig_anno)
            } else if let Err(insertion_idx) = existing_entry_idx {
                // insert at sorted position -> the result will still be a sorted vector
                existing_item_entry.insert(insertion_idx, anno.clone());
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

            let anno_key_entry = self
                .anno_key_sizes
                .entry(orig_anno_key.clone())
                .or_insert(0);
            *anno_key_entry += 1;
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

    pub fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Option<String> {
        let mut result = None;

        let orig_key = key;
        let key = self.anno_keys.get_symbol(key)?;

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
                    self.anno_key_sizes.remove(&orig_key);
                    self.anno_keys.remove(key);
                }

                self.check_and_remove_value_symbol(old_value);
                self.total_number_of_annos -= 1;

                result = Some(old_value);
            }
            // if there are more annotations for this item, re-insert them
            if !all_annos.is_empty() {
                self.by_container.insert(item.clone(), all_annos);
            }
        }

        if let Some(result) = result {
            return self.anno_values.get_value(result).cloned();
        }
        None
    }

    pub fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Option<&str> {
        let key = self.anno_keys.get_symbol(key)?;

        if let Some(all_annos) = self.by_container.get(item) {
            let idx = all_annos.binary_search_by_key(&key, |a| a.key);
            if let Ok(idx) = idx {
                if let Some(val) = self.anno_values.get_value(all_annos[idx].val) {
                    return Some(&val[..]);
                }
            }
        }
        None
    }

    pub fn get_value_for_item_by_id(&self, item: &T, key_id: AnnoKeyID) -> Option<&str> {
        if let Some(all_annos) = self.by_container.get(item) {
            let idx = all_annos.binary_search_by_key(&key_id, |a| a.key);
            if let Ok(idx) = idx {
                if let Some(val) = self.anno_values.get_value(all_annos[idx].val) {
                    return Some(&val[..]);
                }
            }
        }
        None
    }

    pub fn find_annotations_for_item(
        &self,
        item: &T,
        ns: Option<String>,
        name: Option<String>,
    ) -> Vec<AnnoKeyID> {
        if let Some(name) = name {
            if let Some(ns) = ns {
                // fully qualified search
                let key = AnnoKey { ns, name };
                if let Some(key_id) = self.get_key_id(&key) {
                    if self.get_value_for_item_by_id(item, key_id).is_some() {
                        return vec![key_id];
                    }
                }
                return vec![];
            } else {
                // get all qualified names for the given annotation name
                let res: Vec<AnnoKeyID> = self
                    .get_qnames(&name)
                    .into_iter()
                    .filter_map(|key| self.get_key_id(&key))
                    .filter(|key_id| self.get_value_for_item_by_id(item, *key_id).is_some())
                    .collect();
                return res;
            }
        } else if let Some(annos) = self.by_container.get(item) {
            // no annotation name given, return all
            return annos.iter().map(|sparse_anno| sparse_anno.key).collect();
        } else {
            return vec![];
        }
    }

    /// Get all the annotation keys of a node
    pub fn get_all_keys_for_item(&self, item: &T) -> Vec<AnnoKey> {
        if let Some(all_annos) = self.by_container.get(item) {
            let mut result: Vec<AnnoKey> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                if let Some(key) = self.anno_keys.get_value(a.key) {
                    result.push(key.clone());
                }
            }
            return result;
        }
        // return empty result if not found
        Vec::new()
    }

    fn get_annotations_for_item_impl(&self, item: &T) -> Vec<Annotation> {
        if let Some(all_annos) = self.by_container.get(item) {
            let mut result: Vec<Annotation> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                if let Some(a) = self.create_annotation_from_sparse(a) {
                    result.push(a);
                }
            }
            return result;
        }
        // return empty result if not found
        Vec::new()
    }

    pub fn clear(&mut self) {
        self.by_container.clear();
        self.by_anno.clear();
        self.anno_keys.clear();
        self.histogram_bounds.clear();
        self.largest_item = None;
        self.anno_values.clear();
    }

    /// Get all qualified annotation names (including namespace) for a given annotation name
    pub fn get_qnames(&self, name: &str) -> Vec<AnnoKey> {
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

    /// Returns an internal identifier for the annotation key that can be used for faster lookup of values.
    pub fn get_key_id(&self, key: &AnnoKey) -> Option<AnnoKeyID> {
        self.anno_keys.get_symbol(key)
    }

    /// Returns the annotation key from the internal identifier.
    pub fn get_key_value(&self, key_id: AnnoKeyID) -> Option<AnnoKey> {
        self.anno_keys.get_value(key_id).cloned()
    }

    fn get_all_values_impl(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str> {
        if let Some(key) = self.anno_keys.get_symbol(key) {
            if let Some(values_for_key) = self.by_anno.get(&key) {
                if most_frequent_first {
                    let result = values_for_key
                        .iter()
                        .filter_map(|(val, items)| {
                            let val = self.anno_values.get_value(*val)?;
                            Some((items.len(), val))
                        }).sorted();
                    return result.into_iter().rev().map(|(_, val)| &val[..]).collect();
                } else {
                    return values_for_key
                        .iter()
                        .filter_map(|(val, _items)| self.anno_values.get_value(*val))
                        .map(|val| &val[..])
                        .collect();
                }
            }
        }
        return vec![];
    }

    fn matching_items<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = (T, AnnoKeyID)> + 'a> {
        let key_ranges: Vec<AnnoKey> = if let Some(ns) = namespace {
            vec![AnnoKey { ns, name }]
        } else {
            self.get_qnames(&name)
        };
        let values: Vec<(AnnoKeyID, &FxHashMap<usize, Vec<T>>)> = key_ranges
            .into_iter()
            .filter_map(|key| {
                let key_id = self.anno_keys.get_symbol(&key)?;
                if let Some(values_for_key) = self.by_anno.get(&key_id) {
                    Some((key_id, values_for_key))
                } else {
                    None
                }
            }).collect();

        if let Some(value) = value {
            let target_value_symbol = self.anno_values.get_symbol(&value);

            if let Some(target_value_symbol) = target_value_symbol {
                let it = values
                    .into_iter()
                    // find the items with the correct value
                    .filter_map(move |(key_id, values)| {
                        if let Some(items) = values.get(&target_value_symbol) {
                            Some((items, key_id))
                        } else {
                            None
                        }
                    })
                    // flatten the hash set of all items, returns all items for the condition
                    .flat_map(|(items, key_id)| items.iter().cloned().zip(std::iter::repeat(key_id)));
                return Box::new(it);
            } else {
                // value is not known, return empty result
                return Box::new(std::iter::empty());
            }
        } else {
            let it = values
                .into_iter()
                // flatten the hash set of all items, returns all items for the condition
                .flat_map(|(key_id, values)| values.iter().zip(std::iter::repeat(key_id)))
                // create annotations from all flattened values
                .flat_map(move |((_, items), key_id)| items.iter().cloned().zip(std::iter::repeat(key_id)));
            return Box::new(it);
        }
    }

    fn number_of_annotations_by_name_impl(&self, ns: Option<String>, name: String) -> usize {
        let qualified_keys = match ns {
            Some(ns) => self.anno_key_sizes.range((
                Included(AnnoKey {
                    name: name.clone(),
                    ns: ns.clone(),
                }),
                Included(AnnoKey { name, ns }),
            )),
            None => self.anno_key_sizes.range(
                AnnoKey {
                    name: name.clone(),
                    ns: String::default(),
                }..AnnoKey {
                    name,
                    ns: std::char::MAX.to_string(),
                },
            ),
        };
        let mut result = 0;
        for (_anno_key, anno_size) in qualified_keys {
            result += anno_size;
        }
        result
    }

    fn guess_max_count_impl(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey { name, ns }],
            None => self.get_qnames(&name),
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
            (selectivity * (universe_size as f64)).round() as usize
        } else {
            0
        }
    }

    fn guess_max_count_regex_impl(&self, ns: Option<String>, name: String, pattern: &str) -> usize {
        let full_match_pattern = util::regex_full_match(pattern);

        let parsed = regex_syntax::Parser::new().parse(&full_match_pattern);
        if let Ok(parsed) = parsed {
            let expr: regex_syntax::hir::Hir = parsed;

            let prefix_set = regex_syntax::hir::literal::Literals::prefixes(&expr);
            let val_prefix = std::str::from_utf8(prefix_set.longest_common_prefix());

            if val_prefix.is_ok() {
                let lower_val = val_prefix.unwrap();
                let mut upper_val = String::from(lower_val);
                upper_val.push(std::char::MAX);
                return self.guess_max_count_impl(ns, name, &lower_val, &upper_val);
            }
        }

        0
    }

    pub fn get_largest_item(&self) -> Option<T> {
        self.largest_item.clone()
    }

    pub fn calculate_statistics(&mut self) {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separatly
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
                        }).collect();
                    let sampled_anno_indexes: FxHashSet<usize> = rand::seq::index::sample(
                        &mut rng,
                        sampled_anno_values.len(),
                        std::cmp::min(sampled_anno_values.len(), max_sampled_annotations),
                    ).into_iter()
                    .collect();

                    let mut sampled_anno_values: Vec<String> = sampled_anno_values
                        .into_iter()
                        .enumerate()
                        .filter(|x| sampled_anno_indexes.contains(&x.0))
                        .filter_map(|x| self.anno_values.get_value(x.1).cloned())
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
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<()> {
        // always remove all entries first, so even if there is an error the anno storage is empty
        self.clear();

        let path = PathBuf::from(path);
        let f = std::fs::File::open(path.clone()).chain_err(|| {
            format!(
                "Could not load string storage from file {}",
                path.to_string_lossy()
            )
        })?;
        let mut reader = std::io::BufReader::new(f);
        *self = bincode::deserialize_from(&mut reader)?;

        self.anno_keys.after_deserialization();
        self.anno_values.after_deserialization();

        Ok(())
    }
}

impl AnnotationStorage<NodeID> for AnnoStorage<NodeID> {
    fn get_annotations_for_item(&self, item: &NodeID) -> Vec<Annotation> {
        self.get_annotations_for_item_impl(item)
    }

    fn number_of_annotations(&self) -> usize {
        self.total_number_of_annos
    }

    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize {
        self.number_of_annotations_by_name_impl(ns, name)
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let it =
            self.matching_items(namespace, name, value)
                .filter_map(move |item| {
                    Some(item.into())
                });
        Box::new(it)
    }

    fn regex_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        pattern: &str,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if let Ok(re) = compiled_result {
            let it = self
                .matching_items(namespace, name, None)
                .filter(move |(node, anno_key_id)| {
                    if let Some(val) = self.get_value_for_item_by_id(node, *anno_key_id) {
                        re.is_match(val)
                    } else {
                        false
                    }
                }).filter_map(move |item| {
                    Some(item.into())
                });
            return Box::new(it);
        } else {
            // if regular expression pattern is invalid return empty iterator
            return Box::new(std::iter::empty());
        }
    }

    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        self.guess_max_count_impl(ns, name, lower_val, upper_val)
    }

    fn guess_max_count_regex(&self, ns: Option<String>, name: String, pattern: &str) -> usize {
        self.guess_max_count_regex_impl(ns, name, pattern)
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str> {
        self.get_all_values_impl(key, most_frequent_first)
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        self.anno_key_sizes.keys().cloned().collect()
    }
}

impl AnnoStorage<Edge> {
    pub fn after_deserialization(&mut self) {
        self.anno_keys.after_deserialization();
        self.anno_values.after_deserialization();
    }
}

impl AnnotationStorage<Edge> for AnnoStorage<Edge> {
    fn get_annotations_for_item(&self, item: &Edge) -> Vec<Annotation> {
        self.get_annotations_for_item_impl(item)
    }

    fn number_of_annotations(&self) -> usize {
        self.total_number_of_annos
    }

    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize {
        self.number_of_annotations_by_name_impl(ns, name)
    }

    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let it =
            self.matching_items(namespace, name, value)
                .filter_map(move |item| {
                    Some(item.into())
                });
        Box::new(it)
    }

    fn regex_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        pattern: &str,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if let Ok(re) = compiled_result {
            let it = self
                .matching_items(namespace, name, None)
                .filter(move |(node, anno_key_id)| {
                    if let Some(val) = self.get_value_for_item_by_id(node, *anno_key_id) {
                        re.is_match(val)
                    } else {
                        false
                    }
                }).filter_map(move |item| {
                    Some(item.into())
                });
            return Box::new(it);
        } else {
            // if regular expression pattern is invalid return empty iterator
            return Box::new(std::iter::empty());
        }
    }

    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        self.guess_max_count_impl(ns, name, lower_val, upper_val)
    }

    fn guess_max_count_regex(&self, ns: Option<String>, name: String, pattern: &str) -> usize {
        self.guess_max_count_regex_impl(ns, name, pattern)
    }

    fn annotation_keys(&self) -> Vec<AnnoKey> {
        self.anno_key_sizes.keys().cloned().collect()
    }

    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<&str> {
        self.get_all_values_impl(key, most_frequent_first)
    }
}

mod symboltable;
#[cfg(test)]
mod tests;
