use crate::annis::errors::*;
use serde::{Deserialize, Serialize};
use shardio::{ShardReader, ShardSender, ShardWriter};
use sstable::{Table, TableBuilder};

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

        let mut shard_writer: ShardWriter<Entry<K, V>> =
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
        let mut table_builder =
            sstable::TableBuilder::new(sstable::Options::default(), tmp_file.as_file());
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
            phantom_key: std::marker::PhantomData,
            phantom_value: std::marker::PhantomData,
        })
    }
}

pub struct DiskMap<K, V>
where
    for<'de> K: 'static + Serialize + Deserialize<'de> + Send,
    for<'de> V: 'static + Serialize + Deserialize<'de> + Send,
{
    table: Table,
    phantom_key: std::marker::PhantomData<K>,
    phantom_value: std::marker::PhantomData<V>,
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
}
