use std::collections::HashMap;
use std::collections::BTreeMap;

pub struct StringStorage {
    by_id: HashMap<u32,String>,
    by_value: BTreeMap<String, u32>
}

impl StringStorage {

    pub fn new() -> StringStorage {
        StringStorage {
            by_id: HashMap::new(),
            by_value: BTreeMap::new()
        }
    }

    pub fn str(&self, id: u32) -> Option<&String> {
        return self.by_id.get(&id);
    }

    pub fn add(&mut self, val: String) -> u32 {
        {
            let existing = self.by_value.get(&val);
            if existing.is_some() {
                return *(existing.unwrap());
            }
        }
        // non-existing: add a new value
        let mut id = self.by_id.len() as u32 + 1; // since 0 is taken as ANY value begin with 1
        while self.by_id.get(&id).is_some() {
            id = id +1;
        }
        // add the new entry to both maps
        self.by_id.insert(id, val.clone());
        self.by_value.insert(val.clone(), id);
    
        return id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = StringStorage::new();
        let first_id = s.add("abc".to_string());
        
        let x = s.str(first_id);
        match x {
            Some(v) => assert_eq!("abc", v),
            None => panic!("Did not find string"),
        }
    }
}
