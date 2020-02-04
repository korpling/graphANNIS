use crate::annis::errors::*;
use serde::{Deserialize, Serialize};
use shardio::{ShardReader, ShardSender, ShardWriter};
use sstable::{SSIterator, Table, TableBuilder, TableIterator};

use std::ops::{Bound, RangeBounds};

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
    for<'de> K:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    shard_writer: ShardWriter<Entry<K, V>>,
    shard_sender: ShardSender<Entry<K, V>>,
    tmp_file: tempfile::NamedTempFile,
}

impl<K, V> DiskMapBuilder<K, V>
where
    for<'de> K:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
    for<'de> V:
        'static + Clone + Eq + PartialEq + PartialOrd + Ord + Serialize + Deserialize<'de> + Send,
{
    pub fn new() -> Result<DiskMapBuilder<K, V>> {
        let tmp_file = tempfile::NamedTempFile::new()?;

        let shard_writer: ShardWriter<Entry<K, V>> =
            ShardWriter::new(&tmp_file.path(), 64, 256, 1 << 16)?;

        Ok(DiskMapBuilder {
            tmp_file,
            shard_sender: shard_writer.get_sender(),
            shard_writer,
        })
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        self.shard_sender.send(Entry { key, value })?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<DiskMap<K, V>> {
        // Finish sorting
        self.shard_writer.finish()?;
        // Open sorted shard for reading
        let reader = ShardReader::<Entry<K, V>>::open(self.tmp_file.path())?;
        // Create the indexes by iterating over the sorted entries
        let tmp_file = tempfile::NamedTempFile::new()?;
        let mut table_builder = TableBuilder::new(sstable::Options::default(), tmp_file.as_file());
        for entry in reader.iter()? {
            let entry: Entry<K, V> = entry?;
            table_builder.add(
                &bincode::serialize(&entry.key)?,
                &bincode::serialize(&entry.value)?,
            )?;
        }
        table_builder.finish()?;

        // Open the created index file as table
        let tmp_file = tempfile::NamedTempFile::new()?;
        let table = Table::new_from_file(sstable::Options::default(), tmp_file.path())?;
        Ok(DiskMap {
            table,
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
        let key = bincode::serialize(key)?;
        if let Some(value) = self.table.get(&key)? {
            let value = bincode::deserialize(&value)?;
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
                let start = bincode::serialize(start)?;
                table_it.seek(&start);
            }
            Bound::Excluded(start_bound) => {
                let start = bincode::serialize(start_bound)?;
                table_it.seek(&start);
                let mut key = Vec::default();
                let mut value = Vec::default();

                if table_it.valid() && table_it.current(&mut key, &mut value) {
                    let key : K = bincode::deserialize(&key)
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
                let key = bincode::deserialize(&key)
                    .expect("Could not decode previously written data from disk.");
                if self.range.contains(&key) {
                    let value = bincode::deserialize(&value)
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
