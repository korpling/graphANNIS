use std::collections::{BTreeMap, BTreeSet};


#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct AnnoKey {
    pub name: u32,
    pub ns: u32,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ContainerAnnoKey<T: Ord> {
    pub item: T,
    pub key: AnnoKey,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct Annotation {
    pub key: AnnoKey,
    pub val: u32,
}

pub struct AnnoStorage<T: Ord> {
    by_container: BTreeMap<ContainerAnnoKey<T>, u32>,
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

    pub fn insert(&mut self, item: T, anno: &Annotation) {
        self.by_container.insert(
            ContainerAnnoKey {
                item: item.clone(),
                key: anno.key.clone(),
            },
            anno.val,
        );

        // inserts a new element into the set
        // if set is not existing yet it is created
        self.by_anno
            .entry(anno.clone())
            .or_insert(BTreeSet::new())
            .insert(item);

        let anno_key_entry = self.anno_keys.entry(anno.clone().key).or_insert(0);
        *anno_key_entry = *anno_key_entry+1;
    }

    pub fn remove(&mut self, item: T) {
        unimplemented!();
    }

    pub fn len(&self) -> usize {
        self.by_container.len()
    }

    pub fn get(&self, item: T, key: &AnnoKey) -> Option<&u32> {
        let container_key = ContainerAnnoKey{item: item, key: key.clone()};

        self.by_container.get(&container_key)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn insert_same_anno() {
        let test_anno = Annotation {
                key: AnnoKey { name: 1, ns: 1 },
                val: 123,
            };
        let mut a: AnnoStorage<u32> = AnnoStorage::new();
        a.insert(1, &test_anno);
        a.insert(1, &test_anno);
        a.insert(2, &test_anno);
        a.insert(3, &test_anno);

        assert_eq!(3, a.len());
        assert_eq!(3, a.by_container.len());
        assert_eq!(1, a.by_anno.len());
        assert_eq!(1, a.anno_keys.len());

        assert_eq!(123, a.get(3, &AnnoKey{name: 1, ns: 1}).unwrap().clone());

    }
}
