use std::collections::{BTreeMap, BTreeSet};


#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct AnnotationKey {
    pub name: u32,
    pub ns: u32,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ContainerAnnotationKey<T: Ord> {
    pub item: T,
    pub key: AnnotationKey,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct Annotation {
    pub key: AnnotationKey,
    pub val: u32,
}

pub struct AnnoStorage<T: Ord> {
    by_container: BTreeMap<ContainerAnnotationKey<T>, u32>,
    by_anno: BTreeMap<Annotation, BTreeSet<T>>,
}

impl<T: Ord + Clone> AnnoStorage<T> {
    pub fn new() -> AnnoStorage<T> {
        AnnoStorage {
            by_container: BTreeMap::new(),
            by_anno: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, item: T, anno: &Annotation) {
        self.by_container.insert(
            ContainerAnnotationKey {
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
            .insert(item.clone());
    }

    pub fn len(&self) -> usize {
        self.by_container.len()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn insert_same_anno() {
        let mut a: AnnoStorage<u32> = AnnoStorage::new();
        a.add(
            1,
            &Annotation {
                key: AnnotationKey { name: 1, ns: 1 },
                val: 123,
            },
        );
        a.add(
            2,
            &Annotation {
                key: AnnotationKey { name: 1, ns: 1 },
                val: 123,
            },
        );
        a.add(
            3,
            &Annotation {
                key: AnnotationKey { name: 1, ns: 1 },
                val: 123,
            },
        );

        assert_eq!(3, a.len());
    }
}
