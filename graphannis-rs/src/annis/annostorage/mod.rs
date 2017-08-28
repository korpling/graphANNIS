use annis::StringID;
use std::collections::{BTreeMap, BTreeSet};


#[derive(Eq, PartialEq, PartialOrd, Ord, Clone,Debug)]
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

        let found_range = self.by_container.range(ContainerAnnoKey {
            item: item.clone(),
            key: min_key,
        }..ContainerAnnoKey {
            item: item.clone(),
            key: max_key,
        });

        let mut result = vec![];
        for (k, &v) in found_range {
            result.push(Annotation{key: k.clone().key, val: v});
        }

        return result;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use annis::NodeID;

    #[test]
    fn insert_same_anno() {
        let test_anno = Annotation {
            key: AnnoKey { name: 1, ns: 1 },
            val: 123,
        };
        let mut a: AnnoStorage<NodeID> = AnnoStorage::new();
        a.insert(1, test_anno.clone());
        a.insert(1, test_anno.clone());
        a.insert(2, test_anno.clone());
        a.insert(3, test_anno);

        assert_eq!(3, a.len());
        assert_eq!(3, a.by_container.len());
        assert_eq!(1, a.by_anno.len());
        assert_eq!(1, a.anno_keys.len());

        assert_eq!(123, a.get(&3, &AnnoKey { name: 1, ns: 1 }).unwrap().clone());
    }

    #[test]
    fn get_all_for_node() {
        let test_anno1 = Annotation {
            key: AnnoKey { name: 1, ns: 1 },
            val: 123,
        };
        let test_anno2 = Annotation {
            key: AnnoKey { name: 2, ns: 2 },
            val: 123,
        };
        let test_anno3 = Annotation {
            key: AnnoKey { name: 3, ns: 1 },
            val: 123,
        };

        let mut a: AnnoStorage<NodeID> = AnnoStorage::new();
        a.insert(1, test_anno1.clone());
        a.insert(1, test_anno2.clone());
        a.insert(1, test_anno3.clone());

        assert_eq!(3, a.len());

        let all = a.get_all(&1);
        assert_eq!(3, all.len());

        assert_eq!(test_anno1, all[0]);
        assert_eq!(test_anno2, all[1]);
        assert_eq!(test_anno3, all[2]);


    }
}
