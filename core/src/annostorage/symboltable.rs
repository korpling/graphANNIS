use crate::errors::{GraphAnnisCoreError, Result};
use crate::malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use crate::util::memory_estimation::shallow_size_of_fxhashmap;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SymbolTable<T>
where
    T: Eq + Hash + Ord + Clone + Default,
{
    by_id: Vec<Option<Arc<T>>>,
    #[serde(skip)]
    by_value: FxHashMap<Arc<T>, usize>,
    empty_slots: Vec<usize>,
}

impl<T> MallocSizeOf for SymbolTable<T>
where
    T: Eq + Hash + Ord + Clone + Default + MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut size: usize = 0;
        // measure the size of all items and add the overhead of the Arc (two counter fields)
        for s in &self.by_id {
            size += std::mem::size_of::<Arc<T>>() + s.size_of(ops);
        }

        // add the size of the by_value values, the hash map itself and the empty slot vector
        size + (self.by_id.len() * std::mem::size_of::<usize>())
            + shallow_size_of_fxhashmap(&self.by_value, ops)
            + self.empty_slots.size_of(ops)
    }
}

impl<T> SymbolTable<T>
where
    for<'de> T: Eq + Hash + Ord + Clone + Serialize + Deserialize<'de> + Default,
{
    pub fn new() -> SymbolTable<T> {
        let by_id = Vec::default();
        SymbolTable {
            by_id,
            by_value: FxHashMap::default(),
            empty_slots: Vec::default(),
        }
    }

    pub fn after_deserialization(&mut self) {
        // restore the by_value map and make sure the smart pointers point to the same instance
        //self.by_value.reserve(self.by_id.len());
        for i in 0..self.by_id.len() {
            if let Some(ref existing) = self.by_id[i] {
                self.by_value.insert(existing.clone(), i);
            }
        }
    }

    pub fn insert(&mut self, val: T) -> Result<usize> {
        let val = Arc::from(val);
        {
            if let Some(existing_idx) = self.by_value.get(&val) {
                return Ok(*existing_idx);
            }
        }
        // non-existing: add a new value

        // if array is still small enough, just add the value to the end
        let id = if let Some(slot) = self.empty_slots.pop() {
            slot
        } else if self.by_id.len() < usize::max_value() {
            self.by_id.push(Some(val.clone()));
            self.by_id.len() - 1
        } else {
            return Err(GraphAnnisCoreError::SymbolTableOverflow);
        };
        self.by_value.insert(val, id);

        Ok(id)
    }

    pub fn remove(&mut self, symbol: usize) -> Option<Arc<T>> {
        if symbol < self.by_id.len() {
            let existing = self.by_id[symbol].clone();
            self.by_id[symbol] = None;

            if let Some(existing) = existing {
                self.by_value.remove(&existing);

                self.empty_slots.push(symbol);

                return Some(existing);
            }
        }
        None
    }

    pub fn get_value(&self, id: usize) -> Option<Arc<T>> {
        if id < self.by_id.len() {
            if let Some(ref val) = self.by_id[id] {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn get_value_ref(&self, id: usize) -> Option<&T> {
        if id < self.by_id.len() {
            if let Some(ref val) = self.by_id[id] {
                return Some(val.as_ref());
            }
        }
        None
    }

    pub fn get_symbol(&self, val: &T) -> Option<usize> {
        self.by_value.get(val).cloned()
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_value.clear();
        self.empty_slots.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = SymbolTable::<String>::new();
        let id1 = s.insert("abc".to_owned()).unwrap();
        let id2 = s.insert("def".to_owned()).unwrap();
        let id3 = s.insert("def".to_owned()).unwrap();

        assert_eq!(2, s.len());

        assert_eq!(id2, id3);

        {
            let x = s.get_value_ref(id1);
            match x {
                Some(v) => assert_eq!("abc", v),
                None => panic!("Did not find string"),
            }
        }
        s.clear();
        assert_eq!(0, s.len());
    }

    #[test]
    fn insert_clear_insert_get() {
        let mut s = SymbolTable::<String>::new();

        s.insert("abc".to_owned()).unwrap();
        assert_eq!(1, s.len());
        s.clear();
        assert_eq!(0, s.len());
        s.insert("abc".to_owned()).unwrap();
        assert_eq!(1, s.len());
    }
}
