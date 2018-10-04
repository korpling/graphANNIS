use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use rustc_hash::{FxHashMap};
use std;
use std::sync::{Arc};
use bincode;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocShallowSizeOf};
use num::ToPrimitive;
use std::path::{PathBuf};
use std::hash::Hash;
use errors::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SymbolTable<T>
where T: Eq + Hash + Clone + Debug {
    by_id: Vec<Arc<T>>,
    #[serde(skip)]
    by_value: FxHashMap<Arc<T>, usize>,
}

impl<T> MallocSizeOf for SymbolTable<T> 
where T: Eq + Hash + Clone + Debug + MallocSizeOf {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut string_size : usize = 0;
        // measure the size of all strings and add the overhead of the Arc (two counter fields)
        for s in self.by_id.iter() {
            string_size += (2*std::mem::size_of::<usize>()) + s.size_of(ops);
        } 

        // add the size of the vector pointer, the hash map and the strings
        string_size 
        + (self.by_id.len() * std::mem::size_of::<usize>())
        + self.by_value.shallow_size_of(ops)
    }
}

impl<T> SymbolTable<T>
where for<'de> T: Eq + Hash + Clone + Debug + Serialize + Deserialize<'de> + Default {
    pub fn new() -> SymbolTable<T> {
        let by_id = Vec::default();
        SymbolTable {
            by_id: by_id,
            by_value: FxHashMap::default(),
        }
    }

    pub fn get_value(&self, id: usize) -> Option<&T> {
        let id = id.to_usize()?;
        if id < self.by_id.len() {
            return Some(self.by_id[id].as_ref());
        }
        return None;
    }

    pub fn add(&mut self, val: T) -> usize {
        {
            if let Some(existing_idx) = self.by_value.get(&val) {
                return *existing_idx;
            }
        }
        // non-existing: add a new value

        let val : Arc<T> = Arc::from(val);

        // if array is still small enough, just add the value to the end
        let id = if self.by_id.len() < usize::max_value() {
            self.by_id.push(val.clone());
            self.by_id.len()-1
        } else {
            // TODO use WeakRefs in the array and find an empty spot
            // for i in 0..StringID::MAX {
            // }

            // TODO if no empty place found, return an error, do not panic
            panic!("Too man unique strings added to database");
        };
        self.by_value.insert(val, id);

        return id;
    }

    pub fn get_id(&self, val: &T) -> Option<&usize> {
        return self.by_value.get(val);
    }

    pub fn len(&self) -> usize {
        return self.by_id.len();
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_value.clear();
    }

    pub fn save_to_file(&self, path: &str) -> bool {

        let f = std::fs::File::create(path).unwrap();

        let mut buf_writer = std::io::BufWriter::new(f);

        bincode::serialize_into(&mut buf_writer, self).is_ok()
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<()> {

        // always remove all entries first, so even if there is an error the string storage is empty
        self.clear();

        let path = PathBuf::from(path);
        let f = std::fs::File::open(path.clone()).chain_err(|| {
            format!(
                "Could not load string storage from file {}",
                path.to_string_lossy()
            )
        })?;
        let mut reader = std::io::BufReader::new(f);
        *self  = bincode::deserialize_from(&mut reader)?;

        // restore the by_value map and make sure the smart pointers point to the same instance
        self.by_value.reserve(self.by_id.len());
        for i in 0..self.by_id.len() {
            self.by_value.insert(self.by_id[i].clone(), i);
        }

        Ok(())

    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = SymbolTable::<String>::new();
        let id1 = s.add("abc".to_owned());
        let id2 = s.add("def".to_owned());
        let id3 = s.add("def".to_owned());

        assert_eq!(2, s.len());

        assert_eq!(id2, id3);

        {
            let x = s.get_value(id1);
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

        s.add("abc".to_owned());
        assert_eq!(1, s.len());
        s.clear();
        assert_eq!(0, s.len());
        s.add("abc".to_owned());
        assert_eq!(1, s.len());    
    }

    #[test]
    fn serialization() {
        let mut s = SymbolTable::<String>::new();
        s.add("abc".to_owned());
        s.add("def".to_owned());

        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            let file_path = tmp.path().join("out.storage");
            let file_path_str = file_path.to_str().unwrap();
            s.save_to_file(&file_path_str);

            s.clear();

            s.load_from_file(&file_path_str).unwrap();
            assert_eq!(2, s.len());
        }
    }
}

