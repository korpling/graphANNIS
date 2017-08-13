use std::collections::HashMap;
use std::collections::BTreeMap;

pub struct StringStorage {
    by_id: HashMap<u32, String>,
    by_value: BTreeMap<String, u32>,
}

impl StringStorage {
    pub fn new() -> StringStorage {
        StringStorage {
            by_id: HashMap::new(),
            by_value: BTreeMap::new(),
        }
    }

    pub fn str(&self, id: u32) -> Option<&String> {
        return self.by_id.get(&id);
    }

    pub fn add(&mut self, val: &str) -> u32 {
        {
            let existing = self.by_value.get(val);
            if existing.is_some() {
                return *(existing.unwrap());
            }
        }
        // non-existing: add a new value
        let mut id = self.by_id.len() as u32 + 1; // since 0 is taken as ANY value begin with 1
        while self.by_id.get(&id).is_some() {
            id = id + 1;
        }
        // add the new entry to both maps
        self.by_id.insert(id, String::from(val));
        self.by_value.insert(String::from(val), id);

        return id;
    }

    pub fn find_id(&self, val: &str) -> Option<&u32> {
        return self.by_value.get(&String::from(val));
    }

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
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut s = StringStorage::new();
        let id1 = s.add("abc");
        let id2 = s.add("def");
        let id3 = s.add("def");

        assert_eq!(2, s.len());

        assert_eq!(id2, id3);

        {
            let x = s.str(id1);
            match x {
                Some(v) => assert_eq!("abc", v),
                None => panic!("Did not find string"),
            }
        }
        s.clear();
        assert_eq!(0, s.len());
    }
}
