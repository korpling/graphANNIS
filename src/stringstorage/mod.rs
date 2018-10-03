use {StringID};
use rustc_hash::{FxHashMap, FxHashSet};
use regex::Regex;
use std;
use std::sync::{Arc};
use bincode;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocShallowSizeOf};
use num::ToPrimitive;
use std::path::{PathBuf};
use errors::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StringStorage {
    by_id: Vec<Arc<String>>,
    #[serde(skip)]
    by_value: FxHashMap<Arc<String>, StringID>,
}

impl MallocSizeOf for StringStorage {
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

impl StringStorage {
    pub fn new() -> StringStorage {
        let mut by_id = Vec::default();
        // since 0 is taken as ANY value begin with 1
        by_id.push(Arc::from(String::default()));
        StringStorage {
            by_id: by_id,
            by_value: FxHashMap::default(),
        }
    }

    pub fn str(&self, id: StringID) -> Option<&String> {
        let id = id.to_usize()?;
        if id < self.by_id.len() {
            return Some(self.by_id[id].as_ref());
        }
        return None;
    }

    pub fn add(&mut self, val: &str) -> StringID {
        let val = val.to_owned();
        {
            if let Some(existing_idx) = self.by_value.get(&val) {
                return *existing_idx;
            }
        }
        // non-existing: add a new value

        let val : Arc<String> = Arc::from(val);

        // if array is still small enough, just add the value to the end
        let id = if self.by_id.len() < (StringID::max_value() as usize) {
            self.by_id.push(val.clone());
            self.by_id.len()-1
        } else {
            // TODO use WeakRefs in the array and find an empty spot
            // for i in 0..StringID::MAX {
            // }

            // TODO if no empty place found, return an error, do not panic
            panic!("Too man unique strings added to database");
        };
        let id = id as StringID;
        self.by_value.insert(val, id);

        return id;
    }

    pub fn find_id(&self, val: &str) -> Option<&StringID> {
        return self.by_value.get(&String::from(val));
    }

    pub fn find_regex(&self, val: &str) -> FxHashSet<&StringID> {
        let mut result = FxHashSet::default();

        // we always want to match the complete string
        let full_match_pattern = ::util::regex_full_match(val);

        let compiled_result = Regex::new(&full_match_pattern);
        if compiled_result.is_ok() {
            let re = compiled_result.unwrap();

            // check all values
            // TODO: get a valid prefix somehow and check only a range of strings, not all
            for (s, id) in &self.by_value {
                if re.is_match(s) {
                    result.insert(id);
                }
            }
        }

        return result;
    }

    pub fn avg_length(&self) -> f64 {
        let mut sum: usize = 0;
        for s in &self.by_id {
            sum += s.len();
        }
        return (sum as f64) / (self.by_id.len() as f64);
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
            self.by_value.insert(self.by_id[i].clone(), i as StringID);
        }

        Ok(())

    }
}

#[cfg(test)]
mod tests;

