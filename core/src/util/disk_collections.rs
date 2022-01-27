use super::memory_estimation;
use bincode::config::Options;
use itertools::Itertools;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Serialize};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};

use crate::serializer::KeyVec;
use crate::{errors::Result, serializer::KeySerializer};
use std::collections::BTreeMap;
use std::iter::{FusedIterator, Peekable};
use std::ops::{Bound, RangeBounds};
use std::path::Path;

const DEFAULT_MSG : &str = "Accessing the disk-database failed. This is a non-recoverable error since it means something serious is wrong with the disk or file system.";
const MAX_TRIES: usize = 5;
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

pub struct SingleDiskMap<K, V>
where
    K: 'static + KeySerializer + Send + Sync,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send + Sync,
{
    eviction_strategy: EvictionStrategy,
    block_cache_capacity: usize,
    c0: BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    disk_table: Option<Table>,
    serialization: bincode::config::DefaultOptions,

    est_sum_memory: usize,

    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> SingleDiskMap<K, V>
where
    K: 'static + Clone + KeySerializer + Send + Sync + MallocSizeOf,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync + MallocSizeOf,
{
    pub fn new(
        persisted_file: Option<&Path>,
        eviction_strategy: EvictionStrategy,
        block_cache_capacity: usize,
    ) -> Result<SingleDiskMap<K, V>> {
        let mut disk_table = None;

        if let Some(persisted_file) = persisted_file {
            if persisted_file.is_file() {
                // Use existing file as read-only table which contains the whole map
                let table = Table::new_from_file(sstable::Options::default(), persisted_file)?;
                disk_table = Some(table);
            }
        }

        Ok(SingleDiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_table,
            serialization: bincode::options(),
            est_sum_memory: 0,
            phantom: std::marker::PhantomData,
        })
    }

    pub fn new_temporary(
        eviction_strategy: EvictionStrategy,
        block_cache_capacity: usize,
    ) -> SingleDiskMap<K, V> {
        SingleDiskMap {
            eviction_strategy,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_table: None,
            serialization: bincode::options(),
            est_sum_memory: 0,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        let binary_key = K::create_key(&key);

        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);
        let binary_key_size = binary_key.size_of(&mut mem_ops);

        // Add memory size for inserted element
        if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
            self.est_sum_memory +=
                std::mem::size_of::<(Vec<u8>, V)>() + binary_key_size + value.size_of(&mut mem_ops);
        }

        let value = self.serialization.serialize(&value)?;
        let existing_c0_entry = self.c0.insert(binary_key.into_vec(), Some(value));
        if let Some(existing) = &existing_c0_entry {
            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                // Subtract the memory size for the item that was removed
                self.est_sum_memory -= std::mem::size_of::<(Vec<u8>, V)>()
                    + binary_key_size
                    + existing.size_of(&mut mem_ops);
            }
        }

        self.evict_c0_if_necessary()?;

        Ok(())
    }

    fn evict_c0_if_necessary(&mut self) -> Result<()> {
        let evict_c0 = match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => self.c0.len() >= n,
            EvictionStrategy::MaximumBytes(b) => self.est_sum_memory >= b,
        };

        if evict_c0 {
            debug!("Evicting C0 and merging it with existing C1 to temporary file");
            let out_file = tempfile::tempfile()?;

            let mut builder = TableBuilder::new(self.custom_options(), &out_file);

            if let Some(disk_table) = &self.disk_table {
                let c0_iter: Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> = Box::new(
                    self.c0
                        .iter()
                        .filter_map(|(k, v)| v.as_ref().map(|v| (k.clone(), v.clone()))),
                );
                let disk_iter: Box<dyn SSIterator> = Box::new(disk_table.iter());
                for (key, value) in disk_iter.merge(c0_iter) {
                    builder.add(&key, &value)?;
                }
            } else {
                for (key, value) in self.c0.iter() {
                    if value.is_some() {
                        let key = key.create_key();
                        builder.add(&key, &self.serialization.serialize(&value)?)?;
                    }
                }
            }

            builder.finish()?;

            self.est_sum_memory = 0;
            let size = out_file.metadata()?.len();
            let table = Table::new(self.custom_options(), Box::new(out_file), size as usize)?;
            self.disk_table = Some(table);

            self.c0.clear();

            debug!("Finished evicting C0");
        }

        Ok(())
    }

    fn custom_options(&self) -> sstable::Options {
        let blocks = (self.block_cache_capacity / BLOCK_MAX_SIZE).max(1);
        sstable::Options::default().with_cache_capacity(blocks)
    }
}

pub struct DiskMap<K, V>
where
    K: 'static + KeySerializer + Send + Sync,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send + Sync,
{
    eviction_strategy: EvictionStrategy,
    max_number_of_tables: Option<usize>,
    block_cache_capacity: usize,
    c0: BTreeMap<KeyVec, Option<V>>,
    /// A vector of on-disk tables holding the evicted data.
    disk_tables: Vec<Table>,
    /// Marks if all items have been inserted in sorted order and if there has not been any delete operation yet.
    insertion_was_sorted: bool,
    /// True if the current state is not different from when it was loaded from the a single disk-based table.
    /// This is important, since e.g. the serialized table will never contain tombstone entries.
    unchanged_from_disk: bool,
    last_inserted_key: Option<KeyVec>,

    serialization: bincode::config::DefaultOptions,

    est_sum_memory: usize,

    phantom: std::marker::PhantomData<K>,
}

impl<K, V> DiskMap<K, V>
where
    K: 'static + Clone + KeySerializer + Send + Sync + MallocSizeOf,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync + MallocSizeOf,
{
    pub fn new(
        persisted_file: Option<&Path>,
        eviction_strategy: EvictionStrategy,
        max_number_of_tables: Option<usize>,
        block_cache_capacity: usize,
    ) -> Result<DiskMap<K, V>> {
        let mut disk_tables = Vec::default();

        if let Some(persisted_file) = persisted_file {
            if persisted_file.is_file() {
                // Use existing file as read-only table which contains the whole map
                let table = Table::new_from_file(sstable::Options::default(), persisted_file)?;
                disk_tables.push(table);
            }
        }

        Ok(DiskMap {
            eviction_strategy,
            max_number_of_tables,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_tables,
            insertion_was_sorted: true,
            unchanged_from_disk: persisted_file.is_some(),
            last_inserted_key: None,

            serialization: bincode::options(),
            phantom: std::marker::PhantomData,
            est_sum_memory: 0,
        })
    }

    pub fn new_temporary(
        eviction_strategy: EvictionStrategy,
        max_number_of_tables: Option<usize>,
        block_cache_capacity: usize,
    ) -> DiskMap<K, V> {
        DiskMap {
            eviction_strategy,
            max_number_of_tables,
            block_cache_capacity,
            c0: BTreeMap::default(),
            disk_tables: Vec::default(),
            insertion_was_sorted: true,
            unchanged_from_disk: false,
            last_inserted_key: None,

            serialization: bincode::options(),
            phantom: std::marker::PhantomData,
            est_sum_memory: 0,
        }
    }

    fn custom_options(&self) -> sstable::Options {
        let blocks = (self.block_cache_capacity / BLOCK_MAX_SIZE).max(1);
        sstable::Options::default().with_cache_capacity(blocks)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        self.unchanged_from_disk = false;

        let binary_key = K::create_key(&key);

        let mut mem_ops =
            MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);
        let binary_key_size = binary_key.size_of(&mut mem_ops);

        // Add memory size for inserted element
        if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
            self.est_sum_memory +=
                std::mem::size_of::<(Vec<u8>, V)>() + binary_key_size + value.size_of(&mut mem_ops);
        }

        // Check if insertion is still sorted
        if self.insertion_was_sorted {
            if let Some(last_key) = &self.last_inserted_key {
                let last_key: &[u8] = last_key;
                let binary_key: &[u8] = &binary_key;
                self.insertion_was_sorted = last_key < binary_key;
            }
            self.last_inserted_key = Some(binary_key.clone());
        }

        let existing_c0_entry = self.c0.insert(binary_key, Some(value));
        if let Some(existing) = &existing_c0_entry {
            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                // Subtract the memory size for the item that was removed
                self.est_sum_memory -= std::mem::size_of::<(Vec<u8>, V)>()
                    + binary_key_size
                    + existing.size_of(&mut mem_ops);
            }
        }

        self.check_eviction_necessary(true)?;

        Ok(())
    }

    fn check_eviction_necessary(&mut self, write_deleted: bool) -> Result<()> {
        match self.eviction_strategy {
            EvictionStrategy::MaximumItems(n) => {
                if self.c0.len() >= n {
                    self.evict_c0(write_deleted)?;
                }
            }
            EvictionStrategy::MaximumBytes(b) => {
                if self.est_sum_memory >= b {
                    self.evict_c0(write_deleted)?;
                }
            }
        }
        Ok(())
    }

    fn evict_c0(&mut self, write_deleted: bool) -> Result<()> {
        let num_of_tables = if self.c0.is_empty() {
            self.disk_tables.len()
        } else {
            self.disk_tables.len() + 1
        };

        let needs_compacting = self
            .max_number_of_tables
            .map(|max_number_of_tables| num_of_tables > max_number_of_tables)
            .unwrap_or(false);
        if needs_compacting {
            debug!("Compacting disk tables");
            // Directly compact the existing tables and the C0,
            // which will also evict the C0 table.
            self.compact()?;
        } else {
            debug!("Evicting DiskMap C0 to temporary file");
            let out_file = tempfile::tempfile()?;

            let mut builder = TableBuilder::new(self.custom_options(), &out_file);

            for (key, value) in self.c0.iter() {
                let key = key.create_key();
                if write_deleted || value.is_some() {
                    builder.add(&key, &self.serialization.serialize(value)?)?;
                }
            }
            builder.finish()?;

            self.est_sum_memory = 0;
            let size = out_file.metadata()?.len();
            let table = Table::new(self.custom_options(), Box::new(out_file), size as usize)?;
            self.disk_tables.push(table);

            self.c0.clear();
        }

        debug!("Finished evicting DiskMap C0");
        Ok(())
    }

    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        let key = K::create_key(key);

        let existing = self.get_raw(&key)?;
        if existing.is_some() {
            let mut mem_ops =
                MallocSizeOfOps::new(memory_estimation::platform::usable_size, None, None);

            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                self.est_sum_memory -= existing.size_of(&mut mem_ops);
            }

            // Add tombstone entry
            let empty_value = None;
            if let EvictionStrategy::MaximumBytes(_) = self.eviction_strategy {
                self.est_sum_memory += empty_value.size_of(&mut mem_ops);
            }
            self.c0.insert(key, empty_value);

            self.insertion_was_sorted = false;
            self.unchanged_from_disk = false;

            self.check_eviction_necessary(true)?;
        }
        Ok(existing)
    }

    pub fn clear(&mut self) {
        self.c0.clear();
        self.disk_tables.clear();
        self.est_sum_memory = 0;
        self.insertion_was_sorted = true;
        self.unchanged_from_disk = false;
        self.last_inserted_key = None;
    }

    pub fn try_get(&self, key: &K) -> Result<Option<V>> {
        let key = K::create_key(key);
        self.get_raw(&key)
    }

    /// Returns an optional value for the given key.
    ///
    /// # Panics
    ///
    /// The will try to query the disk-based map several times
    /// If a maximum number of tries is reached and all attempts failed, this will panic.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut last_err = None;
        for _ in 0..MAX_TRIES {
            match self.try_get(key) {
                Ok(result) => return result,
                Err(e) => last_err = Some(e),
            }
            // If this is an intermediate error, wait some time before trying again
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("{}\nCause:\n{:?}", DEFAULT_MSG, last_err.unwrap())
    }

    fn get_raw(&self, key: &[u8]) -> Result<Option<V>> {
        // Check C0 first
        if let Some(value) = self.c0.get(key.as_ref()) {
            if value.is_some() {
                return Ok(value.clone());
            } else {
                // Value was explicitly deleted, do not query the disk tables
                return Ok(None);
            }
        }
        // Iterate over all disk-tables to find the entry
        for table in self.disk_tables.iter().rev() {
            if let Some(value) = table.get(key)? {
                let value: Option<V> = self.serialization.deserialize(&value)?;
                if value.is_some() {
                    return Ok(value);
                } else {
                    // Value was explicitly deleted, do not query the rest of the disk tables
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    pub fn try_contains_key(&self, key: &K) -> Result<bool> {
        let key = K::create_key(key);

        if self.unchanged_from_disk {
            // Use a iterator on the single disk to check if there is an entry with this, without getting the value.
            // Since we don't serialize tombstone entries when compacting or writing the disk table to an output file,
            // when we are checking the key, we can safely assume the value is Some() and not None.
            if self.disk_tables.len() == 1 {
                let mut table_it = self.disk_tables[0].iter();
                table_it.seek(&key);
                if let Some(it_key) = table_it.current_key() {
                    if it_key == key.as_ref() {
                        return Ok(true);
                    }
                }
            }
        } else {
            // Check C0 first
            if let Some(value) = self.c0.get(key.as_ref()) {
                if value.is_some() {
                    return Ok(true);
                } else {
                    // Value was explicitly deleted, do not query the disk tables
                    return Ok(false);
                }
            }
            // Iterate over all disk-tables to find the entry
            for table in self.disk_tables.iter().rev() {
                if let Some(value) = table.get(key.as_ref())? {
                    let value: Option<V> = self.serialization.deserialize(&value)?;
                    if value.is_some() {
                        return Ok(true);
                    } else {
                        // Value was explicitly deleted, do not query the rest of the disk tables
                        return Ok(false);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Returns if the given key is contained.
    ///
    /// # Panics
    ///
    /// The will try to query the disk-based map several times
    /// If a maximum number of tries is reached and all attempts failed, this will panic.
    pub fn contains_key(&self, key: &K) -> bool {
        let mut last_err = None;
        for _ in 0..MAX_TRIES {
            match self.try_contains_key(key) {
                Ok(result) => return result,
                Err(e) => last_err = Some(e),
            }
            // If this is an intermediate error, wait some time before trying again
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("{}\nCause:\n{:?}", DEFAULT_MSG, last_err.unwrap())
    }

    pub fn try_is_empty(&self) -> Result<bool> {
        if self.c0.is_empty() && self.disk_tables.is_empty() {
            return Ok(true);
        }
        let mut it = self.try_iter()?;
        Ok(it.next().is_none())
    }

    /// Returns if the map is empty
    ///
    /// # Panics
    ///
    /// The will try to query the disk-based map several times
    /// If a maximum number of tries is reached and all attempts failed, this will panic.
    pub fn is_empty(&self) -> bool {
        let mut last_err = None;
        for _ in 0..MAX_TRIES {
            match self.try_is_empty() {
                Ok(result) => return result,
                Err(e) => last_err = Some(e),
            }
            // If this is an intermediate error, wait some time before trying again
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("{}\nCause:\n{:?}", DEFAULT_MSG, last_err.unwrap())
    }

    pub fn try_iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = (K, V)> + 'a>> {
        if self.unchanged_from_disk && self.disk_tables.len() == 1 {
            // Directly return an iterator over the one single disk table
            let it = SingleDiskTableIteator {
                table_iterator: self.disk_tables[0].iter(),
                serialization: self.serialization,
                phantom: std::marker::PhantomData,
            };
            Ok(Box::new(it))
        } else if self.insertion_was_sorted {
            // Use a less complicated and faster iterator over all items
            let mut remaining_table_iterators = Vec::with_capacity(self.disk_tables.len());
            // The disk tables are sorted by oldest first. Reverse the order to have the oldest ones last, so that
            // calling "pop()" will return older disk tables first.
            for t in self.disk_tables.iter().rev() {
                let it = t.iter();
                remaining_table_iterators.push(it);
            }
            let current_table_iterator = remaining_table_iterators.pop();
            let it = SortedLogTableIterator {
                c0_iterator: self.c0.iter(),
                current_table_iterator,
                remaining_table_iterators,
                serialization: self.serialization,
                phantom: std::marker::PhantomData,
            };
            Ok(Box::new(it))
        } else {
            // Default to an iterator that can handle non-globally sorted tables
            let it = self.range(..);
            Ok(Box::new(it))
        }
    }

    /// Returns an iterator over the all entries.
    ///
    /// # Panics
    ///
    /// The will try to query the disk-based map several times
    /// If a maximum number of tries is reached and all attempts failed, this will panic.
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        let mut last_err = None;
        for _ in 0..MAX_TRIES {
            match self.try_iter() {
                Ok(result) => return result,
                Err(e) => last_err = Some(e),
            }
            // If this is an intermediate error, wait some time before trying again
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("{}\nCause:\n{:?}", DEFAULT_MSG, last_err.unwrap())
    }

    /// Returns an iterator over a range of entries.
    pub fn range<'b, R>(&'b self, range: R) -> Box<dyn Iterator<Item = (K, V)> + 'b>
    where
        R: RangeBounds<K> + Clone,
    {
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

        if self.c0.is_empty() && self.disk_tables.len() == 1 {
            Box::new(SimplifiedRange::new(
                mapped_start_bound,
                mapped_end_bound,
                &self.disk_tables[0],
                self.serialization,
            ))
        } else {
            Box::new(Range::new(
                mapped_start_bound,
                mapped_end_bound,
                self.disk_tables.as_slice(),
                &self.c0,
                self.serialization,
            ))
        }
    }

    /// Compact the existing disk tables and the in-memory table to a single temporary disk table.
    pub fn compact(&mut self) -> Result<()> {
        self.est_sum_memory = 0;

        if self.c0.is_empty() && (self.disk_tables.is_empty() || self.disk_tables.len() == 1) {
            // The table are empty or already compacted, there is nothing to do
            return Ok(());
        }

        // Create single temporary sorted string file by iterating over all entries
        let out_file = tempfile::tempfile()?;
        let mut builder = TableBuilder::new(self.custom_options(), &out_file);
        for (key, value) in self.try_iter()? {
            let key = key.create_key();
            builder.add(&key, &self.serialization.serialize(&Some(value))?)?;
        }
        let size = builder.finish()?;

        // Re-open sorted string table and set it as the only table
        let table = Table::new(self.custom_options(), Box::new(out_file), size)?;
        self.disk_tables = vec![table];
        self.c0.clear();

        self.unchanged_from_disk = true;

        debug!("Finished merging disk-based tables in DiskMap");

        Ok(())
    }

    pub fn number_of_disk_tables(&self) -> usize {
        self.disk_tables.len()
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
        for (key, value) in self.try_iter()? {
            let key = key.create_key();
            builder.add(&key, &self.serialization.serialize(&Some(value))?)?;
        }
        builder.finish()?;

        Ok(())
    }
}

impl<K, V> Default for DiskMap<K, V>
where
    K: 'static + Clone + KeySerializer + Send + Sync + MallocSizeOf,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send + Sync + MallocSizeOf,
{
    fn default() -> Self {
        DiskMap::new(
            None,
            EvictionStrategy::default(),
            Some(DEFAULT_MAX_NUMBER_OF_TABLES),
            DEFAULT_BLOCK_CACHE_CAPACITY,
        )
        .expect("Temporary disk map creation should not fail.")
    }
}

pub struct Range<'a, K, V> {
    range_start: Bound<KeyVec>,
    range_end: Bound<KeyVec>,
    c0_range: Peekable<std::collections::btree_map::Range<'a, KeyVec, Option<V>>>,
    table_iterators: Vec<TableIterator>,
    exhausted: Vec<bool>,
    serialization: bincode::config::DefaultOptions,

    current_key: Vec<u8>,
    current_value: Vec<u8>,

    phantom: std::marker::PhantomData<(K, V)>,
}

impl<'a, K, V> Range<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    fn new(
        range_start: Bound<KeyVec>,
        range_end: Bound<KeyVec>,
        disk_tables: &[Table],
        c0: &'a BTreeMap<KeyVec, Option<V>>,
        serialization: bincode::config::DefaultOptions,
    ) -> Range<'a, K, V> {
        let mut table_iterators: Vec<TableIterator> =
            disk_tables.iter().rev().map(|table| table.iter()).collect();
        let mut exhausted: Vec<bool> = std::iter::repeat(false)
            .take(table_iterators.len())
            .collect();

        // Initialize the table iterators
        match &range_start {
            Bound::Included(start) => {
                let start: &[u8] = start;
                let mut key = Vec::default();
                let mut value = Vec::default();

                for i in 0..table_iterators.len() {
                    let exhausted = &mut exhausted[i];
                    let ti = &mut table_iterators[i];
                    ti.seek(start);

                    if ti.valid() && ti.current(&mut key, &mut value) {
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
                            *exhausted = true;
                        }
                    } else {
                        // Seeked behind last element
                        *exhausted = true;
                    }
                }
            }
            Bound::Excluded(start_bound) => {
                let start_bound: &[u8] = start_bound;

                let mut key: Vec<u8> = Vec::default();
                let mut value = Vec::default();

                for i in 0..table_iterators.len() {
                    let exhausted = &mut exhausted[i];
                    let ti = &mut table_iterators[i];

                    ti.seek(start_bound);
                    if ti.valid() && ti.current(&mut key, &mut value) {
                        let key: &[u8] = &key;
                        if key == start_bound {
                            // We need to exclude the first match
                            ti.advance();
                        }
                    }

                    // Check key after advance
                    if ti.valid() && ti.current(&mut key, &mut value) {
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

        Range {
            c0_range: c0
                .range((range_start.clone(), range_end.clone()))
                .peekable(),
            range_start,
            range_end,
            exhausted,
            table_iterators,
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

    fn advance_all(&mut self, after_key: &[u8]) {
        // Skip all smaller or equal keys in C0
        while let Some(c0_item) = self.c0_range.peek() {
            if c0_item.0.as_slice() <= after_key {
                self.c0_range.next();
            } else {
                break;
            }
        }

        // Skip all smaller or equal keys in all disk tables
        for i in 0..self.table_iterators.len() {
            if !self.exhausted[i]
                && self.table_iterators[i].valid()
                && self.table_iterators[i].current(&mut self.current_key, &mut self.current_value)
            {
                if !self.range_contains(&self.current_key) {
                    self.exhausted[i] = true;
                    break;
                }
                if self.current_key.as_slice() <= after_key {
                    self.table_iterators[i].advance();
                }
            }
        }
    }
}

impl<'a, K, V> Iterator for Range<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        loop {
            // Find the smallest key in all tables.
            let mut smallest_key: Option<(Vec<u8>, Option<V>)> = None;

            // Try C0 first
            if let Some(c0_item) = self.c0_range.peek() {
                let key: &KeyVec = c0_item.0;
                let value: &Option<V> = c0_item.1;
                smallest_key = Some((key.to_vec(), value.clone()));
            }

            // Iterate over all disk tables
            for i in 0..self.table_iterators.len() {
                let table_it = &mut self.table_iterators[i];
                if !self.exhausted[i]
                    && table_it.valid()
                    && table_it.current(&mut self.current_key, &mut self.current_value)
                {
                    if self.range_contains(&self.current_key) {
                        let key_is_smaller = if let Some((smallest_key, _)) = &smallest_key {
                            &self.current_key < smallest_key
                        } else {
                            true
                        };
                        if key_is_smaller {
                            let value: Option<V> = self
                                .serialization
                                .deserialize(&self.current_value)
                                .expect("Could not decode previously written data from disk.");
                            smallest_key = Some((self.current_key.clone(), value));
                        }
                    } else {
                        self.exhausted[i] = true;
                    }
                }
            }

            if let Some(smallest_key) = smallest_key {
                // Set all iterators to the next element
                self.advance_all(&smallest_key.0);
                // Return any non-deleted entry
                if let Some(value) = smallest_key.1 {
                    let key = K::parse_key(&smallest_key.0);
                    return Some((key, value));
                }
            } else {
                // All iterators are exhausted
                return None;
            }
        }
    }
}

/// An iterator implementation for the case that there is only a single disk-table and no C0
pub struct SimplifiedRange<K, V> {
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

/// Implements an optimized iterator a single disk table.
struct SingleDiskTableIteator<K, V> {
    table_iterator: TableIterator,
    serialization: bincode::config::DefaultOptions,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> Iterator for SingleDiskTableIteator<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        if let Some((key, value)) = self.table_iterator.next() {
            let key = K::parse_key(&key);
            let value: Option<V> = self
                .serialization
                .deserialize(&value)
                .expect("Could not decode previously written data from disk.");
            if let Some(value) = value {
                Some((key, value))
            } else {
                panic!("Optimized log table iterator should have been called only if no entry was ever deleted");
            }
        } else {
            None
        }
    }
}

impl<K, V> FusedIterator for SingleDiskTableIteator<K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
}

/// Implements an optimized iterator over C0 and all disk tables.
/// This iterator assumes the table entries have been inserted in sorted
/// order and no delete has occurred.
struct SortedLogTableIterator<'a, K, V> {
    current_table_iterator: Option<TableIterator>,
    remaining_table_iterators: Vec<TableIterator>,
    c0_iterator: std::collections::btree_map::Iter<'a, KeyVec, Option<V>>,
    serialization: bincode::config::DefaultOptions,
    phantom: std::marker::PhantomData<K>,
}

impl<'a, K, V> Iterator for SortedLogTableIterator<'a, K, V>
where
    for<'de> K: 'static + Clone + KeySerializer + Send,
    for<'de> V: 'static + Clone + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        while let Some(t) = &mut self.current_table_iterator {
            if let Some((key, value)) = t.next() {
                let key = K::parse_key(&key);
                let value: Option<V> = self
                    .serialization
                    .deserialize(&value)
                    .expect("Could not decode previously written data from disk.");
                if let Some(value) = value {
                    return Some((key, value));
                } else {
                    panic!("Optimized log table iterator should have been called only if no entry was ever deleted");
                }
            } else {
                self.current_table_iterator = self.remaining_table_iterators.pop();
            }
        }
        // Check C0 (which contains the newest entries)
        if let Some((key, value)) = self.c0_iterator.next() {
            let key = K::parse_key(key);
            if let Some(value) = value {
                return Some((key, value.clone()));
            } else {
                panic!("Optimized log table iterator should have been called only if no entry was ever deleted");
            }
        } else {
        }

        None
    }
}

#[cfg(test)]
mod tests;
