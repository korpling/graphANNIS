use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std;
use rand;
use regex_syntax;
use regex;
use annis::stringstorage::StringStorage;
use bincode;
use serde;
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
pub struct ContainerAnnoKey<T: Ord> {
    pub item: T,
    pub key: AnnoKey,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnnoStorage<T: Ord> {
    by_container: BTreeMap<ContainerAnnoKey<T>, StringID>,
    by_anno: BTreeMap<Annotation, BTreeSet<T>>,
    /// Maps a distinct annotation key to the number of keys available.
    anno_keys: BTreeMap<AnnoKey, usize>,
    /// additional statistical information
    histogram_bounds: BTreeMap<AnnoKey, Vec<String>>,
}


impl<T: Ord + Clone + serde::Serialize + DeserializeOwned> AnnoStorage<T> {
    pub fn new() -> AnnoStorage<T> {
        AnnoStorage {
            by_container: BTreeMap::new(),
            by_anno: BTreeMap::new(),
            anno_keys: BTreeMap::new(),
            histogram_bounds: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, item: T, anno: Annotation) {
        self.by_container.insert(
            ContainerAnnoKey {
                item: item.clone(),
                key: anno.key.clone(),
            },
            anno.val.clone(),
        );

        let anno_key_entry = self.anno_keys.entry(anno.clone().key).or_insert(0);
        *anno_key_entry = *anno_key_entry + 1;

        // inserts a new element into the set
        // if set is not existing yet it is created
        self.by_anno
            .entry(anno.clone())
            .or_insert(BTreeSet::new())
            .insert(item);
    }

    pub fn remove(&mut self, item: &T, key: &AnnoKey) -> Option<StringID> {
        let old_value = self.by_container.remove(&ContainerAnnoKey::<T> {
            item: item.clone(),
            key: key.clone(),
        });
        if old_value.is_some() {
            // of value was found, also remove the item from the other containers
            self.by_anno.remove(&Annotation {
                key: key.clone(),
                val: old_value.unwrap(),
            });
            // decrease the annotation count for this key
            let num_of_keys = self.anno_keys.get_mut(key);
            if num_of_keys.is_some() {
                let x = num_of_keys.unwrap();
                *x = *x - 1;
            }

            return old_value;
        }
        return None;
    }

    pub fn len(&self) -> usize {
        self.by_container.len()
    }

    pub fn get(&self, item: &T, key: &AnnoKey) -> Option<&StringID> {
        let container_key = ContainerAnnoKey::<T> {
            item: item.clone(),
            key: key.clone(),
        };

        self.by_container.get(&container_key)
    }

    pub fn get_all(&self, item: &T) -> Vec<Annotation> {
        let min_key = AnnoKey { name: 0, ns: 0 };
        let max_key = AnnoKey {
            name: StringID::max_value(),
            ns: StringID::max_value(),
        };

        let found_range = self.by_container.range(
            ContainerAnnoKey {
                item: item.clone(),
                key: min_key,
            }..ContainerAnnoKey {
                item: item.clone(),
                key: max_key,
            },
        );

        let mut result = vec![];
        for (k, &v) in found_range {
            result.push(Annotation {
                key: k.clone().key,
                val: v,
            });
        }

        return result;
    }

    pub fn clear(&mut self) {
        self.by_container.clear();
        self.by_anno.clear();
        self.anno_keys.clear();
        self.histogram_bounds.clear();
    }

     pub fn anno_range_exact(
        & self,
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
    ) -> std::ops::Range<Annotation> {
        let ns_pair = match namespace {
            Some(v) => (v, v),
            None => (StringID::min_value(), StringID::max_value()),
        };

        let val_pair = match value {
            Some(v) => (v, v),
            None => (StringID::min_value(), StringID::max_value()),
        };

        let anno_min = Annotation {
            key: AnnoKey {
                name,
                ns: ns_pair.0,
            },
            val: val_pair.0,
        };
        let anno_max = Annotation {
            key: AnnoKey {
                name,
                ns: ns_pair.1,
            },
            val: val_pair.1,
        };

        anno_min..anno_max
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
            Some(ns_id) => self.anno_keys
                .range(AnnoKey { name, ns: ns_id }..AnnoKey { name, ns: ns_id }),
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

        let mut universe_size : usize = 0;
        let mut sum_histogram_buckest : usize = 0;
        let mut count_matches : usize = 0;

        // guess for each fully qualified annotation key and return the sum of all guesses
        for (anno_key, anno_size ) in qualified_keys {
            universe_size += *anno_size;

            let opt_histo = self.histogram_bounds.get(anno_key);
            if opt_histo.is_some() {
                // find the range in which the value is contained
                let histo = opt_histo.unwrap();

                // we need to make sure the histogram is not empty -> should have at least two bounds
                if histo.len() >= 2 {
                    sum_histogram_buckest += histo.len()-1;

                    for i in 0..histo.len()-1 {
                        let bucket_begin = &histo[i];
                        let bucket_end = &histo[i+1];
                        // check if the range overlaps with the search range
                        if  bucket_begin <= &String::from(upper_val) && &String::from(lower_val) <= bucket_end {
                            count_matches += 1;
                        }
                    }
                }
            }
        }

        if sum_histogram_buckest > 0 {
            let selectivity : f64 = (count_matches as f64) / (sum_histogram_buckest as f64);
            return (selectivity * (universe_size as f64)).round() as usize;
        } else {
            return 0;
        }
    }

    pub fn guess_max_count_regex(
        &self,
        ns: Option<StringID>,
        name: StringID,
        pattern : &str
    ) -> usize {

        let full_match_pattern = util::regex_full_match(pattern);

        let opt_expr = regex_syntax::Expr::parse(&full_match_pattern);
        if opt_expr.is_ok() {
            let expr = opt_expr.unwrap();

            let prefix_set = expr.prefixes();
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

    pub fn calculate_statistics(&mut self, string_storage : &stringstorage::StringStorage) {
        let max_histogram_buckets = 250;
        let max_sampled_annotations = 2500;

        self.histogram_bounds.clear();

        // collect statistics for each annotation key separatly
        for anno_key in &self.anno_keys {
            let hist = self.histogram_bounds
                .entry(anno_key.0.clone())
                .or_insert(std::vec::Vec::new());

            let min_anno = Annotation {
                key: anno_key.0.clone(),
                val: StringID::min_value(),
            };
            let max_anno = Annotation {
                key: anno_key.0.clone(),
                val: StringID::min_value(),
            };

            // sample a maximal number of annotation values
            let mut rng = rand::thread_rng();
            let mut sampled_anno_values = rand::sample(
                &mut rng,
                self.by_anno.range(min_anno..max_anno).map(|a| a.0.val),
                max_sampled_annotations,
            );


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
                for i in 0..sampled_anno_values.len() {
                    let val_raw : StringID = sampled_anno_values[pos];
                    hist[i] = string_storage.str(val_raw).unwrap_or(&String::from("")).clone();
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

    #[allow(unused_must_use)]
    pub fn save_to_file(&self, path: &str) {

        let f = std::fs::File::create(path).unwrap();

        let mut buf_writer = std::io::BufWriter::new(f);

        bincode::serialize_into(&mut buf_writer, self, bincode::Infinite);
    }

    pub fn load_from_file(&mut self, path: &str) {

        // always remove all entries first, so even if there is an error the string storage is empty
        self.clear();

        let f = std::fs::File::open(path);
        if f.is_ok() {
            let mut buf_reader = std::io::BufReader::new(f.unwrap());

            let loaded: Result<AnnoStorage<T>, _> =
                bincode::deserialize_from(&mut buf_reader, bincode::Infinite);
            if loaded.is_ok() {
                *self = loaded.unwrap();
            }
        }
    }
}

impl AnnoStorage<NodeID> {
    pub fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
    ) -> Box<Iterator<Item = Match> + 'a> {
        
        let anno_range = self.anno_range_exact(namespace, name, value);

        Box::new(
            self.by_anno
                .range(anno_range)
                .flat_map(|nodes| nodes.1.iter().zip(std::iter::repeat(nodes.0)))
                .map(|m| {
                    Match {
                        node: m.0.clone(),
                        anno: m.1.clone(),
                    }
                }),
        )
    }

    pub fn regex_anno_search<'a> (
        &'a self,
        strings : &'a StringStorage,
        namespace: Option<StringID>,
        name: StringID,
        pattern: &str,
    ) -> Box<Iterator<Item = Match>+'a> {
        
        let ns_pair = match namespace {
            Some(v) => (v, v),
            None => (StringID::min_value(), StringID::max_value()),
        };
        let val_pair = (StringID::min_value(), StringID::max_value());

        let full_match_pattern = util::regex_full_match(pattern);
        let compiled_result = regex::Regex::new(&full_match_pattern);
        if compiled_result.is_ok() {
            let re = compiled_result.unwrap();

            let anno_min = Annotation {
                key: AnnoKey {
                    name,
                    ns: ns_pair.0,
                },
                val: val_pair.0,
            };
            let anno_max = Annotation {
                key: AnnoKey {
                    name,
                    ns: ns_pair.1,
                },
                val: val_pair.1,
            };

            let it = self.by_anno
                .range(anno_min..anno_max)
                .filter(move |a| {
                    match strings.str(a.0.val) {
                        Some(v) => re.is_match(v),
                        None => false,
                    }
                })
                .flat_map(|nodes| nodes.1.iter().zip(std::iter::repeat(nodes.0)))
                .map(|m| {
                    Match {
                        node: m.0.clone(),
                        anno: m.1.clone(),
                    }
                });

            return Box::new(it);

        }
        // if pattern is invalid return empty iterator
        let empty_it = std::iter::empty::<Match>();
        Box::new(empty_it)
    }
}

impl AnnoStorage<Edge> {

    pub fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<StringID>,
        name: StringID,
        value: Option<StringID>,
    ) -> Box<Iterator<Item = Match> + 'a> {
       
       let anno_range = self.anno_range_exact(namespace, name, value);

        Box::new(
            self.by_anno
                .range(anno_range)
                .flat_map(|nodes| nodes.1.iter().zip(std::iter::repeat(nodes.0)))
                .map(|m| {
                    Match {
                        node: m.0.source.clone(),
                        anno: m.1.clone(),
                    }
                }),
        )
    }
}

pub mod c_api;

#[cfg(test)]
mod tests;
