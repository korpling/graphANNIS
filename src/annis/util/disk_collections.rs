use crate::annis::errors::*;
use serde::{Deserialize, Serialize};
use shardio::{ShardReader, ShardWriter};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};

use std::io::Write;
use std::ops::{Bound, RangeBounds};

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

pub struct DiskMapBuilder<K, V>
where
    for<'de> K: 'static
        + Clone
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Serialize
        + Deserialize<'de>
        + Send
        + core::fmt::Debug,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    shard_writer: ShardWriter<Entry<K, V>>,
    serialization: bincode::Config,
    tmp_file: tempfile::NamedTempFile,
}

impl<K, V> DiskMapBuilder<K, V>
where
    for<'de> K: 'static
        + Clone
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Serialize
        + Deserialize<'de>
        + Send
        + core::fmt::Debug,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    pub fn new() -> Result<DiskMapBuilder<K, V>> {
        let tmp_file = tempfile::NamedTempFile::new()?;

        let shard_writer: ShardWriter<Entry<K, V>> =
            ShardWriter::new(&tmp_file.path(), 64, 256, 1 << 16)?;

        let mut serialization = bincode::config();
        serialization.big_endian();

        Ok(DiskMapBuilder {
            tmp_file,
            shard_writer,
            serialization,
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        self.shard_writer.get_sender().send(Entry { key, value })?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<DiskMap<K, V>> {
        // Finish sorting
        self.shard_writer.finish()?;
        // Open sorted shard for reading
        let reader = ShardReader::<Entry<K, V>>::open(self.tmp_file.path())?;
        // Create the indexes by iterating over the sorted entries
        let mut tmp_file = tempfile::NamedTempFile::new()?;
        let mut table_builder = TableBuilder::new(sstable::Options::default(), tmp_file.as_file());
        for entry in reader.iter()? {
            let entry: Entry<K, V> = entry?;

            table_builder.add(
                &self.serialization.serialize(&entry.key)?,
                &self.serialization.serialize(&entry.value)?,
            )?;
        }
        table_builder.finish()?;
        tmp_file.flush()?;

        // Open the created index file as table
        let table = Table::new_from_file(sstable::Options::default(), tmp_file.path())?;
        Ok(DiskMap {
            table,
            serialization: self.serialization,
            phantom: std::marker::PhantomData,
        })
    }
}

pub struct DiskMap<K, V>
where
    for<'de> K: 'static + Serialize + Deserialize<'de> + Send,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send,
{
    table: Table,
    serialization: bincode::Config,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> DiskMap<K, V>
where
    for<'de> K:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let key = self.serialization.serialize(key)?;
        if let Some(value) = self.table.get(&key)? {
            let value = self.serialization.deserialize(&value)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn range<R>(&self, range: R) -> Result<Range<K, V, R>>
    where
        R: RangeBounds<K>,
    {
        let mut table_it = self.table.iter();
        match range.start_bound() {
            Bound::Included(start) => {
                let start = self.serialization.serialize(start)?;
                table_it.seek(&start);
            }
            Bound::Excluded(start_bound) => {
                let start = self.serialization.serialize(start_bound)?;
                table_it.seek(&start);
                let mut key = Vec::default();
                let mut value = Vec::default();

                if table_it.valid() && table_it.current(&mut key, &mut value) {
                    let key: K = self
                        .serialization
                        .deserialize(&key)
                        .expect("Could not decode previously written data from disk.");
                    if key == *start_bound {
                        // We need to exclude the first match
                        table_it.advance();
                    }
                }
            }
            Bound::Unbounded => {
                table_it.seek_to_first();
            }
        };
        Ok(Range {
            range,
            table_it,
            exhausted: false,
            serialization: self.serialization.clone(),
            phantom: std::marker::PhantomData,
        })
    }
}

pub struct Range<K, V, R>
where
    R: RangeBounds<K>,
{
    range: R,
    table_it: TableIterator,
    exhausted: bool,
    serialization: bincode::Config,
    phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V, R> Iterator for Range<K, V, R>
where
    R: RangeBounds<K>,
    for<'de> K:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        if !self.exhausted && self.table_it.valid() {
            let mut key = Vec::default();
            let mut value = Vec::default();

            if self.table_it.current(&mut key, &mut value) {
                let key = self
                    .serialization
                    .deserialize(&key)
                    .expect("Could not decode previously written data from disk.");
                if self.range.contains(&key) {
                    let value = self
                        .serialization
                        .deserialize(&value)
                        .expect("Could not decode previously written data from disk.");

                    self.table_it.advance();
                    return Some((key, value));
                } else {
                    self.exhausted = true;
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
        let mut builder = DiskMapBuilder::new().unwrap();
        builder.insert(0, true).unwrap();
        builder.insert(1, true).unwrap();
        builder.insert(2, true).unwrap();
        builder.insert(3, true).unwrap();
        builder.insert(4, true).unwrap();
        builder.insert(5, true).unwrap();
        let table = builder.finish().unwrap();

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
