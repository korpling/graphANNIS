use super::memory_estimation;
use bincode::config::Options;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};
use transient_btree_index::{BtreeConfig, BtreeIndex};

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
    c2: Option<Table>,
    serialization: bincode::config::DefaultOptions,

    c1_btree_config: BtreeConfig,

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
        c1_config: BtreeConfig,
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
            c2: disk_table,
            serialization: bincode::options(),
            est_sum_memory: 0,
            c1: None,
            c1_btree_config: c1_config,
        })
    }

    pub fn new_temporary(
        eviction_strategy: EvictionStrategy,
        block_cache_capacity: usize,
        c1_config: BtreeConfig,
    ) -> DiskMap<K, V> {
        DiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            c2: None,
            serialization: bincode::options(),
            est_sum_memory: 0,
            c1: None,
            c1_btree_config: c1_config,
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
        // Check C1 (BTree disk index)
        if let Some(c1) = &self.c1 {
            if let Some(value) = c1.get(key)? {
                return Ok(Some(Cow::Owned(value)));
            }
        }

        // Check the C2 (sstable)
        if let Some(c2) = &self.c2 {
            let key = K::create_key(key);
            if let Some(value) = c2.get(&key)? {
                let value: Option<V> = self.serialization.deserialize(&value)?;
                if let Some(value) = value {
                    return Ok(Some(Cow::Owned(value)));
                } else {
                    // Value was explicitly deleted (and still written to disk)
                    return Ok(None);
                }
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

        // Check C1 (BTree disk index)
        if let Some(c1) = &self.c1 {
            if c1.contains_key(key)? {
                return Ok(true);
            }
        }

        // Use a iterator on the single disk to check if there is an entry with this, without getting the value.
        // Since we don't serialize tombstone entries when compacting or writing the disk table to an output file,
        // when we are checking the key, we can safely assume the value is Some() and not None.
        if let Some(c2) = &self.c2 {
            let mut table_it = c2.iter();
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
        if let Some(c1) = &self.c1 {
            if self.c0.is_empty() && self.c2.is_none() {
                let it = c1.range(..)?.filter_map(|e| e.ok());
                return Ok(Box::new(it));
            }
        } else if let Some(c2) = &self.c2 {
            if self.c0.is_empty() && self.c1.as_ref().map_or(true, |c1| c1.is_empty()) {
                let table_iterator = c2.iter();
                let it = SingleTableIterator {
                    table_iterator,
                    serialization: self.serialization,
                    phantom: PhantomData,
                };
                return Ok(Box::new(it));
            }
        } else {
            // Create an iterator that skips the thombstone entries
            let it = self
                .c0
                .iter()
                .filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())));
            return Ok(Box::new(it));
        }

        // Use the flexible range iterator as default
        Ok(Box::new(self.range(..)))
    }

    /// Returns an iterator over a range of entries.
    pub fn range<'a, R>(&'a self, range: R) -> Box<dyn Iterator<Item = (K, V)> + 'a>
    where
        R: RangeBounds<K> + Clone,
    {
        // Check if C0, C1 or C2 are the only non-empty maps and return a specialized iterator
        if let Some(c1) = &self.c1 {
            if self.c0.is_empty() && self.c2.is_none() {
                if let Ok(c1_range) = c1.range(range.clone()) {
                    // Return iterator over C1
                    // TODO: error handling
                    return Box::new(c1_range.filter_map(|e| e.ok()));
                }
            }
        } else if let Some(c2) = &self.c2 {
            if self.c0.is_empty() && self.c1.as_ref().map_or(true, |c1| c1.is_empty()) {
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

                // Return iterator over C2
                return Box::new(SimplifiedRange::new(
                    mapped_start_bound,
                    mapped_end_bound,
                    c2,
                    self.serialization,
                ));
            }
        } else {
            // Neither C1 nor C2 exist:
            // return range iterator over all C0 entries, but skip the tombestone entries
            let it = self
                .c0
                .range(range)
                .filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())));
            return Box::new(it);
        }
        // Use a combined iterator as default
        Box::new(CombinedRange::new(
            range,
            &self.c0,
            self.c1.as_ref(),
            self.c2.as_ref(),
            self.serialization,
        ))
    }

    pub fn is_empty(&self) -> Result<bool> {
        if self.c0.is_empty() && self.c1.is_none() && self.c2.is_none() {
            return Ok(true);
        }
        let mut it = self.iter()?;
        Ok(it.next().is_none())
    }

    pub fn clear(&mut self) {
        self.c0.clear();
        self.c1 = None;
        self.c2 = None;
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
            let value = Some(value);
            builder.add(&key, &self.serialization.serialize(&value)?)?;
        }
        builder.finish()?;

        Ok(())
    }

    /// Compact the existing disk tables and the in-memory table to a single temporary disk table.
    pub fn compact(&mut self) -> Result<()> {
        debug!("Evicting C0 and merging it with existing C1 to a temporary file");

        if self.c1.is_none() {
            let c1 = BtreeIndex::with_capacity(self.c1_btree_config.clone(), self.c0.len())?;
            self.c1 = Some(c1);
        }

        if let Some(c1) = self.c1.as_mut() {
            let mut c0 = BTreeMap::new();
            std::mem::swap(&mut self.c0, &mut c0);
            for (k, v) in c0.into_iter() {
                if let Some(v) = v {
                    c1.insert(k, v)?;
                }
            }
        }

        self.est_sum_memory = 0;

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
        while let Some((key, value)) = self.table_iterator.next() {
            let key = K::parse_key(&key);
            let value: Option<V> = self
                .serialization
                .deserialize(&value)
                .expect("Could not decode previously written data from disk.");
            if let Some(value) = value {
                return Some((key, value));
            }
        }
        None
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
    c0_iterator: Peekable<std::collections::btree_map::Range<'a, K, Option<V>>>,
    c1_iterator: Peekable<Box<dyn Iterator<Item = (K, V)> + 'a>>,
    c2_iterator: Peekable<Box<dyn Iterator<Item = (K, V)> + 'a>>,
}

impl<'a, K, V> CombinedRange<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Serialize + Deserialize<'de> + Send + Sync + Ord,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync,
{
    fn new<R: RangeBounds<K> + Clone>(
        range: R,
        c0: &'a BTreeMap<K, Option<V>>,
        c1: Option<&'a BtreeIndex<K, V>>,
        c2: Option<&Table>,
        serialization: bincode::config::DefaultOptions,
    ) -> CombinedRange<'a, K, V> {
        let c1_iterator: Box<dyn Iterator<Item = (K, V)>> = if let Some(c1) = c1 {
            if let Ok(it) = c1.range(range.clone()) {
                // TODO: add error handling
                Box::new(it.filter_map(|e| e.ok()))
            } else {
                // TODO: add error handling
                Box::new(std::iter::empty())
            }
        } else {
            Box::new(std::iter::empty())
        };

        let c2: Box<dyn Iterator<Item = (K, V)>> = if let Some(c2) = c2 {
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

            let it = SimplifiedRange::new(table_start_bound, table_end_bound, c2, serialization);
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        };

        CombinedRange {
            c0_iterator: c0.range(range).peekable(),
            c1_iterator: c1_iterator.peekable(),
            c2_iterator: c2.peekable(),
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
        while self.c0_iterator.peek().is_some()
            || self.c1_iterator.peek().is_some()
            || self.c2_iterator.peek().is_some()
        {
            // Get keys from all iterators and determine which is the smallest one
            let c0 = self.c0_iterator.peek().map(|(k, _v)| *k);
            let c1 = self.c1_iterator.peek().map(|(k, _v)| k);
            let c3 = self.c2_iterator.peek().map(|(k, _v)| k);

            let min_key = vec![c0, c1, c3].into_iter().filter_map(|k| k).min();
            if let Some(min_key) = min_key {
                let c0_is_min = c0.map_or(false, |k| k == min_key);
                let c1_is_min = c1.map_or(false, |k| k == min_key);
                let c3_is_min = c3.map_or(false, |k| k == min_key);

                // Advance all iterators with the same (minimal) key
                let c0 = if c0_is_min {
                    self.c0_iterator.next()
                } else {
                    None
                };
                let c1 = if c1_is_min {
                    self.c1_iterator.next()
                } else {
                    None
                };
                let c3 = if c3_is_min {
                    self.c2_iterator.next()
                } else {
                    None
                };

                // Output the value from the most recent map
                if let Some((k, v)) = c0 {
                    if let Some(v) = v {
                        return Some((k.clone(), v.clone()));
                    } else {
                        // Value was explicitly deleted, do not check the other maps
                        continue;
                    }
                } else if let Some((k, v)) = c1 {
                    return Some((k.clone(), v.clone()));
                } else if let Some((k, v)) = c3 {
                    return Some((k.clone(), v.clone()));
                }
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
        DiskMap::new_temporary(
            EvictionStrategy::default(),
            DEFAULT_BLOCK_CACHE_CAPACITY,
            BtreeConfig::default(),
        )
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
                    let value: Option<V> = self
                        .serialization
                        .deserialize(&self.current_value)
                        .expect("Could not decode previously written data from disk.");

                    self.table_it.advance();

                    if let Some(value) = value {
                        return Some((K::parse_key(&self.current_key), value));
                    }
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
