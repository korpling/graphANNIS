use super::*;

use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;

#[test]
fn range() {
    let mut table = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(3),
        Some(DEFAULT_MAX_NUMBER_OF_TABLES),
        DEFAULT_BLOCK_CACHE_CAPACITY,
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
    let result: Vec<(u8, bool)> = table.range(0..6).collect();
    assert_eq!(
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true)
        ],
        result
    );

    // Start in between, exclusive end
    let result: Vec<(u8, bool)> = table.range(3..5).collect();
    assert_eq!(vec![(3, true), (4, true)], result);

    // Start in between, inclusive end
    let result: Vec<(u8, bool)> = table.range(3..=5).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);

    // Start from beginning, but exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result
    );

    // Start in between and  exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .collect();
    assert_eq!(vec![(5, true)], result);

    // Unbound end
    let result: Vec<(u8, bool)> = table.range(3..).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);

    // After compaction
    table.compact().unwrap();

    // Start from beginning, exclusive end
    let result: Vec<(u8, bool)> = table.range(0..6).collect();
    assert_eq!(
        vec![
            (0, true),
            (1, true),
            (2, true),
            (3, true),
            (4, true),
            (5, true)
        ],
        result
    );

    // Start in between, exclusive end
    let result: Vec<(u8, bool)> = table.range(3..5).collect();
    assert_eq!(vec![(3, true), (4, true)], result);

    // Start in between, inclusive end
    let result: Vec<(u8, bool)> = table.range(3..=5).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);

    // Start from beginning, but exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result
    );

    // Start in between and  exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .collect();
    assert_eq!(vec![(5, true)], result);

    // Unbound end
    let result: Vec<(u8, bool)> = table.range(3..).collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);
}

#[test]
fn known_key() {
    let test_key = "DsfbaAGn".to_string();

    let mut table = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(5),
        Some(DEFAULT_MAX_NUMBER_OF_TABLES),
        DEFAULT_BLOCK_CACHE_CAPACITY,
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
    assert_eq!(Some("Test".to_string()), table.try_get(&test_key).unwrap());
    assert_eq!(true, table.try_contains_key(&test_key).unwrap());

    // compact and check again
    table.compact().unwrap();
    assert_eq!(Some("Test".to_string()), table.try_get(&test_key).unwrap());
    assert_eq!(true, table.try_contains_key(&test_key).unwrap());
}

#[test]
fn unknown_key() {
    let test_key = "DsfbaAGn".to_string();

    let mut table = DiskMap::new(
        None,
        EvictionStrategy::MaximumItems(5),
        Some(DEFAULT_MAX_NUMBER_OF_TABLES),
        DEFAULT_BLOCK_CACHE_CAPACITY,
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
    assert_eq!(None, table.try_get(&test_key).unwrap());
    assert_eq!(
        None,
        table.range(test_key.clone()..=test_key.clone()).next()
    );
    assert_eq!(false, table.try_contains_key(&test_key).unwrap());

    // compact and check again
    table.compact().unwrap();
    assert_eq!(None, table.try_get(&test_key).unwrap());
    assert_eq!(
        None,
        table.range(test_key.clone()..=test_key.clone()).next()
    );
    assert_eq!(false, table.try_contains_key(&test_key).unwrap());
}

#[test]
fn never_compact() {
    let mut table: DiskMap<usize, bool> =
        DiskMap::new_temporary(EvictionStrategy::MaximumItems(1), None, 1);

    // populate with 250 unique keys
    for i in 0..250 {
        table.insert(i, true).unwrap();
    }

    // With 250 entries and a maximum number of 1 item in C0, this disk map
    // should have 250 evicted disk tables.
    assert_eq!(250, table.number_of_disk_tables());

    let mut table: DiskMap<usize, bool> =
        DiskMap::new_temporary(EvictionStrategy::MaximumItems(5), None, 1);

    // populate with 1000 unique keys
    for i in 0..1000 {
        table.insert(i, true).unwrap();
    }

    // With 1000 entries and a maximum number of 5 items in C0, this disk map
    // should have 1000 / 5 = 200 evicted disk tables.
    assert_eq!(200, table.number_of_disk_tables());
}
