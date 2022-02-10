use super::memory_estimation;
use bincode::config::Options;
use itertools::Itertools;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};
use transient_btree_index::BtreeIndex;

use crate::serializer::KeyVec;
use crate::{errors::Result, serializer::KeySerializer};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::iter::{FusedIterator, Peekable};
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};
use std::path::Path;

/// Limits the number of sorted string tables the data might be fragmented into before compacting it into one large table.
/// Since each table can use a certain amount of RAM for the block cache, limit the number of tables to limit RAM usage.
pub const DEFAULT_MAX_NUMBER_OF_TABLES: usize = 32;

const KB: usize = 1 << 10;
const MB: usize = KB * KB;
const BLOCK_MAX_SIZE: usize = 4 * KB;

/// Uses a cache for each disk table with 1 MB capacity.
pub const DEFAULT_BLOCK_CACHE_CAPACITY: usize = MB;

#[derive(Serialize, Deserialize)]
struct Entry<K, V>
where
    K: Ord,
{
    key: K,
    value: V,
}

pub enum EvictionStrategy {
    MaximumItems(usize),
    MaximumBytes(usize),
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        EvictionStrategy::MaximumBytes(32 * MB)
    }
}

pub struct DiskMap<K, V>
where
    K: 'static + KeySerializer + Serialize + DeserializeOwned + Clone + Send + Sync + Ord,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Clone + Send + Sync,
{
    eviction_strategy: EvictionStrategy,
    block_cache_capacity: usize,
    c0: BTreeMap<K, Option<V>>,
    c1: Option<BtreeIndex<K, V>>,
    disk_table: Option<Table>,
    serialization: bincode::config::DefaultOptions,

    est_sum_memory: usize,
}

impl<K, V> DiskMap<K, V>
where
    K: 'static
        + Clone
        + KeySerializer
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + MallocSizeOf
        + Ord,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync + MallocSizeOf,
{
    pub fn new(
        persisted_file: Option<&Path>,
        eviction_strategy: EvictionStrategy,
        block_cache_capacity: usize,
    ) -> Result<DiskMap<K, V>> {
        let mut disk_table = None;

        if let Some(persisted_file) = persisted_file {
            if persisted_file.is_file() {
                // Use existing file as read-only table which contains the whole map
                let table = Table::new_from_file(sstable::Options::default(), persisted_file)?;
                disk_table = Some(table);
            }
        }

        Ok(DiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_table,
            serialization: bincode::options(),
            est_sum_memory: 0,
            c1: None,
        })
    }

    pub fn new_temporary(
        eviction_strategy: EvictionStrategy,
        block_cache_capacity: usize,
    ) -> DiskMap<K, V> {
        DiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_table: None,
            serialization: bincode::options(),
            est_sum_memory: 0,
            c1: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);
        let key_size = key.size_of(&mut mem_ops);

        // Add memory size for inserted element
        if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
            self.est_sum_memory +=
                std::mem::size_of::<(Vec<u8>, V)>() + key_size + value.size_of(&mut mem_ops);
        }

        let existing_c0_entry = self.c0.insert(key, Some(value));
        if let Some(existing) = &existing_c0_entry {
            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                // Subtract the memory size for the item that was removed
                self.est_sum_memory -=
                    std::mem::size_of::<(Vec<u8>, V)>() + key_size + existing.size_of(&mut mem_ops);
            }
        }

        self.evict_c0_if_necessary()?;

        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<Cow<V>>> {
        // Check C0 first
        if let Some(entry) = self.c0.get(key) {
            if let Some(value) = entry {
                return Ok(Some(Cow::Borrowed(value)));
            } else {
                // Value was explicitly deleted with a tombstone entry
                //  do not query the disk tables
                return Ok(None);
            }
        }
        // Check the disk table
        let key = K::create_key(key);
        if let Some(table) = &self.disk_table {
            if let Some(value) = table.get(&key)? {
                let value = self.serialization.deserialize(&value)?;
                return Ok(Some(Cow::Owned(value)));
            }
        }

        Ok(None)
    }

    pub fn contains_key(&self, key: &K) -> Result<bool> {
        // Check C0 first
        if let Some(value) = self.c0.get(key) {
            if value.is_some() {
                return Ok(true);
            } else {
                // Value was explicitly deleted, do not query the disk tables
                return Ok(false);
            }
        }

        // Use a iterator on the single disk to check if there is an entry with this, without getting the value.
        // Since we don't serialize tombstone entries when compacting or writing the disk table to an output file,
        // when we are checking the key, we can safely assume the value is Some() and not None.
        if let Some(disk_table) = &self.disk_table {
            let mut table_it = disk_table.iter();
            let key = K::create_key(key);
            table_it.seek(&key);
            if let Some(it_key) = table_it.current_key() {
                if it_key == key.as_ref() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        let existing = self.get(key)?.map(|existing| existing.into_owned());
        if existing.is_some() {
            // Add tombstone entry
            let empty_value = None;
            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                let mut mem_ops =
                    MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

                self.est_sum_memory +=
                    empty_value.size_of(&mut mem_ops) + key.size_of(&mut mem_ops);
            }
            self.c0.insert(key.clone(), empty_value);

            self.evict_c0_if_necessary()?;
        }

        Ok(existing)
    }

    pub fn iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = (K, V)> + 'a>> {
        if let Some(disk_table) = &self.disk_table {
            if self.c0.is_empty() {
                let table_iterator = disk_table.iter();
                let it = SingleTableIterator {
                    table_iterator,
                    serialization: self.serialization,
                    phantom: PhantomData,
                };
                Ok(Box::new(it))
            } else {
                Ok(Box::new(self.range(..)))
            }
        } else {
            // Create an iterator that skips the thombstone entries
            let it = self
                .c0
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())));
            Ok(Box::new(it))
        }
    }

    /// Returns an iterator over a range of entries.
    pub fn range<'a, R>(&'a self, range: R) -> Box<dyn Iterator<Item = (K, V)> + 'a>
    where
        R: RangeBounds<K> + Clone,
    {
        if let Some(disk_table) = &self.disk_table {
            if self.c0.is_empty() {
                let mapped_start_bound: std::ops::Bound<KeyVec> = match range.start_bound() {
                    Bound::Included(end) => Bound::Included(K::create_key(end)),
                    Bound::Excluded(end) => Bound::Excluded(K::create_key(end)),
                    Bound::Unbounded => Bound::Unbounded,
                };

                let mapped_end_bound: std::ops::Bound<KeyVec> = match range.end_bound() {
                    Bound::Included(end) => Bound::Included(K::create_key(end)),
                    Bound::Excluded(end) => Bound::Excluded(K::create_key(end)),
                    Bound::Unbounded => Bound::Unbounded,
                };

                Box::new(SimplifiedRange::new(
                    mapped_start_bound,
                    mapped_end_bound,
                    disk_table,
                    self.serialization,
                ))
            } else {
                Box::new(CombinedRange::new(
                    range,
                    disk_table,
                    &self.c0,
                    self.serialization,
                ))
            }
        } else {
            // Return range iterator over all C0 entries, but skip the tombestone entries
            let it = self
                .c0
                .range(range)
                .filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())));
            Box::new(it)
        }
    }

    pub fn is_empty(&self) -> Result<bool> {
        if self.c0.is_empty() && self.disk_table.is_none() {
            return Ok(true);
        }
        let mut it = self.iter()?;
        Ok(it.next().is_none())
    }

    pub fn clear(&mut self) {
        self.c0.clear();
        self.disk_table = None;
        self.est_sum_memory = 0;
    }

    pub fn write_to(&self, location: &Path) -> Result<()> {
        // Make sure the parent directory exist
        if let Some(parent) = location.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Open file as writable
        let out_file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&location)?;
        let mut builder = TableBuilder::new(self.custom_options(), out_file);
        for (key, value) in self.iter()? {
            let key = key.create_key();
            builder.add(&key, &self.serialization.serialize(&value)?)?;
        }
        builder.finish()?;

        Ok(())
    }

    /// Compact the existing disk tables and the in-memory table to a single temporary disk table.
    pub fn compact(&mut self) -> Result<()> {
        debug!("Evicting C0 and merging it with existing C1 to a temporary file");
        let out_file = tempfile::tempfile()?;

        let mut builder = TableBuilder::new(self.custom_options(), &out_file);

        if let Some(disk_table) = &self.disk_table {
            let c0_iter = self
                .c0
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
                .map(|(k, v)| -> Result<(Vec<u8>, Vec<u8>)> {
                    let k = K::create_key(k).into_vec();
                    let v = self.serialization.serialize(v)?;
                    Ok((k, v))
                });
            let disk_iter: Box<dyn SSIterator> = Box::new(disk_table.iter());
            for entry in disk_iter
                .map(|e| -> Result<(Vec<u8>, Vec<u8>)> { Ok(e) })
                .merge_by(c0_iter, |x, y| {
                    // We need to return if x <= y.
                    // Errors should always be sorted before other
                    // values to catch them early in the loop
                    if let Ok(x) = x {
                        if let Ok(y) = y {
                            // Compare the bot non-error values
                            x <= y
                        } else {
                            // We know that x is ok, but y is an error, so
                            // y < x is true. This is equivalent to x > y,
                            // so x <= y must be false
                            false
                        }
                    } else {
                        if let Ok(_) = y {
                            // x is an error, but y is not: sort x before y
                            // because x <= y holds
                            true
                        } else {
                            // Both values are errors, treat them as equal in this sort
                            true
                        }
                    }
                })
            {
                let (key, value) = entry?;
                builder.add(&key, &value)?;
            }
        } else {
            for (key, value) in self.c0.iter() {
                if let Some(value) = value {
                    let key = key.create_key();
                    builder.add(&key, &self.serialization.serialize(&value)?)?;
                }
            }
        }

        builder.finish()?;

        self.c0.clear();
        self.est_sum_memory = 0;

        // Load the new file as disk table
        let size = out_file.metadata()?.len();
        let table = Table::new(self.custom_options(), Box::new(out_file), size as usize)?;
        self.disk_table = Some(table);

        debug!("Finished evicting C0");
        Ok(())
    }

    fn evict_c0_if_necessary(&mut self) -> Result<()> {
        let evict_c0 = match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => self.c0.len() >= n,
            EvictionStrategy::MaximumBytes(b) => self.est_sum_memory >= b,
        };

        if evict_c0 {
            self.compact()?;
        }

        Ok(())
    }

    fn custom_options(&self) -> sstable::Options {
        let blocks = (self.block_cache_capacity / BLOCK_MAX_SIZE).max(1);
        sstable::Options::default().with_cache_capacity(blocks)
    }
}

/// Implements an optimized iterator a single disk table.
struct SingleTableIterator<K, V> {
    table_iterator: TableIterator,
    serialization: bincode::config::DefaultOptions,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> Iterator for SingleTableIterator<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        if let Some((key, value)) = self.table_iterator.next() {
            let key = K::parse_key(&key);
            let value: V = self
                .serialization
                .deserialize(&value)
                .expect("Could not decode previously written data from disk.");
            Some((key, value))
        } else {
            None
        }
    }
}

impl<K, V> FusedIterator for SingleTableIterator<K, V>
where
    K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
}

pub struct CombinedRange<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    c0_range_iterator: Peekable<std::collections::btree_map::Range<'a, K, Option<V>>>,
    table_iterator: Peekable<SimplifiedRange<K, V>>,
}

impl<'a, K, V> CombinedRange<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send + Ord,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    fn new<R: RangeBounds<K>>(
        range: R,
        disk_table: &Table,
        c0: &'a BTreeMap<K, Option<V>>,
        serialization: bincode::config::DefaultOptions,
    ) -> CombinedRange<'a, K, V> {
        let table_start_bound = match range.start_bound() {
            Bound::Included(end) => Bound::Included(K::create_key(&end)),
            Bound::Excluded(end) => Bound::Excluded(K::create_key(&end)),
            Bound::Unbounded => Bound::Unbounded,
        };

        let table_end_bound: std::ops::Bound<KeyVec> = match range.end_bound() {
            Bound::Included(end) => Bound::Included(K::create_key(&end)),
            Bound::Excluded(end) => Bound::Excluded(K::create_key(&end)),
            Bound::Unbounded => Bound::Unbounded,
        };

        let table_iterator = SimplifiedRange::new(
            table_start_bound,
            table_end_bound,
            disk_table,
            serialization,
        )
        .peekable();

        CombinedRange {
            c0_range_iterator: c0.range(range).peekable(),
            table_iterator,
        }
    }
}

impl<'a, K, V> Iterator for CombinedRange<'a, K, V>
where
    K: Ord,
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        while self.c0_range_iterator.peek().is_some() || self.table_iterator.peek().is_some() {
            let c0 = self.c0_range_iterator.peek();
            let table = self.table_iterator.peek();

            if let (Some(c0), Some(table)) = (c0, table) {
                // Test which one is smaller and output the smaller one
                // Additional checks are needed when the keys are the same, e.g.
                // because a deletion was marked in C0, but the key still exists in C1
                match c0.0.cmp(&table.0) {
                    std::cmp::Ordering::Less => {
                        if let Some((key, value)) = self.c0_range_iterator.next() {
                            // Only output C0, if it is not explictily deleted
                            if let Some(value) = value {
                                return Some((key.clone(), value.clone()));
                            }
                        }
                    }
                    std::cmp::Ordering::Greater => {
                        if let Some(item) = self.table_iterator.next() {
                            return Some(item);
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        // Advance both iterators, but only output the result from C0
                        self.table_iterator.next();
                        if let Some((key, value)) = self.c0_range_iterator.next() {
                            // Only output C0, if it is not explictily deleted
                            if let Some(value) = value {
                                return Some((key.clone(), value.clone()));
                            }
                        }
                    }
                }
            } else if let Some((key, value)) = self.c0_range_iterator.next() {
                // Only output C0, if it is not explictily deleted
                if let Some(value) = value {
                    return Some((key.clone(), value.clone()));
                }
            } else if let Some(item) = self.table_iterator.next() {
                return Some(item);
            }
        }
        None
    }
}

impl<'a, K, V> FusedIterator for CombinedRange<'a, K, V>
where
    K: 'static + Ord + Clone + KeySerializer + Serialize + DeserializeOwned + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
}

impl<K, V> Default for DiskMap<K, V>
where
    K: 'static
        + Ord
        + Clone
        + KeySerializer
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + MallocSizeOf,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync + MallocSizeOf,
{
    fn default() -> Self {
        DiskMap::new_temporary(EvictionStrategy::default(), DEFAULT_BLOCK_CACHE_CAPACITY)
    }
}

/// An iterator implementation for the case that there is only a single disk-table and no C0
struct SimplifiedRange<K, V> {
    range_start: Bound<KeyVec>,
    range_end: Bound<KeyVec>,
    table_it: TableIterator,
    exhausted: bool,
    serialization: bincode::config::DefaultOptions,

    current_key: Vec<u8>,
    current_value: Vec<u8>,

    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> SimplifiedRange<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    fn new(
        range_start: Bound<KeyVec>,
        range_end: Bound<KeyVec>,
        disk_table: &Table,
        serialization: bincode::config::DefaultOptions,
    ) -> SimplifiedRange<K, V> {
        let mut table_it = disk_table.iter();
        let mut exhausted = false;

        // Initialize the table iterators
        match &range_start {
            Bound::Included(start) => {
                let start: &[u8] = start;
                let mut key = Vec::default();
                let mut value = Vec::default();

                table_it.seek(start);

                if table_it.valid() && table_it.current(&mut key, &mut value) {
                    let key: &[u8] = &key;
                    // Check if the seeked element is actually part of the range
                    let start_included = match &range_start {
                        Bound::Included(start) => {
                            let start: &[u8] = start;
                            key >= start
                        }
                        Bound::Excluded(start) => {
                            let start: &[u8] = start;
                            key > start
                        }
                        Bound::Unbounded => true,
                    };
                    let end_included = match &range_end {
                        Bound::Included(end) => {
                            let end: &[u8] = end;
                            key <= end
                        }
                        Bound::Excluded(end) => {
                            let end: &[u8] = end;
                            key < end
                        }
                        Bound::Unbounded => true,
                    };
                    if !start_included || !end_included {
                        exhausted = true;
                    }
                } else {
                    // Seeked behind last element
                    exhausted = true;
                }
            }
            Bound::Excluded(start_bound) => {
                let start_bound: &[u8] = start_bound;

                let mut key: Vec<u8> = Vec::default();
                let mut value = Vec::default();

                table_it.seek(start_bound);
                if table_it.valid() && table_it.current(&mut key, &mut value) {
                    let key: &[u8] = &key;
                    if key == start_bound {
                        // We need to exclude the first match
                        table_it.advance();
                    }
                }

                // Check key after advance
                if table_it.valid() && table_it.current(&mut key, &mut value) {
                    let key: &[u8] = &key;

                    // Check if the seeked element is actually part of the range
                    let start_included = match &range_start {
                        Bound::Included(start) => {
                            let start: &[u8] = start;
                            key >= start
                        }
                        Bound::Excluded(start) => {
                            let start: &[u8] = start;
                            key > start
                        }
                        Bound::Unbounded => true,
                    };
                    let end_included = match &range_end {
                        Bound::Included(end) => {
                            let end: &[u8] = end;
                            key <= end
                        }
                        Bound::Excluded(end) => {
                            let end: &[u8] = end;
                            key < end
                        }
                        Bound::Unbounded => true,
                    };
                    if !start_included || !end_included {
                        exhausted = true;
                    }
                } else {
                    // Seeked behind last element
                    exhausted = true;
                }
            }
            Bound::Unbounded => {
                table_it.seek_to_first();

                if !table_it.valid() {
                    exhausted = true;
                }
            }
        };

        SimplifiedRange {
            range_start,
            range_end,
            exhausted,
            table_it,
            serialization,
            current_key: Vec::new(),
            current_value: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    fn range_contains(&self, item: &[u8]) -> bool {
        (match &self.range_start {
            Bound::Included(ref start) => start.as_slice() <= item,
            Bound::Excluded(ref start) => start.as_slice() < item,
            Bound::Unbounded => true,
        }) && (match &self.range_end {
            Bound::Included(ref end) => item <= end,
            Bound::Excluded(ref end) => item < end,
            Bound::Unbounded => true,
        })
    }
}

impl<K, V> Iterator for SimplifiedRange<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        while !self.exhausted && self.table_it.valid() {
            if self
                .table_it
                .current(&mut self.current_key, &mut self.current_value)
            {
                if self.range_contains(&self.current_key) {
                    let value: V = self
                        .serialization
                        .deserialize(&self.current_value)
                        .expect("Could not decode previously written data from disk.");

                    self.table_it.advance();

                    return Some((K::parse_key(&self.current_key), value));
                } else {
                    self.exhausted = true;
                }
            }
        }
        None
    }
}

impl<K, V> FusedIterator for SimplifiedRange<K, V>
where
    K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
}

#[cfg(test)]
mod tests;
