use crate::annis::errors::*;
use crate::annis::util::memory_estimation;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Serialize};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};

use std::collections::BTreeMap;
use std::fs::File;
use std::iter::Peekable;
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

enum TableEntry {
    Temporary {
        table: Table,
        tmp_file: tempfile::NamedTempFile,
    },
    Persistant {
        table: Table,
    },
}

pub enum EvictionStrategy {
    #[allow(dead_code)]
    MaximumItems(usize),
    MaximumBytes(usize),
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        EvictionStrategy::MaximumBytes(16 * 1024 * 1024)
    }
}

pub struct DiskMap<K, V>
where
    K: 'static + KeySerializer + Send,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send,
{
    persistance_file: Option<PathBuf>,
    eviction_strategy: EvictionStrategy,
    c0: BTreeMap<K, Option<V>>,
    disk_tables: Vec<TableEntry>,
    serialization: bincode::Config,
    table_opts: sstable::Options,

    mem_ops: MallocSizeOfOps,
    est_sum_memory: usize,
}

impl<K, V> DiskMap<K, V>
where
    K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send + MallocSizeOf,
    for<'de> V: 'static
        + Clone
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Serialize
        + Deserialize<'de>
        + Send
        + MallocSizeOf,
{
    pub fn new(
        persistance_file: Option<&Path>,
        eviction_strategy: EvictionStrategy,
    ) -> Result<DiskMap<K, V>> {
        let mut serialization = bincode::config();
        serialization.big_endian();

        let table_opts = sstable::Options::default();

        let mut disk_tables = Vec::default();

        if let Some(persistance_file) = persistance_file {
            // Use existing file as read-only table which contains the whole map
            let table = Table::new_from_file(table_opts.clone(), persistance_file)?;
            disk_tables.push(TableEntry::Persistant { table });
        }

        let mem_ops = MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

        Ok(DiskMap {
            eviction_strategy,
            persistance_file: persistance_file.map(|p| p.to_owned()),
            c0: BTreeMap::default(),
            disk_tables: Vec::default(),
            serialization: serialization,
            table_opts,

            mem_ops,
            est_sum_memory: 0,
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        let existing = self.get(&key)?;

        if let Some(existing) = &existing {
            self.est_sum_memory -= std::mem::size_of::<(K, V)>()
                + key.size_of(&mut self.mem_ops)
                + existing.size_of(&mut self.mem_ops)
        }
        self.est_sum_memory += std::mem::size_of::<(K, V)>()
            + key.size_of(&mut self.mem_ops)
            + value.size_of(&mut self.mem_ops);

        self.c0.insert(key, Some(value));
        self.check_eviction_necessary(true)?;
        Ok(existing)
    }

    fn check_eviction_necessary(&mut self, write_deleted: bool) -> Result<()> {
        match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => {
                if self.c0.len() > n {
                    self.evict_c0(write_deleted)?;
                }
            }
            EvictionStrategy::MaximumBytes(b) => {
                if self.est_sum_memory > b {
                    self.evict_c0(write_deleted)?;
                }
            }
        }
        Ok(())
    }

    fn evict_c0(&mut self, write_deleted: bool) -> Result<()> {
        let out_file = tempfile::NamedTempFile::new()?;
        {
            let mut builder = TableBuilder::new(self.table_opts.clone(), out_file.as_file());

            for (key, value) in self.c0.iter() {
                let key = key.create_key();
                if write_deleted || value.is_some() {
                    builder.add(&key, &self.serialization.serialize(value)?)?;
                }
            }
            builder.finish()?;
        }

        self.est_sum_memory = 0;

        let table = Table::new_from_file(self.table_opts.clone(), out_file.path())?;

        self.disk_tables.push(TableEntry::Temporary {
            table,
            tmp_file: out_file,
        });

        self.c0.clear();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        let existing = self.get(key)?;
        if existing.is_some() {
            self.est_sum_memory -= existing.size_of(&mut self.mem_ops);

            // Add tombstone entry
            let empty_value = None;
            self.est_sum_memory += empty_value.size_of(&mut self.mem_ops);
            self.c0.insert(key.clone(), empty_value);
            self.check_eviction_necessary(true)?;
        }
        Ok(existing)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.c0.clear();
        self.disk_tables.clear();
        self.est_sum_memory = 0;
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
        for table_entry in self.disk_tables.iter().rev() {
            match table_entry {
                TableEntry::Temporary { table, .. } | TableEntry::Persistant { table } => {
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
            }
        }

        Ok(None)
    }

    pub fn contains_key(&self, key: &K) -> Result<bool> {
        self.get(key).map(|item| item.is_some())
    }

    pub fn is_empty(&self) -> Result<bool> {
        if self.c0.is_empty() && self.disk_tables.is_empty() {
            return Ok(true);
        }
        let mut it = self.iter()?;
        Ok(it.next().is_none())
    }

    pub fn iter(&self) -> Result<Range<K, V, std::ops::RangeFull>> {
        self.range(..)
    }

    pub fn range<R>(&self, range: R) -> Result<Range<K, V, R>>
    where
        R: RangeBounds<K> + Clone,
    {
        let mut table_iterators: Vec<TableIterator> = self
            .disk_tables
            .iter()
            .rev()
            .map(|entry| match entry {
                TableEntry::Temporary { table, .. } | TableEntry::Persistant { table } => {
                    table.iter()
                }
            })
            .collect();
        let mut exhausted: Vec<bool> = std::iter::repeat(false)
            .take(table_iterators.len())
            .collect();

        match range.start_bound() {
            Bound::Included(start) => {
                let start = start.create_key();
                let mut key = Vec::default();
                let mut value = Vec::default();

                for i in 0..table_iterators.len() {
                    let exhausted = &mut exhausted[i];
                    let ti = &mut table_iterators[i];
                    ti.seek(&start);

                    if ti.valid() && ti.current(&mut key, &mut value) {
                        // Check if the seeked element is actually part of the range
                        let key = K::parse_key(&key);
                        if !range.contains(&key) {
                            *exhausted = true;
                        }
                    } else {
                        // Seeked behind last element
                        *exhausted = true;
                    }
                }
            }
            Bound::Excluded(start_bound) => {
                let start = start_bound.create_key();
                let mut key: Vec<u8> = Vec::default();
                let mut value = Vec::default();

                for i in 0..table_iterators.len() {
                    let exhausted = &mut exhausted[i];
                    let ti = &mut table_iterators[i];

                    ti.seek(&start);
                    if ti.valid() && ti.current(&mut key, &mut value) {
                        let key = K::parse_key(&key);
                        if key == *start_bound {
                            // We need to exclude the first match
                            ti.advance();
                        }
                    }

                    // Check key after advance
                    if ti.valid() && ti.current(&mut key, &mut value) {
                        // Check if the seeked element is actually part of the range
                        let key = K::parse_key(&key);
                        if !range.contains(&key) {
                            *exhausted = true;
                        }
                    } else {
                        // Seeked behind last element
                        *exhausted = true;
                    }
                }
            }
            Bound::Unbounded => {
                for i in 0..table_iterators.len() {
                    let exhausted = &mut exhausted[i];
                    let ti = &mut table_iterators[i];

                    ti.seek_to_first();

                    if !ti.valid() {
                        *exhausted = true;
                    }
                }
            }
        };
        Ok(Range {
            c0_range: self.c0.range(range.clone()).peekable(),
            range,
            exhausted,
            table_iterators,
            serialization: self.serialization.clone(),
            phantom: std::marker::PhantomData,
        })
    }

    /// Merges two disk tables.
    /// Newer entries overwrite older ones from the base table.
    ///
    /// - `write_deleted` - If `true`, tombstones for deleted entries are preserved and written to disk
    fn merge_disk_tables(
        &self,
        older: &TableEntry,
        newer: &TableEntry,
        file: &File,
        write_deleted: bool,
    ) -> Result<()> {
        let mut builder = TableBuilder::new(self.table_opts.clone(), file);

        let mut it_older = match older {
            TableEntry::Temporary { table, .. } | TableEntry::Persistant { table } => table.iter(),
        };
        let mut it_newer = match newer {
            TableEntry::Temporary { table, .. } | TableEntry::Persistant { table } => table.iter(),
        };

        let mut item_older = it_older.next();
        let mut item_newer = it_newer.next();

        while let (Some((k_older, v_older)), Some((k_newer, v_newer))) = (&item_older, &item_newer)
        {
            if k_older < k_newer {
                // Add the value from the older table
                if write_deleted {
                    builder.add(k_older, &v_older)?;
                } else {
                    let parsed: Option<V> = self.serialization.deserialize(&v_older)?;
                    if parsed.is_some() {
                        builder.add(k_older, &v_older)?;
                    }
                }
                item_older = it_older.next();
            } else if k_older > k_newer {
                // Add the value from the newer table
                if write_deleted {
                    builder.add(k_newer, &v_newer)?;
                } else {
                    let parsed: Option<V> = self.serialization.deserialize(&v_newer)?;
                    if parsed.is_some() {
                        builder.add(k_newer, &v_newer)?;
                    }
                }
                item_newer = it_newer.next();
            } else {
                // Use the newer values for the same keys
                if write_deleted {
                    builder.add(k_newer, &v_newer)?;
                } else {
                    let parsed: Option<V> = self.serialization.deserialize(&v_newer)?;
                    if parsed.is_some() {
                        builder.add(k_newer, &v_newer)?;
                    }
                }
                item_older = it_older.next();
                item_newer = it_newer.next();
            }
        }

        builder.finish()?;

        Ok(())
    }

    /// Compact the existing disk tables and the in-memory table to a single disk table.
    /// If the map has a persistance file set, affter calling this function the persistance
    /// file will contain the complete content of the map.
    pub fn compact_and_flush(&mut self) -> Result<()> {
        // Make sure all entries of C0 are written to disk.
        // Ommit all deleted entries if this becomes the only, and therefore complete, disk table
        if !self.c0.is_empty() {
            self.evict_c0(!self.disk_tables.is_empty())?;
        }

        // More recent entries are always appended to the end.
        // To make it easier to pop entries we are reversing the vector once, so calling "pop" will always return
        // the oldest entry.
        // We don't need to reverse again after the compaction, because there will be only at most one entry left.
        self.disk_tables.reverse();

        // Start from the end of disk tables (now containing the older entries) and merge them pairwise into temporary tables
        let mut base_optional = self.disk_tables.pop();
        let mut newer_optional = self.disk_tables.pop();
        while let (Some(base), Some(newer)) = (&base_optional, &newer_optional) {
            let tmp_file = tempfile::NamedTempFile::new()?;

            // When merging the last two tables, prune the deleted entries
            let write_deleted = !self.disk_tables.is_empty();

            self.merge_disk_tables(base, newer, tmp_file.as_file(), write_deleted)?;
            // Re-Open created table as "older" table
            let table = Table::new_from_file(self.table_opts.clone(), tmp_file.path())?;
            base_optional = Some(TableEntry::Temporary { table, tmp_file });
            // Prepare merging with the next younger table from the log
            newer_optional = self.disk_tables.pop();
        }

        // After evicting C0 and merging all tables, a single disk-table should have been created.
        // Check if we need to persist this table to an external location.
        if let Some(persistance_file) = &self.persistance_file {
            if let Some(table_entry) = base_optional {
                match table_entry {
                    TableEntry::Temporary { tmp_file, .. } => {
                        tmp_file.persist(persistance_file)?;
                    }
                    TableEntry::Persistant { .. } => {
                        // Do nothing.
                        // Persistant table entries are only added in the constructor of the map.
                        // If any change happend, this would lead to changes in C0 and the construction of a new
                        // temporary file table when the immutable persistant table and the one created from C0 are merged.
                        // Since the persistance file path can't be changed after construction, we can safely assume
                        // that the content of the map and the one of the persistant file are the same.
                    }
                }
            } else {
                // Create and persist a new empty table
                let file = File::create(&persistance_file)?;
                let builder = TableBuilder::new(self.table_opts.clone(), file);
                builder.finish()?;
            }
            // Re-open this file and register it as the only disk table
            let table = Table::new_from_file(self.table_opts.clone(), persistance_file)?;
            self.disk_tables = vec![TableEntry::Persistant { table }]
        } else {
            if let Some(table_entry) = base_optional {
                self.disk_tables = vec![table_entry];
            } else {
                self.disk_tables = Vec::default();
            }
        }

        self.est_sum_memory = 0;

        Ok(())
    }
}

impl<K, V> Default for DiskMap<K, V>
where
    K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send + MallocSizeOf,
    for<'de> V: 'static
        + Clone
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Serialize
        + Deserialize<'de>
        + Send
        + MallocSizeOf,
{
    fn default() -> Self {
        DiskMap::new(None, EvictionStrategy::default())
            .expect("Temporary disk map creation should not fail.")
    }
}

pub struct Range<'a, K, V, R>
where
    R: RangeBounds<K>,
{
    range: R,
    c0_range: Peekable<std::collections::btree_map::Range<'a, K, Option<V>>>,
    table_iterators: Vec<TableIterator>,
    exhausted: Vec<bool>,
    serialization: bincode::Config,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<'a, K, V, R> Range<'a, K, V, R>
where
    R: RangeBounds<K>,
    for<'de> K: 'static + Clone + Eq + PartialEq + PartialOrd + Ord + KeySerializer + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    fn advance_all(&mut self, after_key: &K) {
        // Skip all smaller or equal keys in C0
        while let Some(c0_item) = self.c0_range.peek() {
            let key: &K = c0_item.0;
            if key <= after_key {
                self.c0_range.next();
            } else {
                break;
            }
        }

        // Skip all smaller or equal keys in all disk tables
        for i in 0..self.table_iterators.len() {
            let exhausted = &mut self.exhausted[i];
            let table_it = &mut self.table_iterators[i];

            if *exhausted == false && table_it.valid() {
                let mut key = Vec::default();
                let mut value = Vec::default();
                if table_it.current(&mut key, &mut value) {
                    let key = K::parse_key(&key);
                    if !self.range.contains(&key) {
                        *exhausted = true;
                        break;
                    } else if &key <= after_key {
                        table_it.advance();
                    }
                }
            }
        }
    }
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
        loop {
            // Find the smallest key in all tables.
            let mut smallest_key: Option<(K, Option<V>)> = None;

            // Try C0 first
            if let Some(c0_item) = self.c0_range.peek() {
                let key: &K = c0_item.0;
                let value: &Option<V> = c0_item.1;
                smallest_key = Some((key.clone(), value.clone()));
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
                            smallest_key = Some((key, value));
                        } else {
                            *exhausted = true;
                        }
                    }
                }
            }

            if let Some(smallest_key) = smallest_key {
                // Set all iterators to the next element
                self.advance_all(&smallest_key.0);
                // Return any non-deleted entry
                if let Some(value) = smallest_key.1 {
                    return Some((smallest_key.0, value));
                }
            } else {
                // All iterators are exhausted
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        let mut table = DiskMap::new(None, EvictionStrategy::MaximumItems(3)).unwrap();
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
