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
fn insert_clear_insert_get() {
    let mut s = StringStorage::new();

    s.add("abc");
    assert_eq!(1, s.len());
    s.clear();
    assert_eq!(0, s.len());
    s.add("abc");
    assert_eq!(1, s.len());    
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