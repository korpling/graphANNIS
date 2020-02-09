use super::*;

use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;

#[test]
fn range() {
    let mut table = DiskMap::new(None, EvictionStrategy::MaximumItems(3)).unwrap();
    table.insert(0, true).unwrap();
    table.insert(1, true).unwrap();
    table.insert(2, true).unwrap();
    table.insert(3, true).unwrap();
    table.insert(4, true).unwrap();
    table.insert(5, true).unwrap();

    // Before compaction

    // Start from beginning, exclusive end
    let result: Vec<(u8, bool)> = table.range(0..6).unwrap().collect();
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
    let result: Vec<(u8, bool)> = table.range(3..5).unwrap().collect();
    assert_eq!(vec![(3, true), (4, true)], result);

    // Start in between, inclusive end
    let result: Vec<(u8, bool)> = table.range(3..=5).unwrap().collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);

    // Start from beginning, but exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .unwrap()
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result
    );

    // Start in between and  exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .unwrap()
        .collect();
    assert_eq!(vec![(5, true)], result);

    // Unbound end
    let result: Vec<(u8, bool)> = table
        .range(3..)
        .unwrap()
        .collect();
    assert_eq!(vec![(3, true),(4, true),(5, true)], result);

    // After compaction
    table.compact_and_flush().unwrap();

    // Start from beginning, exclusive end
    let result: Vec<(u8, bool)> = table.range(0..6).unwrap().collect();
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
    let result: Vec<(u8, bool)> = table.range(3..5).unwrap().collect();
    assert_eq!(vec![(3, true), (4, true)], result);

    // Start in between, inclusive end
    let result: Vec<(u8, bool)> = table.range(3..=5).unwrap().collect();
    assert_eq!(vec![(3, true), (4, true), (5, true)], result);

    // Start from beginning, but exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(0), Bound::Excluded(6)))
        .unwrap()
        .collect();
    assert_eq!(
        vec![(1, true), (2, true), (3, true), (4, true), (5, true)],
        result
    );

    // Start in between and  exclude start
    let result: Vec<(u8, bool)> = table
        .range((Bound::Excluded(4), Bound::Excluded(6)))
        .unwrap()
        .collect();
    assert_eq!(vec![(5, true)], result);

    // Unbound end
    let result: Vec<(u8, bool)> = table
        .range(3..)
        .unwrap()
        .collect();
    assert_eq!(vec![(3, true),(4, true),(5, true)], result);
}

#[test]
fn unknown_key() {
    let test_key = "DsfbaAGn".to_string();

    let mut table = DiskMap::new(None, EvictionStrategy::MaximumItems(5)).unwrap();
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
    assert_eq!(None, table.range(test_key.clone()..=test_key.clone()).unwrap().next());

    // compact and check again
    table.compact_and_flush().unwrap();
    assert_eq!(None, table.get(&test_key).unwrap());
    assert_eq!(None, table.range(test_key.clone()..=test_key.clone()).unwrap().next());
}

