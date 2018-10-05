use serde::{Serialize, Deserialize};
use rustc_hash::{FxHashMap};
use std;
use std::sync::{Arc};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocShallowSizeOf};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SymbolTable<T>
where T: Eq + Hash + Clone + Default {
    by_id: Vec<Arc<T>>,
    #[serde(skip)]
    by_value: FxHashMap<Arc<T>, usize>,
}

impl<T> MallocSizeOf for SymbolTable<T> 
where T: Eq + Hash + Clone + Default + MallocSizeOf {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut string_size : usize = 0;
        // measure the size of all items and add the overhead of the Arc (two counter fields)
        for s in self.by_id.iter() {
            string_size += std::mem::size_of::<Arc<T>>() + s.size_of(ops);
        } 

        // add the size of the vector pointer, the hash map and the strings
        string_size 
        + (self.by_id.len() * std::mem::size_of::<usize>())
        + self.by_value.shallow_size_of(ops)
    }
}

impl<T> SymbolTable<T>
where for<'de> T: Eq + Hash + Clone + Serialize + Deserialize<'de> + Default {
    pub fn new() -> SymbolTable<T> {
        let by_id = Vec::default();
        SymbolTable {
            by_id: by_id,
            by_value: FxHashMap::default(),
        }
    }

    

    pub fn add(&mut self, val: Arc<T>) -> usize {
        {
            if let Some(existing_idx) = self.by_value.get(&val) {
                return *existing_idx;
            }
        }
        // non-existing: add a new value

        // if array is still small enough, just add the value to the end
        let id = if self.by_id.len() < usize::max_value() {
            self.by_id.push(val.clone());
            self.by_id.len()-1
        } else {
            // TODO use WeakRefs in the array and find an empty spot
            // for i in 0..StringID::MAX {
            // }

            // TODO if no empty place found, return an error, do not panic
            panic!("Too man unique items added to symbol table");
        };
        self.by_value.insert(val, id);

        return id;
    }

    pub fn get_value(&self, id: usize) -> Option<Arc<T>> {
        if id < self.by_id.len() {
            return Some(self.by_id[id].clone());
        }
        return None;
    }

    pub fn get_symbol(&self, val: &T) -> Option<usize> {
        return self.by_value.get(val).cloned();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        return self.by_id.len();
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_value.clear();
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = SymbolTable::<String>::new();
        let id1 = s.add(Arc::from("abc".to_owned()));
        let id2 = s.add(Arc::from("def".to_owned()));
        let id3 = s.add(Arc::from("def".to_owned()));

        assert_eq!(2, s.len());

        assert_eq!(id2, id3);

        {
            let x = s.get_value(id1);
            match x {
                Some(v) => assert_eq!("abc", v.as_ref()),
                None => panic!("Did not find string"),
            }
        }
        s.clear();
        assert_eq!(0, s.len());
    }

    #[test]
    fn insert_clear_insert_get() {
        let mut s = SymbolTable::<String>::new();

        s.add(Arc::from("abc".to_owned()));
        assert_eq!(1, s.len());
        s.clear();
        assert_eq!(0, s.len());
        s.add(Arc::from("abc".to_owned()));
        assert_eq!(1, s.len());    
    }
}

