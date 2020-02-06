use crate::annis::errors::*;
use serde::{Deserialize, Serialize};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};

use std::collections::BTreeMap;
use std::fs::File;
use std::ops::{Bound, RangeBounds};
use std::path::{Path, PathBuf};

mod serializer;

pub use serializer::KeySerializer;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord)]
struct Entry<K, V>
where
    K: Clone + Eq + PartialEq + PartialOrd + Ord,
    V: Clone + Eq + PartialEq + PartialOrd + Ord,
{
    key: K,
    value: V,
}

pub struct DiskMap<K, V>
where
    K: 'static + KeySerializer + Send,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send,
{
    persistance_file: Option<PathBuf>,
    eviction_strategy: EvictionStrategy,
    c0: BTreeMap<K, Option<V>>,
    disk_tables: Vec<Table>,
    serialization: bincode::Config,
}

pub enum EvictionStrategy {
    MaximumItems(usize),
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        EvictionStrategy::MaximumItems(1024)
    }
}

impl<K, V> DiskMap<K, V>
where
    K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    pub fn new(
        persistance_file: Option<&Path>,
        eviction_strategy: EvictionStrategy,
    ) -> DiskMap<K, V> {
        let mut serialization = bincode::config();
        serialization.big_endian();

        DiskMap {
            eviction_strategy,
            persistance_file: persistance_file.map(|p| p.to_owned()),
            c0: BTreeMap::default(),
            disk_tables: Vec::default(),
            serialization: serialization,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        let existing = self.get(&key)?;
        self.c0.insert(key, Some(value));

        match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => {
                if self.c0.len() > n {
                    self.evict_c0()?;
                }
            }
        }
        Ok(existing)
    }

    fn evict_c0(&mut self) -> Result<tempfile::NamedTempFile> {
        let out_file = tempfile::NamedTempFile::new()?;
        let mut builder = TableBuilder::new(sstable::Options::default(), out_file.as_file());

        for (key, value) in self.c0.iter() {
            let key = key.create_key();
            builder.add(&key, &self.serialization.serialize(value)?)?;
        }
        builder.finish()?;

        let table = Table::new_from_file(sstable::Options::default(), out_file.path())?;

        self.disk_tables.push(table);

        self.c0.clear();

        Ok(out_file)
    }

    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        let existing = self.get(key)?;
        if existing.is_some() {
            // Add tombstone entry
            self.c0.insert(key.clone(), None);
        }
        Ok(existing)
    }

    pub fn clear(&mut self) {
        self.c0.clear();
        self.disk_tables.clear();
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        // Check C0 first
        if let Some(value) = self.c0.get(&key) {
            if value.is_some() {
                return Ok(value.clone());
            } else {
                // Value was explicitly deleted, do not query the disk tables
                return Ok(None);
            }
        }
        // Iterate over all disk-tables to find the entry
        let key: Vec<u8> = key.create_key();
        for table in self.disk_tables.iter().rev() {
            if let Some(value) = table.get(&key)? {
                let value: Option<V> = self.serialization.deserialize(&value)?;
                if value.is_some() {
                    return Ok(value.clone());
                } else {
                    // Value was explicitly deleted, do not query the rest of the disk tables
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub fn range<R>(&self, range: R) -> Result<Range<K, V, R>>
    where
        R: RangeBounds<K> + Clone,
    {
        let mut table_iterators: Vec<TableIterator> =
            self.disk_tables.iter().rev().map(|t| t.iter()).collect();
        match range.start_bound() {
            Bound::Included(start) => {
                let start = start.create_key();
                for ti in table_iterators.iter_mut() {
                    ti.seek(&start);
                }
            }
            Bound::Excluded(start_bound) => {
                let start = start_bound.create_key();
                let mut key: Vec<u8> = Vec::default();
                let mut value = Vec::default();

                for ti in table_iterators.iter_mut() {
                    ti.seek(&start);
                    if ti.valid() && ti.current(&mut key, &mut value) {
                        let key = K::parse_key(&key);
                        if key == *start_bound {
                            // We need to exclude the first match
                            ti.advance();
                        }
                    }
                }
            }
            Bound::Unbounded => {
                table_iterators[0].seek_to_first();
            }
        };
        Ok(Range {
            c0_range: self.c0.range(range.clone()),
            range,
            exhausted: std::iter::repeat(false)
                .take(table_iterators.len())
                .collect(),
            table_iterators,
            serialization: self.serialization.clone(),
            phantom: std::marker::PhantomData,
        })
    }

    /// Merges two disk tables.
    /// Newer entries overwrite older ones and tombstones for deleted entries are preserved.
    fn merge_disk_tables(&self, older: &Table, newer: &Table, file: &File) -> Result<()> {
        let mut builder = TableBuilder::new(sstable::Options::default(), file);

        let mut it_older = older.iter();
        let mut it_newer = newer.iter();

        let mut item_older = it_older.next();
        let mut item_newer = it_newer.next();

        while let (Some((k_older, v_older)), Some((k_newer, v_newer))) = (&item_older, &item_newer)
        {
            if k_older < k_newer {
                // Add the value from the older table
                builder.add(k_older, &v_older)?;
                item_older = it_older.next();
            } else if k_older > k_newer {
                // Add the value from the newer table
                builder.add(k_newer, &v_newer)?;
                item_newer = it_newer.next();
            } else {
                // Use the newer values for the same keys
                builder.add(k_newer, &v_newer)?;
                item_older = it_older.next();
                item_newer = it_newer.next();
            }
        }

        builder.finish()?;

        Ok(())
    }

    /// Merges the in-memory map with a disk-based one.
    /// Entries from the in-memory map overwrite the one from the disk.
    /// Since tombstones entries for deleted entries are omitted, this function only works if the resulting table is the only
    /// disk-based one left.
    fn merge_disk_with_c0(
        &self,
        older: &Table,
        newer: &BTreeMap<K, Option<V>>,
        file: &File,
    ) -> Result<()> {
        let mut builder = TableBuilder::new(sstable::Options::default(), file);

        let mut it_older = older.iter();
        let mut it_newer = newer.into_iter();

        let mut item_older = it_older.next();
        let mut item_newer = it_newer.next();

        while let (Some((k_older, v_older)), Some((k_newer, v_newer))) = (&item_older, item_newer) {
            // Create the actual value for the disk key for comparision
            let k_older_parsed = K::parse_key(k_older);
            if &k_older_parsed < k_newer {
                // Add the value from the older table, but do not add a deleted entry
                let parsed: Option<V> = self.serialization.deserialize(v_older)?;
                if parsed.is_some() {
                    builder.add(k_older, &v_older)?;
                }
                item_older = it_older.next();
            } else if &k_older_parsed > k_newer {
                // Add the value from the newer table, but do not add a deleted entry
                if v_newer.is_some() {
                    let raw_key = k_newer.create_key();
                    builder.add(&raw_key, &self.serialization.serialize(&v_newer)?)?;
                }
                item_newer = it_newer.next();
            } else {
                // Use the newer values for the same keys, but check if the newer one is a deletion
                if v_newer.is_some() {
                    let raw_key = k_newer.create_key();
                    builder.add(&raw_key, &self.serialization.serialize(&v_newer)?)?;
                }
                item_older = it_older.next();
                item_newer = it_newer.next();
            }
        }

        builder.finish()?;

        Ok(())
    }

    /// Compact the existing disk tables and the in-memory table to a single disk table.
    pub fn compact(&mut self) -> Result<()> {
        // Newer entries are always appended to the end, to make it easier to pop entries reverse the vector.
        // At the end of the compaction, there is always at most one entry left which is the single most current entry.
        self.disk_tables.reverse();
        // Start from the end of disk tables (now containing the older entries) and merge them pairwise into temporary tables
        let mut older_optional = self.disk_tables.pop();
        let mut newer_optional = self.disk_tables.pop();
        let mut last_outfile = None;
        while let (Some(older), Some(newer)) = (&older_optional, &newer_optional) {
            let out_file = tempfile::NamedTempFile::new()?;
            self.merge_disk_tables(older, newer, out_file.as_file())?;
            // Re-Open as "older" table
            older_optional = Some(Table::new_from_file(
                sstable::Options::default(),
                out_file.path(),
            )?);
            newer_optional = self.disk_tables.pop();
            last_outfile = Some(out_file);
        }

        let table = if self.c0.is_empty() {
            if let Some(last_outfile) = last_outfile {
                // Skip merging C0 and use last table file directly
                if let Some(persistance_file) = &self.persistance_file {
                    last_outfile.persist(persistance_file)?;
                    Table::new_from_file(sstable::Options::default(), persistance_file)?
                } else {
                    Table::new_from_file(sstable::Options::default(), last_outfile.path())?
                }
            } else {
                // C0 is empty and there was no disk table: return new empty disk table
                let out_file = tempfile::NamedTempFile::new()?;
                let builder = TableBuilder::new(sstable::Options::default(), out_file.as_file());
                builder.finish()?;

                if let Some(persistance_file) = &self.persistance_file {
                    out_file.persist(persistance_file)?;
                    Table::new_from_file(sstable::Options::default(), persistance_file)?
                } else {
                    Table::new_from_file(sstable::Options::default(), out_file.path())?
                }
            }
        } else if let Some(newer_optional) = newer_optional {
            // merge C0 and disk-table into new disk table
            let out_file = tempfile::NamedTempFile::new()?;
            self.merge_disk_with_c0(&newer_optional, &self.c0, out_file.as_file())?;

            if let Some(persistance_file) = &self.persistance_file {
                out_file.persist(persistance_file)?;
                Table::new_from_file(sstable::Options::default(), persistance_file)?
            } else {
                Table::new_from_file(sstable::Options::default(), out_file.path())?
            }
        } else {
            // C0 is non-empty but there is no existing disk: write out C0
            let out_file = self.evict_c0()?;
            if let Some(persistance_file) = &self.persistance_file {
                out_file.persist(persistance_file)?;
                Table::new_from_file(sstable::Options::default(), persistance_file)?
            } else {
                Table::new_from_file(sstable::Options::default(), out_file.path())?
            }
        };

        self.c0.clear();
        self.disk_tables = vec![table];

        Ok(())
    }
}

impl<K, V> Default for DiskMap<K, V>
where
    K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    fn default() -> Self {
        DiskMap::new(None, EvictionStrategy::default())
    }
}

pub struct Range<'a, K, V, R>
where
    R: RangeBounds<K>,
{
    range: R,
    c0_range: std::collections::btree_map::Range<'a, K, Option<V>>,
    table_iterators: Vec<TableIterator>,
    exhausted: Vec<bool>,
    serialization: bincode::Config,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<'a, K, V, R> Iterator for Range<'a, K, V, R>
where
    R: RangeBounds<K>,
    for<'de> K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        // TODO: how do we handle deleted values in range queries?

        // Try C0 first
        if let Some((key, value)) = self.c0_range.next() {
            if let Some(value) = value {
                return Some((key.clone(), value.clone()));
            }
        }

        // Iterate over all disk tables
        for i in 0..self.table_iterators.len() {
            let exhausted = &mut self.exhausted[i];
            let table_it = &mut self.table_iterators[i];

            if *exhausted == false && table_it.valid() {
                let mut key = Vec::default();
                let mut value = Vec::default();
                if table_it.current(&mut key, &mut value) {
                    let key = K::parse_key(&key);
                    if self.range.contains(&key) {
                        let value: Option<V> = self
                            .serialization
                            .deserialize(&value)
                            .expect("Could not decode previously written data from disk.");
                        table_it.advance();

                        if let Some(value) = value {
                            return Some((key, value));
                        }
                    } else {
                        *exhausted = true;
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let mut table = DiskMap::new(None, EvictionStrategy::MaximumItems(3));
        table.insert(0, true).unwrap();
        table.insert(1, true).unwrap();
        table.insert(2, true).unwrap();
        table.insert(3, true).unwrap();
        table.insert(4, true).unwrap();
        table.insert(5, true).unwrap();

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
    }
}
