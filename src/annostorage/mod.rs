use super::*;
use std::collections::{BTreeMap};
use rustc_hash::{FxHashMap,FxHashSet};
use std::collections::Bound::*;
use std::hash::Hash;
use std;
use std::path::PathBuf;
use rand;
use regex_syntax;
use regex;
use stringstorage::StringStorage;
use bincode;
use serde;
use serde::de::DeserializeOwned;
use itertools::Itertools;
use malloc_size_of::{MallocSizeOf};
use errors::*;

#[derive(Serialize, Deserialize, Clone, Default, MallocSizeOf)]
pub struct AnnoStorage<T: Ord + Hash + MallocSizeOf + Default> {
    by_container: FxHashMap<T, Vec<Annotation>>,
    #[serde(skip)]
    by_anno: FxHashMap<AnnoKey, FxHashMap<StringID, FxHashSet<T>>>,
    /// Maps a distinct annotation key to the number of elements having this annotation key.
    anno_keys: BTreeMap<AnnoKey, usize>,
    /// additional statistical information
    histogram_bounds: BTreeMap<AnnoKey, Vec<String>>,
    largest_item: Option<T>,
    total_number_of_annos: usize,
}


impl<T: Ord + Hash + Clone + serde::Serialize + DeserializeOwned + MallocSizeOf + Default> AnnoStorage<T> {
    pub fn new() -> AnnoStorage<T> {
        AnnoStorage {
            by_container: FxHashMap::default(),
            by_anno: FxHashMap::default(),
            anno_keys: BTreeMap::new(),
            histogram_bounds: BTreeMap::new(),
            largest_item: None,
            total_number_of_annos: 0,
        }
    }

    fn remove_element_from_by_anno(&mut self, anno: &Annotation, item: &T) {
        let remove_anno_key = if let Some(mut annos_for_key) = self.by_anno.get_mut(&anno.key) {
            
            let remove_anno_val = if let Some(items_for_anno) = annos_for_key.get_mut(&anno.val) {
                items_for_anno.remove(&item);
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
        }
    }

    pub fn insert(&mut self, item: T, anno: Annotation) {
        
        let existing_anno = {
            let existing_item_entry = self.by_container.entry(item.clone()).or_insert(Vec::new());

            // check if there is already an item with the same annotation key
            let existing_entry_idx =
                existing_item_entry.binary_search_by_key(&anno.key, |a| a.key.clone());

            
            if let Ok(existing_entry_idx) = existing_entry_idx {
                let orig_anno = existing_item_entry[existing_entry_idx].clone();
                // insert annotation for item at existing position
                existing_item_entry[existing_entry_idx] = anno.clone();
                Some(orig_anno)
            } else if let Err(insertion_idx) = existing_entry_idx {
                // insert at sorted position -> the result will still be a sorted vector
                existing_item_entry.insert(insertion_idx, anno.clone());
                None
            } else {None}
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
            .or_insert(FxHashSet::default())
            .insert(item.clone());

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

            let anno_key_entry = self.anno_keys.entry(anno.key.clone()).or_insert(0);
            *anno_key_entry = *anno_key_entry + 1;
        }
    }

    pub fn remove(&mut self, item: &T, key: &AnnoKey) -> Option<StringID> {
        let mut result = None;

        if let Some(mut all_annos) = self.by_container.remove(item) {
            // find the specific annotation key from the sorted vector of all annotations of this item
            let anno_idx = all_annos.binary_search_by_key(key, |a| a.key.clone());

            if let Ok(anno_idx) = anno_idx {
                // since value was found, also remove the item from the other containers
                self.remove_element_from_by_anno(
                    &all_annos[anno_idx],
                    item,
                );

                let old_value = all_annos[anno_idx].val;

                // remove the specific annotation key from the entry
                all_annos.remove(anno_idx);

                
                // decrease the annotation count for this key
                let num_of_keys = self.anno_keys.get_mut(key);
                if num_of_keys.is_some() {
                    let x = num_of_keys.unwrap();
                    *x = *x - 1;
                }

                self.total_number_of_annos -= 1;

                result = Some(old_value);
            }
            // if there are more annotations for this item, re-insert them
            if !all_annos.is_empty() {
                self.by_container.insert(item.clone(), all_annos);
            }
        }
        return result;
    }

    pub fn len(&self) -> usize {
        self.total_number_of_annos
    }

    pub fn get(&self, item: &T, key: &AnnoKey) -> Option<&StringID> {
        if let Some(all_annos) = self.by_container.get(item) {

            let idx = all_annos.binary_search_by_key(key, |a| a.key.clone());
            if let Ok(idx) = idx {
                return Some(&all_annos[idx].val);
            }
        }
        return None;
    }

    pub fn find_by_name(
        &self,
        item: &T,
        ns: Option<StringID>,
        name: Option<StringID>,
    ) -> Vec<Annotation> {
        if let Some(name) = name {
            if let Some(ns) = ns {
                // fully qualified search
                let key = AnnoKey { ns, name };
                let res = self.get(item, &key);
                if let Some(val) = res {
                    return vec![
                        Annotation {
                            key,
                            val: val.clone(),
                        },
                    ];
                } else {
                    return vec![];
                }
            } else {
                // get all qualified names for the given annotation name
                let res: Vec<Annotation> = self.get_qnames(name)
                    .into_iter()
                    .filter_map(|key| {
                        self.get(item, &key).map(|val| Annotation {
                            key,
                            val: val.clone(),
                        })
                    })
                    .collect();
                return res;
            }
        } else {
            // no annotation name given, return all
            return self.get_all(item);
        }
    }

    pub fn get_all(&self, item: &T) -> Vec<Annotation> {
       
        if let Some(all_annos) = self.by_container.get(item) {
            let mut result : Vec<Annotation> = Vec::with_capacity(all_annos.len());
            for a in all_annos.iter() {
                result.push(a.clone())
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
    }

    /// Get all qualified annotation names (including namespace) for a given annotation name
    pub fn get_qnames(&self, name: StringID) -> Vec<AnnoKey> {
        self.anno_keys
            .range(
                AnnoKey {
                    name,
                    ns: StringID::min_value(),
                }..AnnoKey {
                    name,
                    ns: StringID::max_value(),
                },
            )
            .map(|r| r.0)
            .cloned()
            .collect::<Vec<AnnoKey>>()
    }

    /// Get all the annotation keys which are part of this annotation storage
    pub fn get_all_keys(&self) -> Vec<AnnoKey> {
        return self.anno_keys.keys().cloned().collect();
    }

    pub fn get_all_values<'a>(
        &'a self,
        key: AnnoKey,
        most_frequent_first: bool,
    ) -> Box<Iterator<Item = StringID> + 'a> {
        
        if let Some(values_for_key) = self.by_anno.get(&key) {
            if most_frequent_first {
                let it = values_for_key
                    .iter()
                    .map(|(ref val, ref items)| (items.len(), val.clone()))
                    .sorted()
                    .into_iter()
                    .rev()
                    .map(|(_, val)| val.clone());

                return Box::from(it);
            } else {
                let it = values_for_key
                    .iter()
                    .map(|(val, _items)| val.clone());
                return Box::from(it);
            }
        } else {
            return Box::from(std::iter::empty());
        }
    }

    fn matching_items<'a>(
        &'a self,
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
    ) -> Box<Iterator<Item = (&T, Annotation)> + 'a> {
        let key_ranges: Vec<AnnoKey> = if let Some(ns) = namespace {
            vec![AnnoKey { ns, name }]
        } else {
            self.get_qnames(name)
        };

        let values: Vec<(AnnoKey, &FxHashMap<StringID, FxHashSet<T>>)> = key_ranges
            .into_iter()
            .filter_map(|k| {
                if let Some(values_for_key) = self.by_anno.get(&k) {
                    Some((k.clone(), values_for_key))
                } else {
                    None
                }
            }).collect();

        if let Some(value) = value {
            let it = values
            .into_iter()
            // find the items with the correct value
            .filter_map(move |(key, values)| if let Some(items) = values.get(&value) {
                Some((items, Annotation {
                    key,
                    val: value.clone(),
                }))
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
            .flat_map(| ((val, items), key) | items.iter().zip(std::iter::repeat(Annotation {
                key, val: val.clone()
            })));
            return Box::new(it);
        }
    }

    pub fn num_of_annotations(&self, ns: Option<StringID>, name: StringID) -> usize {
        let qualified_keys = match ns {
            Some(ns_id) => self.anno_keys.range((
                Included(AnnoKey { name, ns: ns_id }),
                Included(AnnoKey { name, ns: ns_id }),
            )),
            None => self.anno_keys.range(
                AnnoKey {
                    name,
                    ns: StringID::min_value(),
                }..AnnoKey {
                    name,
                    ns: StringID::max_value(),
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
        ns: Option<StringID>,
        name: StringID,
        lower_val: &str,
        upper_val: &str,
    ) -> usize {
        // find all complete keys which have the given name (and namespace if given)
        let qualified_keys = match ns {
            Some(ns_id) => self.anno_keys.range((
                Included(AnnoKey { name, ns: ns_id }),
                Included(AnnoKey { name, ns: ns_id }),
            )),
            None => self.anno_keys.range(
                AnnoKey {
                    name,
                    ns: StringID::min_value(),
                }..AnnoKey {
                    name,
                    ns: StringID::max_value(),
                },
            ),
        };

        let mut universe_size: usize = 0;
        let mut sum_histogram_buckets: usize = 0;
        let mut count_matches: usize = 0;

        // guess for each fully qualified annotation key and return the sum of all guesses
        for (anno_key, anno_size) in qualified_keys {
            universe_size += *anno_size;

            let opt_histo = self.histogram_bounds.get(anno_key);
            if opt_histo.is_some() {
                // find the range in which the value is contained
                let histo = opt_histo.unwrap();

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

        if sum_histogram_buckets > 0 {
            let selectivity: f64 = (count_matches as f64) / (sum_histogram_buckets as f64);
            return (selectivity * (universe_size as f64)).round() as usize;
        } else {
            return 0;
        }
    }

    pub fn guess_max_count_regex(
        &self,
        ns: Option<StringID>,
        name: StringID,
        pattern: &str,
    ) -> usize {
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

    pub fn calculate_statistics(&mut self, string_storage: &stringstorage::StringStorage) {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separatly
        for (anno_key, _num_of_annos) in &self.anno_keys {
            let hist = self.histogram_bounds
                .entry(anno_key.clone())
                .or_insert(std::vec::Vec::new());

            // sample a maximal number of annotation values
            let mut rng = rand::thread_rng();
            if let Some(values_for_key) = self.by_anno.get(anno_key) {
                let sampled_anno_values: Vec<&String> = values_for_key
                    .iter()
                    .flat_map(|(val, items)| {
                        let s = string_storage.str(*val);
                        let v = if let Some(s) = s {
                            // repeat value corresponding to the number of nodes with this annotation
                            vec![s; items.len()]
                        } else {
                            vec![]
                        };
                        v.into_iter()
                    })
                    .collect();
                let sampled_anno_indexes: FxHashSet<usize> = rand::seq::sample_indices(
                    &mut rng,
                    sampled_anno_values.len(),
                    std::cmp::min(sampled_anno_values.len(), max_sampled_annotations),
                ).into_iter()
                    .collect();

                let mut sampled_anno_values: Vec<&String> = sampled_anno_values
                    .into_iter()
                    .enumerate()
                    .filter(|x| sampled_anno_indexes.contains(&x.0))
                    .map(|x| x.1)
                    .collect();
                // create uniformly distributed histogram bounds
                sampled_anno_values.sort();

                let num_hist_bounds = if sampled_anno_values.len() < (max_histogram_buckets + 1) {
                    sampled_anno_values.len()
                } else {
                    max_histogram_buckets + 1
                };

                if num_hist_bounds >= 2 {
                    hist.resize(num_hist_bounds, String::from(""));

                    let delta: usize = (sampled_anno_values.len() - 1) / (num_hist_bounds - 1);
                    let delta_fraction: usize = (sampled_anno_values.len() - 1) % (num_hist_bounds - 1);

                    let mut pos = 0;
                    let mut pos_fraction = 0;
                    for i in 0..num_hist_bounds {
                        hist[i] = sampled_anno_values[pos].clone();
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

        // optimize for read-only and shrink all containers to minimum
        self.by_container.shrink_to_fit();
        
        // restore the by_anno map
        for (item, annos) in self.by_container.iter_mut() {
            annos.shrink_to_fit();
            for a in annos.iter() {
                self.by_anno.entry(a.key.clone()).or_insert(FxHashMap::default())
                    .entry(a.val.clone()).or_insert(FxHashSet::default())
                    .insert(item.clone());
            }
        }

        self.by_anno.shrink_to_fit();
        for (_key, values_for_key) in self.by_anno.iter_mut() {
            values_for_key.shrink_to_fit();
            for (_, items) in values_for_key.iter_mut() {
                items.shrink_to_fit();
            }
        }

        Ok(())
    }

}

impl AnnoStorage<NodeID> {
    pub fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let it = self
            .matching_items(namespace, name, value)
            .map(|(node, anno)| Match { node: *node, anno });
        return Box::new(it);
    }

    pub fn regex_anno_search<'a>(
        &'a self,
        strings: &'a StringStorage,
        namespace: Option<StringID>,
        name: StringID,
        pattern: &str,
    ) -> Box<Iterator<Item = Match> + 'a> {
        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if let Ok(re) = compiled_result {
            let it = self
                .matching_items(namespace, name, None)
                .filter(move |(_node, anno)| match strings.str(anno.val) {
                    Some(v) => re.is_match(v),
                    None => false,
                })
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
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
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

#[cfg(test)]
mod tests;
