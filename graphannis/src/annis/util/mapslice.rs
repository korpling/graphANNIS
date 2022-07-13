use std::borrow::Cow;

use crate::annis::errors::{GraphAnnisError, Result};
use serde::{de::DeserializeOwned, Serialize};
use transient_btree_index::{BtreeConfig, BtreeIndex};

pub trait SortableContainer<T: Clone>: Send {
    /// Swaps two elements in the container.
    ///
    ///  See also [slice::swap()]
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()>;

    fn try_len(&self) -> Result<usize>;

    fn try_get<'b>(&'b self, index: usize) -> Result<Cow<'b, T>>;

    fn try_split_off(&mut self, at: usize) -> Result<Box<dyn SortableContainer<T>>>;
}

impl<T> SortableContainer<T> for Vec<T>
where
    T: Clone + Send + 'static,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        self.swap(a, b);
        Ok(())
    }

    fn try_len(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn try_get<'b>(&'b self, index: usize) -> Result<Cow<'b, T>> {
        if let Some(result) = self.get(index) {
            Ok(Cow::Borrowed(result))
        } else {
            Err(GraphAnnisError::IndexOutOfBounds(index))
        }
    }

    fn try_split_off(&mut self, at: usize) -> Result<Box<dyn SortableContainer<T>>> {
        let new_vec = self.split_off(at);
        Ok(Box::new(new_vec))
    }
}

impl<T> SortableContainer<T> for transient_btree_index::BtreeIndex<usize, Option<T>>
where
    T: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        let val_a = self
            .get(&a)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(a))?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(a))?;
        let val_b = self
            .get(&b)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(b))?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(b))?;

        self.insert(b, Some(val_a))?;
        self.insert(a, Some(val_b))?;

        Ok(())
    }

    fn try_len(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn try_get<'b>(&'b self, index: usize) -> Result<Cow<'b, T>> {
        let result = self
            .get(&index)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(index))?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(index))?;
        Ok(Cow::Owned(result))
    }

    fn try_split_off(&mut self, at: usize) -> Result<Box<dyn SortableContainer<T>>> {
        // Create a new container which contains the right side
        let mut new_container = BtreeIndex::with_capacity(BtreeConfig::default(), self.len() - at)?;
        for i in at..self.len() {
            let v = self
                .get(&i)?
                .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(i))?
                .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(i))?;
            // Insert into new container
            new_container.insert(i, Some(v))?;
            // Remove from the old one by adding a tombstone entry
            self.insert(i, None)?;
        }
        Ok(Box::new(new_container))
    }
}
