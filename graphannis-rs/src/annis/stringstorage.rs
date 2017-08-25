use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use regex::Regex;
use std;
use bincode;

#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
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

    pub fn find_regex(&self, val: &str) -> BTreeSet<&u32> {
        let mut result = BTreeSet::new();

        // we always want to match the complete string
        let mut full_match_pattern = String::new();
        full_match_pattern.push_str(r"\A");
        full_match_pattern.push_str(val);
        full_match_pattern.push_str(r"\z");

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
        for (s, _) in &self.by_value {
            sum += s.len();
        }
        return (sum as f64) / (self.by_value.len() as f64);
    }

    pub fn len(&self) -> usize {
        return self.by_id.len();
    }

    pub fn clear(&mut self) {
        self.by_id.clear();
        self.by_value.clear();
    }

    pub fn save_to_file(&self, path: &str) {
        let f = std::fs::File::create(path).unwrap();

        let mut buf_writer = std::io::BufWriter::new(f);

        bincode::serialize_into(&mut buf_writer, self, bincode::Infinite);
    }

    pub fn load_from_file(&mut self, path: &str) {

        let f = std::fs::File::open(path).unwrap();
        let mut buf_reader = std::io::BufReader::new(f);

        let loaded: Result<StringStorage, _> = bincode::deserialize_from(&mut buf_reader,
                                                                         bincode::Infinite);
        if loaded.is_ok() {
            *self = loaded.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir;

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

    #[test]
    fn serialization() {
        let mut s = StringStorage::new();
        s.add("abc");
        s.add("def");

        if let Ok(tmp) = tempdir::TempDir::new("annis_test") {
            let file_path = tmp.path().join("out.storage");
            let file_path_str = file_path.to_str().unwrap();
            s.save_to_file(&file_path_str);

            s.clear();

            s.load_from_file(&file_path_str);
            assert_eq!(2, s.len());
        }
    }
}
