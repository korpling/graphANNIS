use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std;

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
pub struct AnnoKey {
    pub name: StringID,
    pub ns: StringID,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
pub struct ContainerAnnoKey<T: Ord> {
    pub item: T,
    pub key: AnnoKey,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug)]
pub struct Annotation {
    pub key: AnnoKey,
    pub val: StringID,
}

#[derive(Debug)]
pub struct Match {
    node: NodeID,
    anno: Annotation,
}

pub struct AnnoStorage<T: Ord> {
    by_container: BTreeMap<ContainerAnnoKey<T>, StringID>,
    by_anno: BTreeMap<Annotation, BTreeSet<T>>,
    /// Maps a distinct annotation key to the number of keys available.
    anno_keys: BTreeMap<AnnoKey, usize>,
}


impl<T: Ord + Clone> AnnoStorage<T> {
    pub fn new() -> AnnoStorage<T> {
        AnnoStorage {
            by_container: BTreeMap::new(),
            by_anno: BTreeMap::new(),
            anno_keys: BTreeMap::new(),
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
}

impl<'a> AnnoStorage<NodeID> {
    pub fn exact_annokey_search(&'a self, namespace : Option<StringID>, name : StringID) -> Box<Iterator<Item = Match >+ 'a> {
        let ns_pair = match namespace {
            Some(v) => (v,v),
            None => (StringID::min_value(), StringID::max_value()),
        };

        let anno_min = Annotation {
            key: AnnoKey{name, ns: ns_pair.0},
            val: 0,
        };
        let anno_max = Annotation {
            key: AnnoKey{name, ns: ns_pair.1},
            val: NodeID::max_value(),
        };

        Box::new(
            self.by_anno
                .range(anno_min..anno_max)
                .flat_map(|nodes| nodes.1.iter().zip(std::iter::repeat(nodes.0)))
                .map(|m| {
                    Match {
                        node: m.0.clone(),
                        anno: m.1.clone(),
                    }
                }),
        )
    }
}

pub mod c_api;

#[cfg(test)]
mod tests;
