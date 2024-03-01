use super::*;

use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use tempfile::NamedTempFile;

#[test]
fn range() {
    let mut table: DiskMap<u8, bool> = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(3),
        DEFAULT_BLOCK_CACHE_CAPACITY,
        BtreeConfig::default().fixed_key_size(1).fixed_value_size(2),
    )
    .unwrap();
    table.insert(0, true).unwrap();
    table.insert(1, true).unwrap();
    table.insert(2, true).unwrap();
    table.insert(3, true).unwrap();
    table.insert(4, true).unwrap();
    table.insert(5, true).unwrap();

    // Before compaction

    // Start from beginning, exclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(0..6).collect();
    assert_eq!(
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true)
        ],
        result.unwrap()
    );

    // Start in between, exclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(3..5).collect();
    assert_eq!(vec![(3, true), (4, true)], result.unwrap());

    // Start in between, inclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(3..=5).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result.unwrap());

    // Start from beginning, but exclude start
    let result: Result<Vec<(u8, bool)>> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result.unwrap()
    );

    // Start in between and  exclude start
    let result: Result<Vec<(u8, bool)>> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .collect();
    assert_eq!(vec![(5, true)], result.unwrap());

    // Unbound end
    let result: Result<Vec<(u8, bool)>> = table.range(3..).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result.unwrap());

    // After compaction
    table.compact().unwrap();

    // Start from beginning, exclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(0..6).collect();
    assert_eq!(
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true)
        ],
        result.unwrap()
    );

    // Start in between, exclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(3..5).collect();
    assert_eq!(vec![(3, true), (4, true)], result.unwrap());

    // Start in between, inclusive end
    let result: Result<Vec<(u8, bool)>> = table.range(3..=5).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result.unwrap());

    // Start from beginning, but exclude start
    let result: Result<Vec<(u8, bool)>> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result.unwrap()
    );

    // Start in between and  exclude start
    let result: Result<Vec<(u8, bool)>> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .collect();
    assert_eq!(vec![(5, true)], result.unwrap());

    // Unbound end
    let result: Result<Vec<(u8, bool)>> = table.range(3..).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result.unwrap());
}

#[test]
fn single_table_iter() {
    let mut table: DiskMap<u8, bool> = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(3),
        DEFAULT_BLOCK_CACHE_CAPACITY,
        BtreeConfig::default().fixed_key_size(1).fixed_value_size(2),
    )
    .unwrap();
    table.insert(0, true).unwrap();
    table.insert(1, true).unwrap();
    table.insert(2, true).unwrap();
    table.insert(3, true).unwrap();
    table.insert(4, true).unwrap();
    table.insert(5, true).unwrap();

    // Serialize to sorted string table
    let tmp_path = NamedTempFile::new().unwrap();
    table.write_to(tmp_path.path()).unwrap();
    // Load as new table and check that iterating over all values works
    let loaded_table: DiskMap<u8, bool> = DiskMap::new(
        Some(tmp_path.path()),
        EvictionStrategy::MaximumItems(5),
        DEFAULT_BLOCK_CACHE_CAPACITY,
        BtreeConfig::default().fixed_key_size(1).fixed_value_size(2),
    )
    .unwrap();
    let items: Result<Vec<_>> = loaded_table.iter().unwrap().collect();
    let items = items.unwrap();
    assert_eq!(
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true)
        ],
        items
    );
}

#[test]
fn known_key() {
    let test_key = "DsfbaAGn".to_string();

    let mut table = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(5),
        DEFAULT_BLOCK_CACHE_CAPACITY,
        BtreeConfig::default(),
    )
    .unwrap();
    table.insert(test_key.clone(), "Test".to_string()).unwrap();
    // populate with names
    for _ in 0..100 {
        let last_name: String = LastName(EN).fake();
        let first_name: String = FirstName(EN).fake();
        if test_key != last_name {
            table.insert(last_name, first_name).unwrap();
        }
    }

    // check before compaction both with get() and range()
    assert_eq!(
        "Test",
        table.get(&test_key).unwrap().unwrap_or_default().as_str()
    );
    assert_eq!(true, table.contains_key(&test_key).unwrap());

    // compact and check again
    table.compact().unwrap();
    assert_eq!(
        "Test",
        table.get(&test_key).unwrap().unwrap_or_default().as_str()
    );
    assert_eq!(true, table.contains_key(&test_key).unwrap());
}

#[test]
fn unknown_key() {
    let test_key = "DsfbaAGn".to_string();

    let mut table = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(5),
        DEFAULT_BLOCK_CACHE_CAPACITY,
        BtreeConfig::default(),
    )
    .unwrap();
    // populate with names
    for _ in 0..100 {
        let last_name: String = LastName(EN).fake();
        let first_name: String = FirstName(EN).fake();
        if test_key != last_name {
            table.insert(last_name, first_name).unwrap();
        }
    }

    // check before compaction both with get() and range()
    assert_eq!(None, table.get(&test_key).unwrap());
    assert_eq!(
        true,
        table
            .range(test_key.clone()..=test_key.clone())
            .next()
            .is_none()
    );
    assert_eq!(false, table.contains_key(&test_key).unwrap());

    // compact and check again
    table.compact().unwrap();
    assert_eq!(None, table.get(&test_key).unwrap());
    assert_eq!(
        true,
        table
            .range(test_key.clone()..=test_key.clone())
            .next()
            .is_none()
    );
    assert_eq!(false, table.contains_key(&test_key).unwrap());
}
