use self::symboltable::SymbolTable;
use super::*;
use bincode;
use errors::*;
use itertools::Itertools;
use malloc_size_of::MallocSizeOf;
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
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default, MallocSizeOf)]
pub struct AnnoStorage<T: Ord + Hash + MallocSizeOf + Default> {
    by_container: FxHashMap<T, Vec<AnnotationRef>>,
    /// A map from an annotation key symbol to a map of all its values to the items having this value for the annotation key
    by_anno: FxHashMap<usize, FxHashMap<usize, Vec<T>>>,
    /// Maps a distinct annotation key to the number of elements having this annotation key.
    anno_key_sizes: BTreeMap<AnnoKey, usize>,
    anno_keys: SymbolTable<AnnoKey>,
    anno_values: SymbolTable<String>,

    /// additional statistical information
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

    fn create_anno_ref(&mut self, orig: Annotation) -> AnnotationRef {
        AnnotationRef {
            key: self.anno_keys.insert(orig.key),
            val: self.anno_values.insert(orig.val),
        }
    }

    pub fn annotation_from_ref(&self, orig: &AnnotationRef) -> Option<Annotation> {
        let key = self.anno_keys.get_value(orig.key)?;
        let val = self.anno_values.get_value(orig.val)?;

        Some(Annotation { key, val })
    }

    fn remove_element_from_by_anno(&mut self, anno: &AnnotationRef, item: &T) {
        let remove_anno_key = if let Some(mut annos_for_key) = self.by_anno.get_mut(&anno.key) {
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
        let anno = self.create_anno_ref(anno);

        let existing_anno = {
            let existing_item_entry = self.by_container.entry(item.clone()).or_insert(Vec::new());

            // check if there is already an item with the same annotation key
            let existing_entry_idx =
                existing_item_entry.binary_search_by_key(&anno.key, |a| a.key.clone());

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
            .entry(anno.key.clone())
            .or_insert(FxHashMap::default())
            .entry(anno.val.clone())
            .or_insert(Vec::default())
            .push(item.clone());

        if existing_anno.is_none() {
            // a new annotation entry was inserted and did not replace an existing one
            self.total_number_of_annos += 1;

            if let Some(largest_item) = self.largest_item.clone() {
                if largest_item < item {
                    self.largest_item = Some(item.clone());
                }
            } else {
                self.largest_item = Some(item.clone());
            }

            let anno_key_entry = self
                .anno_key_sizes
                .entry(orig_anno_key.as_ref().clone())
                .or_insert(0);
            *anno_key_entry = *anno_key_entry + 1;
        }
    }

    fn check_and_remove_value_symbol(&mut self, value_id: usize) {
        let mut still_used = false;
        for (_, values) in self.by_anno.iter() {
            if values.contains_key(&value_id) {
                still_used = true;
                break;
            }
        }
        if !still_used {
            self.anno_values.remove(value_id);
        }
    }

    pub fn remove(&mut self, item: &T, key: &AnnoKey) -> Option<Arc<String>> {
        let mut result = None;

        let orig_key = key;
        let key = self.anno_keys.get_symbol(key)?;

        if let Some(mut all_annos) = self.by_container.remove(item) {
            // find the specific annotation key from the sorted vector of all annotations of this item
            let anno_idx = all_annos.binary_search_by_key(&key, |a| a.key);

            if let Ok(anno_idx) = anno_idx {
                // since value was found, also remove the item from the other containers
                self.remove_element_from_by_anno(&all_annos[anno_idx], item);

                let old_value = all_annos[anno_idx].val.clone();

                // remove the specific annotation key from the entry
                all_annos.remove(anno_idx);

                // decrease the annotation count for this key
                let new_key_count: usize =
                    if let Some(num_of_keys) = self.anno_key_sizes.get_mut(orig_key) {
                        *num_of_keys -= 1;
                        num_of_keys.clone()
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
            return self.anno_values.get_value(result);
        }
        return None;
    }

    pub fn len(&self) -> usize {
        self.total_number_of_annos
    }

    pub fn get_by_key(&self, item: &T, key: &AnnoKey) -> Option<Arc<String>> {
        let key = self.anno_keys.get_symbol(key)?;

        if let Some(all_annos) = self.by_container.get(item) {
            let idx = all_annos.binary_search_by_key(&key, |a| a.key);
            if let Ok(idx) = idx {
                return self.anno_values.get_value(all_annos[idx].val);
            }
        }
        return None;
    }

    pub fn get_by_id(&self, item: &T, key_id: usize) -> Option<Arc<String>> {
        
        if let Some(all_annos) = self.by_container.get(item) {
            let idx = all_annos.binary_search_by_key(&key_id, |a| a.key);
            if let Ok(idx) = idx {
                return self.anno_values.get_value(all_annos[idx].val);
            }
        }
        return None;
    }

    pub fn find_by_name(
        &self,
        item: &T,
        ns: Option<String>,
        name: Option<String>,
    ) -> Vec<Annotation> {
        if let Some(name) = name {
            if let Some(ns) = ns {
                // fully qualified search
                let key = AnnoKey { ns, name };
                let res = self.get_by_key(item, &key);
                if let Some(val) = res {
                    return vec![Annotation {
                        key: Arc::from(key),
                        val: val.clone(),
                    }];
                } else {
                    return vec![];
                }
            } else {
                // get all qualified names for the given annotation name
                let res: Vec<Annotation> = self
                    .get_qnames(&name)
                    .into_iter()
                    .filter_map(|key| {
                        self.get_by_key(item, &key).map(|val| Annotation {
                            key: Arc::from(key),
                            val: val.clone(),
                        })
                    }).collect();
                return res;
            }
        } else {
            // no annotation name given, return all
            return self.get_all(item);
        }
    }

    pub fn get_all(&self, item: &T) -> Vec<Annotation> {
        if let Some(all_annos) = self.by_container.get(item) {
            let mut result: Vec<Annotation> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                if let Some(a) = self.annotation_from_ref(a) {
                    result.push(a);
                }
            }
            return result;
        }
        // return empty result if not found
        return Vec::new();
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
        return result;
    }

    /// Get all the annotation keys which are part of this annotation storage
    pub fn get_all_keys(&self) -> Vec<AnnoKey> {
        return self.anno_key_sizes.keys().cloned().collect();
    }

    /// Returns an internal identifier for the annotation key that can be used for faster lookup of values.
    pub fn get_key_id(&self, key: &AnnoKey) -> Option<usize> {
        self.anno_keys.get_symbol(key)
    }

    pub fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<Arc<String>> {
        if let Some(key) = self.anno_keys.get_symbol(key) {
            if let Some(values_for_key) = self.by_anno.get(&key) {
                if most_frequent_first {
                    let result = values_for_key
                        .iter()
                        .filter_map(|(val, items)| {
                            let val = self.anno_values.get_value(*val)?;
                            Some((items.len(), val))
                        }).sorted();
                    return result.into_iter().rev().map(|(_, val)| val).collect();
                } else {
                    return values_for_key
                        .iter()
                        .filter_map(|(val, _items)| self.anno_values.get_value(*val))
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
    ) -> Box<Iterator<Item = (&T, Annotation)> + 'a> {
        let key_ranges: Vec<AnnoKey> = if let Some(ns) = namespace {
            vec![AnnoKey { ns, name }]
        } else {
            self.get_qnames(&name)
        };

        let value = value.and_then(|v| self.anno_values.get_symbol(&v));

        let values: Vec<(Arc<AnnoKey>, &FxHashMap<usize, Vec<T>>)> = key_ranges
            .into_iter()
            .filter_map(|key| {
                let key_id = self.anno_keys.get_symbol(&key)?;
                if let Some(values_for_key) = self.by_anno.get(&key_id) {
                    Some((Arc::from(key), values_for_key))
                } else {
                    None
                }
            }).collect();

        if let Some(value) = value {
            let it = values
            .into_iter()
            // find the items with the correct value
            .filter_map(move |(key, values)| if let Some(items) = values.get(&value) {
                let anno = Annotation {
                    key: key,
                    val: self.anno_values.get_value(value)?,
                };
                Some((items, anno))
            } else {
                None
            })
            // flatten the hash set of all items, returns all items for the condition
            .flat_map(|(items, anno)| items.iter().zip(std::iter::repeat(anno.clone())));
            return Box::new(it);
        } else {
            let it = values
            .into_iter()
            // flatten the hash set of all items, returns all items for the condition
            .flat_map(|(key, values)| values.iter().zip(std::iter::repeat(key.clone())))
            // create annotations from all flattened values
            .flat_map(move | ((val, items), key) | {
                let val = if let Some(val) = self.anno_values.get_value(*val) {
                    val
                } else {
                    panic!("Could not get value for internal symbold with ID {}", val);
                };

                let anno = Annotation {
                    key,
                    val,
                };
                items.iter().zip(std::iter::repeat(anno))
            });
            return Box::new(it);
        }
    }

    pub fn num_of_annotations(&self, ns: Option<String>, name: String) -> usize {
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
        return result;
    }

    pub fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns) => vec![AnnoKey {name, ns}],
            None => self.get_qnames(&name),
        };
        
        let mut universe_size: usize = 0;
        let mut sum_histogram_buckets: usize = 0;
        let mut count_matches: usize = 0;

        // guess for each fully qualified annotation key and return the sum of all guesses
        for anno_key in qualified_keys.into_iter() {
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
                                if bucket_begin <= &String::from(upper_val)
                                    && &String::from(lower_val) <= bucket_end
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
            return (selectivity * (universe_size as f64)).round() as usize;
        } else {
            return 0;
        }
    }

    pub fn guess_max_count_regex(&self, ns: Option<String>, name: String, pattern: &str) -> usize {
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
                return self.guess_max_count(ns, name, &lower_val, &upper_val);
            }
        }

        return 0;
    }

    pub fn get_largest_item(&self) -> Option<T> {
        self.largest_item.clone()
    }

    pub fn calculate_statistics(&mut self) {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separatly
        for (anno_key, _num_of_annos) in &self.anno_key_sizes {
            if let Some(anno_key) = self.anno_keys.get_symbol(anno_key) {
                // sample a maximal number of annotation values
                let mut rng = rand::thread_rng();
                if let Some(values_for_key) = self.by_anno.get(&anno_key) {
                    let sampled_anno_values: Vec<usize> = values_for_key
                        .iter()
                        .flat_map(|(val, items)| {
                            // repeat value corresponding to the number of nodes with this annotation
                            let v = vec![val.clone(); items.len()];
                            v.into_iter()
                        }).collect();
                    let sampled_anno_indexes: FxHashSet<usize> = rand::seq::sample_indices(
                        &mut rng,
                        sampled_anno_values.len(),
                        std::cmp::min(sampled_anno_values.len(), max_sampled_annotations),
                    ).into_iter()
                    .collect();

                    let mut sampled_anno_values: Vec<Arc<String>> = sampled_anno_values
                        .into_iter()
                        .enumerate()
                        .filter(|x| sampled_anno_indexes.contains(&x.0))
                        .filter_map(|x| self.anno_values.get_value(x.1))
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
                        .entry(anno_key.clone())
                        .or_insert(std::vec::Vec::new());

                    if num_hist_bounds >= 2 {
                        hist.resize(num_hist_bounds, String::from(""));

                        let delta: usize = (sampled_anno_values.len() - 1) / (num_hist_bounds - 1);
                        let delta_fraction: usize =
                            (sampled_anno_values.len() - 1) % (num_hist_bounds - 1);

                        let mut pos = 0;
                        let mut pos_fraction = 0;
                        for i in 0..num_hist_bounds {
                            hist[i] = sampled_anno_values[pos].as_ref().clone();
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

    pub fn save_to_file(&self, path: &str) -> bool {
        let f = std::fs::File::create(path).unwrap();

        let mut buf_writer = std::io::BufWriter::new(f);

        bincode::serialize_into(&mut buf_writer, self).is_ok()
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

impl AnnoStorage<NodeID> {
    pub fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let it = self
            .matching_items(namespace, name, value)
            .map(|(node, anno)| Match { node: *node, anno });
        return Box::new(it);
    }

    pub fn regex_anno_search<'a>(
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
                .filter(move |(_node, anno)| re.is_match(&anno.val))
                .map(|(node, anno)| Match {
                    node: *node,
                    anno: anno,
                });
            return Box::new(it);
        } else {
            // if regular expression pattern is invalid return empty iterator
            return Box::new(std::iter::empty());
        }
    }
}

impl AnnoStorage<Edge> {
    pub fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: Option<String>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let it = self
            .matching_items(namespace, name, value)
            .map(|(edge, anno)| Match {
                node: edge.source.clone(),
                anno,
            });
        return Box::new(it);
    }
}

mod symboltable;
#[cfg(test)]
mod tests;
