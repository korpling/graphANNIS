use bincode::config::Options;
use itertools::Itertools;
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

const KB: usize = 1 << 10;
pub const MB: usize = KB * KB;
const BLOCK_MAX_SIZE: usize = 4 * KB;

/// Uses a cache for each disk table with 8 MB capacity.
pub const DEFAULT_BLOCK_CACHE_CAPACITY: usize = 8 * MB;

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
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        EvictionStrategy::MaximumItems(10_000)
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
    c1: Option<BtreeIndex<K, Option<V>>>,
    c2: Option<Table>,
    serialization: bincode::config::DefaultOptions,

    c1_btree_config: BtreeConfig,
}

fn custom_options(block_cache_capacity: usize) -> sstable::Options {
    let blocks = (block_cache_capacity / BLOCK_MAX_SIZE).max(1);
    sstable::Options::default().with_cache_capacity(blocks)
}

impl<K, V> DiskMap<K, V>
where
    K: 'static + Clone + KeySerializer + Serialize + DeserializeOwned + Send + Sync + Ord,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync,
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
                let table =
                    Table::new_from_file(custom_options(block_cache_capacity), persisted_file)?;
                disk_table = Some(table);
            }
        }

        Ok(DiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            c2: disk_table,
            serialization: bincode::options(),
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
            c1: None,
            c1_btree_config: c1_config,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        self.c0.insert(key, Some(value));

        self.evict_c0_if_necessary()?;

        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<Cow<V>>> {
        // Check C0 first
        if let Some(entry) = self.c0.get(key) {
            if let Some(value) = entry {
                return Ok(Some(Cow::Borrowed(value)));
            } else {
                // Value was explicitly deleted with a tombstone entry.
                // Do not query C1 and C2.
                return Ok(None);
            }
        }
        // Check C1 (BTree disk index)
        if let Some(c1) = &self.c1 {
            if let Some(entry) = c1.get(key)? {
                if let Some(value) = entry {
                    return Ok(Some(Cow::Owned(value)));
                } else {
                    // Value was explicitly deleted with a tombstone entry.
                    // Do not query C1 and C2.
                    return Ok(None);
                }
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
            self.c0.insert(key.clone(), None);

            self.evict_c0_if_necessary()?;
        }

        Ok(existing)
    }

    pub fn iter(&self) -> Result<ResultIterator<K, V>> {
        if let Some(c1) = &self.c1 {
            if self.c0.is_empty() && self.c2.is_none() {
                // Create an iterator that skips the thombstone entries
                let it = c1
                    .range(..)?
                    .filter_map_ok(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())))
                    .map(|entry| entry.map_err(|e| e.into()));

                return Ok(Box::new(it));
            }
        } else if let Some(c2) = &self.c2 {
            if self.c0.is_empty() && self.c1.as_ref().is_none_or(|c1| c1.is_empty()) {
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
                .filter_map(|(k, v)| v.as_ref().map(|v| Ok((k.clone(), v.clone()))));
            return Ok(Box::new(it));
        }

        // Use the flexible range iterator as default
        Ok(Box::new(self.range(..)))
    }

    /// Returns an iterator over a range of entries.
    pub fn range<'a, R>(&'a self, range: R) -> Box<dyn Iterator<Item = Result<(K, V)>> + 'a>
    where
        R: RangeBounds<K> + Clone,
    {
        // Check if C0, C1 or C2 are the only non-empty maps and return a specialized iterator
        if let Some(c1) = &self.c1 {
            if self.c0.is_empty() && self.c2.is_none() {
                let c1_range = match c1.range(range).map_err(|e| e.into()) {
                    Ok(c1_range) => c1_range,
                    Err(e) => return Box::new(std::iter::once(Err(e))),
                };
                // Return iterator over C1 that skips the tombstone entries
                let it = c1_range
                    .filter_map_ok(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone())))
                    .map(|entry| entry.map_err(|e| e.into()));
                return Box::new(it);
            }
        } else if let Some(c2) = &self.c2 {
            if self.c0.is_empty() && self.c1.as_ref().is_none_or(|c1| c1.is_empty()) {
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
                .filter_map(|(k, v)| v.as_ref().map(|v| Ok((k.clone(), v.clone()))));
            return Box::new(it);
        }
        // Use a combined iterator as default
        match CombinedRange::new(
            range,
            &self.c0,
            self.c1.as_ref(),
            self.c2.as_ref(),
            self.serialization,
        ) {
            Ok(result) => Box::new(result),
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
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
            .truncate(true)
            .open(location)?;
        let mut builder = TableBuilder::new(custom_options(self.block_cache_capacity), out_file);
        for entry in self.iter()? {
            let (key, value) = entry?;
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
                c1.insert(k, v)?;
            }
        }

        debug!("Finished evicting C0");
        Ok(())
    }

    fn evict_c0_if_necessary(&mut self) -> Result<()> {
        let evict_c0 = match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => self.c0.len() >= n,
        };

        if evict_c0 {
            self.compact()?;
        }

        Ok(())
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
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((key, value)) = self.table_iterator.next() {
            let key = match K::parse_key(&key) {
                Ok(key) => key,
                Err(e) => return Some(Err(e.into())),
            };
            let value: Option<V> = match self.serialization.deserialize(&value) {
                Ok(value) => value,
                Err(e) => return Some(Err(e.into())),
            };
            if let Some(value) = value {
                return Some(Ok((key, value)));
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

type ResultIterator<'a, K, V> = Box<dyn Iterator<Item = Result<(K, V)>> + 'a>;

pub struct CombinedRange<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    c0_iterator: Peekable<std::collections::btree_map::Range<'a, K, Option<V>>>,
    c1_iterator: Peekable<ResultIterator<'a, K, Option<V>>>,
    c2_iterator: Peekable<ResultIterator<'a, K, V>>,
}

impl<'a, K, V> CombinedRange<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Serialize + Deserialize<'de> + Send + Sync + Ord,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync,
{
    fn new<R: RangeBounds<K> + Clone>(
        range: R,
        c0: &'a BTreeMap<K, Option<V>>,
        c1: Option<&'a BtreeIndex<K, Option<V>>>,
        c2: Option<&Table>,
        serialization: bincode::config::DefaultOptions,
    ) -> Result<CombinedRange<'a, K, V>> {
        let c1_iterator: Box<dyn Iterator<Item = Result<(K, Option<V>)>>> = if let Some(c1) = c1 {
            let it = c1
                .range(range.clone())?
                .map(|entry| entry.map_err(|e| e.into()));
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        };

        let c2: Box<dyn Iterator<Item = Result<(K, V)>>> = if let Some(c2) = c2 {
            let table_start_bound = match range.start_bound() {
                Bound::Included(end) => Bound::Included(K::create_key(end)),
                Bound::Excluded(end) => Bound::Excluded(K::create_key(end)),
                Bound::Unbounded => Bound::Unbounded,
            };

            let table_end_bound: std::ops::Bound<KeyVec> = match range.end_bound() {
                Bound::Included(end) => Bound::Included(K::create_key(end)),
                Bound::Excluded(end) => Bound::Excluded(K::create_key(end)),
                Bound::Unbounded => Bound::Unbounded,
            };

            let it = SimplifiedRange::new(table_start_bound, table_end_bound, c2, serialization);
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        };

        Ok(CombinedRange {
            c0_iterator: c0.range(range).peekable(),
            c1_iterator: c1_iterator.peekable(),
            c2_iterator: c2.peekable(),
        })
    }
}

impl<K, V> Iterator for CombinedRange<'_, K, V>
where
    K: Ord,
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.c0_iterator.peek().is_some()
            || self.c1_iterator.peek().is_some()
            || self.c2_iterator.peek().is_some()
        {
            // Get keys from all iterators and determine which is the smallest one
            let c0 = self.c0_iterator.peek().map(|(k, _v)| Some(*k));
            let c1 = self.c1_iterator.peek().map(|entry| match entry {
                Ok((k, _v)) => Some(k),
                Err(_) => None,
            });
            let c3 = self.c2_iterator.peek().map(|entry| match entry {
                Ok((k, _v)) => Some(k),
                Err(_) => None,
            });

            let min_key = vec![c0, c1, c3].into_iter().flatten().min();
            if let Some(min_key) = min_key {
                let c0_is_min = c0 == Some(min_key);
                let c1_is_min = c1 == Some(min_key);
                let c3_is_min = c3 == Some(min_key);

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
                        return Some(Ok((k.clone(), v.clone())));
                    } else {
                        // Value was explicitly deleted, do not check the other maps
                        continue;
                    }
                } else if let Some(entry) = c1 {
                    match entry {
                        Ok((k, v)) => {
                            if let Some(v) = v {
                                return Some(Ok((k, v)));
                            } else {
                                // Value was explicitly deleted, do not check the other maps
                                continue;
                            }
                        }
                        Err(e) => {
                            return Some(Err(e));
                        }
                    };
                } else if let Some(entry) = c3 {
                    match entry {
                        Ok((k, v)) => {
                            return Some(Ok((k, v)));
                        }
                        Err(e) => {
                            return Some(Err(e));
                        }
                    }
                }
            }
        }
        None
    }
}

impl<K, V> FusedIterator for CombinedRange<'_, K, V>
where
    K: 'static + Ord + Clone + KeySerializer + Serialize + DeserializeOwned + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
}

impl<K, V> Default for DiskMap<K, V>
where
    K: 'static + Ord + Clone + KeySerializer + Serialize + DeserializeOwned + Send + Sync,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync,
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
            Bound::Included(ref end) => item <= end.as_ref(),
            Bound::Excluded(ref end) => item < end.as_ref(),
            Bound::Unbounded => true,
        })
    }
}

impl<K, V> Iterator for SimplifiedRange<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.exhausted && self.table_it.valid() {
            if self
                .table_it
                .current(&mut self.current_key, &mut self.current_value)
            {
                if self.range_contains(&self.current_key) {
                    let value: Option<V> = match self.serialization.deserialize(&self.current_value)
                    {
                        Ok(value) => value,
                        Err(e) => return Some(Err(e.into())),
                    };

                    self.table_it.advance();

                    if let Some(value) = value {
                        let key = match K::parse_key(&self.current_key) {
                            Ok(key) => key,
                            Err(e) => return Some(Err(e.into())),
                        };
                        return Some(Ok((key, value)));
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
