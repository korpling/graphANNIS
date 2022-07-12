use crate::annis::errors::Result;
use serde::{de::DeserializeOwned, Serialize};
use transient_btree_index::{BtreeConfig, BtreeIndex};

pub trait SortableContainer {
    /// Swaps two elements in the container.
    ///
    ///  See also [slice::swap()]
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()>;
}

impl<T> SortableContainer for Vec<T> {
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        self.swap(a, b);
        Ok(())
    }
}

pub struct DiskBtreeMapSlice<T>
where
    T: Serialize + DeserializeOwned + Clone + Sync,
{
    btree: transient_btree_index::BtreeIndex<usize, T>,
}

impl<T> DiskBtreeMapSlice<T>
where
    T: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
{
    pub fn new() -> Result<DiskBtreeMapSlice<T>> {
        let config = BtreeConfig::default();
        let btree = BtreeIndex::with_capacity(config, 1024)?;
        let result = DiskBtreeMapSlice { btree: btree };
        Ok(result)
    }
}

impl<T> SortableContainer for DiskBtreeMapSlice<T>
where
    T: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        let val_a = self.btree.get(&a)?;
        let val_b = self.btree.get(&b)?;
        if let (Some(val_a), Some(val_b)) = (val_a, val_b) {
            self.btree.insert(b, val_a)?;
            self.btree.insert(a, val_b)?;
        }
        Ok(())
    }
}
