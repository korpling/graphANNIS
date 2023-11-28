use crate::errors::{GraphAnnisCoreError, Result};
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
        self.insert_shared(val)
    }

    pub fn insert_shared(&mut self, val: Arc<T>) -> Result<usize> {
        if let Some(existing_idx) = self.by_value.get(&val) {
            return Ok(*existing_idx);
        }

        // non-existing: add a new value

        // if array is still small enough, just add the value to the end
        let id = if let Some(slot) = self.empty_slots.pop() {
            self.by_id[slot] = Some(val.clone());
            slot
        } else if self.by_id.len() < usize::MAX {
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
        self.by_value.get(val).copied()
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
    fn reuse_id() {
        let mut s = SymbolTable::<String>::new();

        let id1 = s.insert("abc".to_owned()).unwrap();
        assert_eq!(1, s.len());
        let val = s.get_value(id1).unwrap();
        assert_eq!("abc", val.as_str());

        assert_eq!(1, s.by_id.len());
        assert_eq!(1, s.by_value.len());

        assert_eq!(Some(val), s.by_id[id1]);
        assert_eq!(0, s.empty_slots.len());

        s.remove(0);

        assert_eq!(1, s.by_id.len());
        assert_eq!(None, s.by_id[0]);

        assert_eq!(0, s.by_value.len());
        assert_eq!(vec![id1], s.empty_slots);

        let id2 = s.insert("abc".to_owned()).unwrap();
        // Inserting a new value must use the empty slot
        assert_eq!(id1, id2);
        let val = s.get_value(id2).unwrap();

        assert_eq!(1, s.by_id.len());
        assert_eq!(1, s.by_value.len());

        assert_eq!(Some(val), s.by_id[id2]);
        assert_eq!(0, s.empty_slots.len());
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
