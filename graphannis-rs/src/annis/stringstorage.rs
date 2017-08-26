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

    #[allow(unused_must_use)]
    pub fn save_to_file(&self, path: &str) {

        let f = std::fs::File::create(path).unwrap();

        let mut buf_writer = std::io::BufWriter::new(f);

        bincode::serialize_into(&mut buf_writer, self, bincode::Infinite);
    }

    pub fn load_from_file(&mut self, path: &str) {

        // always remove all entries first, so even if there is an error the string storage is empty
        self.clear();

        let f = std::fs::File::open(path);
        if f.is_ok() {
            let mut buf_reader = std::io::BufReader::new(f.unwrap());

            let loaded: Result<StringStorage, _> = bincode::deserialize_from(&mut buf_reader,
                                                                             bincode::Infinite);
            if loaded.is_ok() {
                *self = loaded.unwrap();
            }
        }
    }

    pub fn estimate_memory_size(&self) -> usize {

        return ::annis::util::memory_estimation::hash_map_size(&self.by_id) +
               ::annis::util::memory_estimation::btree_map_size(&self.by_value);
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    extern crate tempdir;

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

pub mod c_api {

    use libc;
    use libc::{c_char};
    use std::ffi::CStr;
    use super::*;

    #[repr(C)]
    pub struct annis_StringStoragePtr(StringStorage);

    #[repr(C)]
    pub struct annis_OptionalString {
        pub valid: libc::c_int,
        pub value: *const c_char,
        pub length: libc::size_t,
    }

    #[repr(C)]
    pub struct annis_Option_u32 {
        pub valid: libc::c_int,
        pub value: libc::uint32_t,
    }


    #[no_mangle]
    pub extern "C" fn annis_stringstorage_new() -> *mut annis_StringStoragePtr {
        let s = StringStorage::new();
        Box::into_raw(Box::new(annis_StringStoragePtr(s)))
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_free(target: *mut annis_StringStoragePtr) {
        // take ownership and destroy the pointer
        unsafe {assert!(!target.is_null()); Box::from_raw(target) };
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_str(target: *const annis_StringStoragePtr,
                                              id: libc::uint32_t)
                                              -> annis_OptionalString {

        let s = unsafe {assert!(!target.is_null()); &(*target).0 };
        let result = match s.str(id) {
            Some(v) => {
                annis_OptionalString {
                    valid: 1,
                    value: v.as_ptr() as *const c_char,
                    length: v.len(),
                }
            }
            None => {
                annis_OptionalString {
                    valid: 0,
                    value: std::ptr::null(),
                    length: 0,
                }
            }
        };

        return result;
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_find_id(target: *const annis_StringStoragePtr,
                                                  value: *const c_char)
                                                  -> annis_Option_u32 {
        let s = unsafe {assert!(!target.is_null()); &(*target).0 };
        let c_value = unsafe {
            assert!(!value.is_null());
            CStr::from_ptr(value)
        };

        let result = match c_value.to_str() {
            Ok(v) => {
                match s.find_id(v) {
                    Some(x) => {
                        annis_Option_u32 {
                            valid: 1,
                            value: *x,
                        }
                    }
                    None => {
                        annis_Option_u32 {
                            valid: 0,
                            value: 0,
                        }
                    }
                }
            }
            Err(_) => {
                annis_Option_u32 {
                    valid: 0,
                    value: 0,
                }
            }
        };

        return result;
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_add(target: *mut annis_StringStoragePtr,
                                              value: *const c_char)
                                              -> libc::uint32_t {
        let mut s = unsafe {assert!(!target.is_null()); &mut (*target).0 };
        let c_value = unsafe {
            assert!(!value.is_null());
            CStr::from_ptr(value)
        };

        match c_value.to_str() {
            Ok(v) => s.add(v),
            Err(_) => 0,
        }
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_clear(target: *mut annis_StringStoragePtr) {
        let mut s = unsafe {assert!(!target.is_null()); &mut (*target).0 };
        s.clear();
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_len(target: *const annis_StringStoragePtr)
                                              -> libc::size_t {
        let s = unsafe {assert!(!target.is_null()); &(*target).0 };
        return s.len();
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_avg_length(target: *const annis_StringStoragePtr)
                                                     -> libc::c_double {
        let s = unsafe {assert!(!target.is_null()); &(*target).0 };
        return s.avg_length();
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_save_to_file(target: *const annis_StringStoragePtr,
                                                       path: *const c_char) {
        let s = unsafe {assert!(!target.is_null()); &(*target).0 };
        let c_path = unsafe {
            assert!(!path.is_null());
            CStr::from_ptr(path)
        };
        let safe_path = c_path.to_str();
        if safe_path.is_ok() {
            s.save_to_file(safe_path.unwrap());
        }
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_load_from_file(target: *mut annis_StringStoragePtr,
                                                         path: *const c_char) {
        let s = unsafe {assert!(!target.is_null()); &mut (*target).0 };
        let c_path = unsafe {
            assert!(!path.is_null());
            CStr::from_ptr(path)
        };
        let safe_path = c_path.to_str();
        if safe_path.is_ok() {
            s.load_from_file(safe_path.unwrap());
        }
    }

    #[no_mangle]
    pub extern "C" fn annis_stringstorage_estimate_memory(target: *const annis_StringStoragePtr)
                                                          -> libc::size_t {
        let s = unsafe {assert!(!target.is_null()); &(*target).0 };

        return s.estimate_memory_size();
    }
}