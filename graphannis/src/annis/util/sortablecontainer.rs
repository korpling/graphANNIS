use std::borrow::Cow;

use crate::annis::errors::{GraphAnnisError, Result};
use serde::{de::DeserializeOwned, Serialize};

pub trait SortableContainer<T: Clone>: Send {
    /// Swaps two elements in the container.
    ///
    ///  See also [slice::swap()]
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()>;

    fn try_len(&self) -> Result<usize>;

    fn try_get<'b>(&'b self, index: usize) -> Result<Cow<'b, T>>;
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
}

impl<T> SortableContainer<T> for transient_btree_index::BtreeIndex<usize, T>
where
    T: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        let val_a = self
            .get(&a)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(a))?;
        let val_b = self
            .get(&b)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(b))?;

        self.insert(b, val_a)?;
        self.insert(a, val_b)?;

        Ok(())
    }

    fn try_len(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn try_get<'b>(&'b self, index: usize) -> Result<Cow<'b, T>> {
        let result = self
            .get(&index)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(index))?;
        Ok(Cow::Owned(result))
    }
}
