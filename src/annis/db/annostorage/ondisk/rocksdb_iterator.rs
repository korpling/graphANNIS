use crate::annis::errors::*;
use crate::annis::types::{AnnoKey, Annotation};
use rocksdb::{DBRawIterator, DB};

use std::marker::PhantomData;
use std::sync::Arc;

pub struct AnnotationValueIterator<'a, T> {
    raw: DBRawIterator<'a>,
    anno_key: Arc<AnnoKey>,
    upper_bound: Vec<u8>,
    exhausted: bool,
    phantom: PhantomData<T>,
}

impl<'a, T> AnnotationValueIterator<'a, T>
where
    T: super::KeyProvider,
{
    pub fn new(
        db: &'a DB,
        cf: &'a rocksdb::ColumnFamily,
        anno_key: Arc<AnnoKey>,
        value: Option<String>,
    ) -> Result<AnnotationValueIterator<'a, T>> {
       
        // restrict search to qualified name prefix
        let prefix: Vec<u8> = super::create_str_vec_key(&[&anno_key.ns, &anno_key.name]);
        let it = db.prefix_iterator_cf(&cf, prefix)?;

        let lower_bound = Annotation {
            key: anno_key.as_ref().clone(),
            val: if let Some(value) = &value {
                value.to_string()
            } else {
                "".to_string()
            },
        };

        let upper_bound = Annotation {
            key: anno_key.as_ref().clone(),
            val: if let Some(value) = value {
                value
            } else {
                std::char::MAX.to_string()
            },
        };

        let lower_bound = super::create_by_anno_qname_key(&T::min_value(), &lower_bound);
        let upper_bound = super::create_by_anno_qname_key(&T::max_value(), &upper_bound);

        let mut raw: DBRawIterator = it.into();

        raw.seek(lower_bound);

        Ok(AnnotationValueIterator {
            raw,
            anno_key,
            upper_bound,
            exhausted: false,
            phantom: PhantomData,
        })
    }
}

impl<'a, T> Iterator for AnnotationValueIterator<'a, T>
where
    T: super::KeyProvider,
{
    type Item = (T, Arc<AnnoKey>);

    fn next(&mut self) -> Option<(T, Arc<AnnoKey>)> {
        if !self.exhausted {
            if self.raw.valid() {
                // get the current item
                if let Some(key) = self.raw.key() {
                    // check if item has reached the upper bound
                    if key < &self.upper_bound[..] {
                        // parse the node ID from this item
                        let item_id = T::from_key(&key[(key.len() - T::key_size())..]);
                        // set iterator to next item
                        self.raw.next();

                        return Some((item_id, self.anno_key.clone()));
                    } else {
                        // iterator is exhausted: make sure that raw.next() is not called again
                        self.exhausted = true;
                    }
                }
            } else {
                self.exhausted = true;
            }
        }
        None
    }
}
