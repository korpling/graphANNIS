
use graphannis::annis::stringstorage::*;
use libc;
use std;

#[no_mangle]
pub extern "C" fn annis_stringstorage_new() -> *mut StringStorage {
    let s = StringStorage::new();
    Box::into_raw(Box::new(s))
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_free(target: *mut StringStorage) {
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(target) };
}


#[repr(C)]
pub struct OptionalString {
    pub valid: libc::c_int,
    pub value: *const libc::c_char,
    pub length: libc::size_t,
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_str(target: *const StringStorage,
                                          id: libc::uint32_t)
                                          -> OptionalString {

    let s = unsafe { &*target };
    let result = match s.str(id) {
        Some(v) => {
            OptionalString {
                valid: 1,
                value: v.as_ptr() as *const libc::c_char,
                length: v.len(),
            }
        }
        None => {
            OptionalString {
                valid: 0,
                value: std::ptr::null(),
                length: 0,
            }
        }
    };

    return result;
}

#[no_mangle]
pub extern "C" fn annis_stringstorage_add(target: *mut StringStorage,
                                          value: *mut libc::c_char)
                                          -> libc::uint32_t {
    let mut s = unsafe { &mut *target };
    let wrapped_str = unsafe { std::ffi::CStr::from_ptr(value)};

    match std::str::from_utf8(wrapped_str.to_bytes()) {
        Ok(v) => s.add(v),
        Err(_) => 0
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
