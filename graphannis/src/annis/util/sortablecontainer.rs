use std::borrow::Cow;

use crate::annis::errors::{GraphAnnisError, Result};
use serde::{de::DeserializeOwned, Serialize};

pub trait SortableContainer<T: Clone>: Send {
    /// Swaps two elements in the container.
    ///
    ///  See also [slice::swap()]
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()>;

    fn try_len(&self) -> Result<usize>;

    fn try_get(&self, index: usize) -> Result<Cow<T>>;

    fn try_set(&mut self, index: usize, val: T) -> Result<()>;
}

impl<T> SortableContainer<T> for Vec<T>
where
    T: Clone + Send,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        if a >= self.len() {
            return Err(GraphAnnisError::IndexOutOfBounds(a));
        }
        if b >= self.len() {
            return Err(GraphAnnisError::IndexOutOfBounds(b));
        }
        if a != b {
            self.swap(a, b);
        }
        Ok(())
    }

    fn try_len(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn try_get(&self, index: usize) -> Result<Cow<T>> {
        if let Some(result) = self.get(index) {
            Ok(Cow::Borrowed(result))
        } else {
            Err(GraphAnnisError::IndexOutOfBounds(index))
        }
    }

    fn try_set(&mut self, index: usize, val: T) -> Result<()> {
        self[index] = val;
        Ok(())
    }
}

impl<T> SortableContainer<T> for transient_btree_index::BtreeIndex<usize, T>
where
    T: Serialize + DeserializeOwned + Clone + Sync + Send + 'static,
{
    fn try_swap(&mut self, a: usize, b: usize) -> Result<()> {
        if a != b {
            self.swap(&a, &b)?;
        }
        Ok(())
    }

    fn try_len(&self) -> Result<usize> {
        Ok(self.len())
    }

    fn try_get(&self, index: usize) -> Result<Cow<T>> {
        let result = self
            .get(&index)?
            .ok_or_else(|| GraphAnnisError::IndexOutOfBounds(index))?;
        Ok(Cow::Owned(result))
    }

    fn try_set(&mut self, index: usize, val: T) -> Result<()> {
        self.insert(index, val)?;
        Ok(())
    }
}
