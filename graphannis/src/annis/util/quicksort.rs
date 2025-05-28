use std::ops::Range;

use crate::errors::Result;
use rand::prelude::*;

use super::sortablecontainer::SortableContainer;

/// Make sure all items of the complete vector are sorted by the given comparision function.
pub fn sort<T, F>(items: &mut dyn SortableContainer<T>, mut order_func: F) -> Result<()>
where
    T: Clone + Send,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    let item_len = items.try_len()?;
    if item_len > 0 {
        quicksort(items, 0..item_len, item_len, &mut order_func)?;
    }
    Ok(())
}

/// Make sure that the first `n` items of the complete vector are sorted by the given comparision function.
///
/// This returns the original items and it is guaranteed that the items (0..n) are
/// sorted and that all of these items are smaller or equal to the n-th item.
pub fn sort_first_n_items<T, F>(
    items: &mut dyn SortableContainer<T>,
    n: usize,
    mut order_func: F,
) -> Result<()>
where
    T: Clone + Send,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    let item_len = items.try_len()?;
    if item_len > 0 {
        quicksort(items, 0..item_len, n, &mut order_func)?;
    }
    Ok(())
}

/// Sort the given range using insertion sort.
fn insertion_sort<T, F>(
    items: &mut dyn SortableContainer<T>,
    items_range: Range<usize>,
    order_func: &mut F,
) -> Result<()>
where
    T: Clone,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    for i in items_range.start..items_range.end {
        for j in (items_range.start..i).rev() {
            if order_func(items.try_get(j)?.as_ref(), items.try_get(j + 1)?.as_ref())?.is_ge() {
                items.try_swap(j, j + 1)?;
            } else {
                break;
            }
        }
    }
    Ok(())
}

/// Classic implementation of a quicksort algorithm, see Cormen et al. 2009 "Introduction to Algorithms" p. 170ff
/// for the specific algorithm used as a base here.
///
/// The algorithm has been modified to accept a `max_size` parameter which allows to abort the algorithm
/// if at least `max_size` items at the beginning of the vector have been sorted.
///
/// The algorithm used a randomized pivot element.
fn quicksort<T, F>(
    items: &mut dyn SortableContainer<T>,
    items_range: Range<usize>,
    max_size: usize,
    order_func: &mut F,
) -> Result<()>
where
    T: Clone,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    let range_size = items_range.end - items_range.start;
    if range_size > 1 {
        if range_size <= 20 {
            insertion_sort(items, items_range, order_func)?;
        } else {
            let q = randomized_partition(items, items_range.clone(), order_func)?;
            let low_range = items_range.start..q;
            let high_range = q..items_range.end;

            quicksort(items, low_range, max_size, order_func)?;

            // only sort right partition if the left partition is not large enough
            if q < max_size {
                quicksort(items, high_range, max_size, order_func)?;
            }
        }
    }
    Ok(())
}

fn randomized_partition<T, F>(
    items: &mut dyn SortableContainer<T>,
    item_range: Range<usize>,
    order_func: &mut F,
) -> Result<usize>
where
    T: Clone,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    if (item_range.end - item_range.start) == 1 {
        Ok(item_range.start)
    } else {
        let mut rng = rand::rng();
        // Use the median of 3 random positions as pivot
        let i1 = rng.random_range(item_range.clone());
        let i2 = rng.random_range(item_range.clone());
        let i3 = rng.random_range(item_range.clone());

        let v1 = (i1, items.try_get(i1)?.into_owned());
        let v2 = (i2, items.try_get(i2)?.into_owned());
        let v3 = (i3, items.try_get(i3)?.into_owned());

        let mut v = [v1, v2, v3];
        // Adapted median creation using swaps from
        // https://en.wikipedia.org/wiki/Quicksort#Choice_of_pivot
        let low = 0;
        let mid = 1;
        let high = 2;
        if order_func(&v[mid].1, &v[low].1)?.is_lt() {
            v.swap(low, mid);
        }
        if order_func(&v[high].1, &v[low].1)?.is_lt() {
            v.swap(low, high)
        }
        if order_func(&v[mid].1, &v[high].1)?.is_lt() {
            v.swap(mid, high)
        }
        let pivot = v[high].0;
        // Position the pivot element at the end
        items.try_swap(item_range.end - 1, pivot)?;
        partition(items, item_range, order_func)
    }
}

fn partition<T, F>(
    items: &mut dyn SortableContainer<T>,
    item_range: Range<usize>,
    order_func: &mut F,
) -> Result<usize>
where
    T: Clone,
    F: FnMut(&T, &T) -> Result<std::cmp::Ordering>,
{
    let r = item_range.end - 1;
    let item_r = items.try_get(r)?.into_owned();

    let mut i = item_range.start;

    for j in item_range.start..(item_range.end - 1) {
        let item_j = items.try_get(j)?;

        let comparision = order_func(item_j.as_ref(), &item_r)?;
        match comparision {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                items.try_swap(i, j)?;
                i += 1;
            }
            _ => {}
        }
    }

    items.try_swap(i, r)?;

    Ok(i)
}

#[cfg(test)]
mod test {

    use rand::prelude::*;
    use serde::{de::DeserializeOwned, Serialize};
    use transient_btree_index::{BtreeConfig, BtreeIndex};

    fn index_from_vec<V>(items: Vec<V>) -> BtreeIndex<usize, V>
    where
        V: 'static + Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        let mut result = BtreeIndex::with_capacity(BtreeConfig::default(), items.len()).unwrap();
        for i in 0..items.len() {
            result.insert(i, items[i].clone()).unwrap();
        }
        result
    }

    fn index_to_vec<V>(index: BtreeIndex<usize, V>) -> Vec<V>
    where
        V: 'static + Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        let mut result = Vec::with_capacity(index.len());
        for i in 0..index.len() {
            result.push(index.get(&i).unwrap().unwrap());
        }
        result
    }

    #[test]
    fn canary_sort_vec() {
        let mut items: Vec<usize> = vec![2, 1];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2], items);

        let mut items = vec![
            4, 10, 100, 4, 5, 20, 10, 5, 10, 32, 42, 5, 1, 1, 3, 101, 202, 99, 42, 23, 56,
        ];
        let num_items = items.len();
        super::sort_first_n_items(&mut items, num_items, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(
            vec![1, 1, 3, 4, 4, 5, 5, 5, 10, 10, 10, 20, 23, 32, 42, 42, 56, 99, 100, 101, 202],
            items
        );

        let mut items = vec![4, 10, 100, 4, 5];
        let num_items = items.len();
        super::sort_first_n_items(&mut items, num_items, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![4, 4, 5, 10, 100], items);

        let mut items: Vec<usize> = vec![];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        let empty_items: Vec<usize> = vec![];
        assert_eq!(empty_items, items);

        let mut items: Vec<usize> = vec![1];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1], items);

        let mut items: Vec<usize> = vec![1, 2];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![2, 1];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![1, 2, 3, 4, 5];
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2, 3, 4, 5], items);
    }

    #[test]
    fn random_sort_vec() {
        // compare 100 random arrays against the standard library sort
        let mut rng = rand::rng();
        let random_item_gen = rand::distr::Uniform::new(1, 100).unwrap();

        for _i in 0..100 {
            // the arrays should have a size from 40 to 50
            let items_size = rng.random_range(40..51);
            let mut items = Vec::with_capacity(items_size);
            for _j in 0..items_size {
                items.push(random_item_gen.sample(&mut rng));
            }

            let mut sorted_by_stdlib = items.clone();
            sorted_by_stdlib.sort_unstable();
            super::sort_first_n_items(&mut items, items_size, |x, y| Ok(x.cmp(y))).unwrap();
            assert_eq!(items, sorted_by_stdlib);
        }
    }

    #[test]
    fn canary_sort_btree() {
        let mut items = index_from_vec(vec![
            4, 10, 100, 4, 5, 20, 10, 5, 10, 32, 42, 5, 1, 1, 3, 101, 202, 99, 42, 23, 56,
        ]);
        let num_items = items.len();
        super::sort_first_n_items(&mut items, num_items, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(
            vec![1, 1, 3, 4, 4, 5, 5, 5, 10, 10, 10, 20, 23, 32, 42, 42, 56, 99, 100, 101, 202],
            index_to_vec(items)
        );

        let mut items = index_from_vec(vec![4, 10, 100, 4, 5]);
        let num_items = items.len();

        super::sort_first_n_items(&mut items, num_items, |x, y| Ok(x.cmp(y))).unwrap();

        assert_eq!(vec![4, 4, 5, 10, 100], index_to_vec(items));

        let mut items: BtreeIndex<usize, usize> = index_from_vec(vec![]);
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        let empty_items: Vec<usize> = vec![];
        assert_eq!(empty_items, index_to_vec(items));

        let mut items = index_from_vec(vec![1]);
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1], index_to_vec(items));

        let mut items = index_from_vec(vec![1, 2]);
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2], index_to_vec(items));

        let mut items = index_from_vec(vec![2, 1]);
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2], index_to_vec(items));

        let mut items = index_from_vec(vec![1, 2, 3, 4, 5]);
        super::sort_first_n_items(&mut items, 0, |x, y| Ok(x.cmp(y))).unwrap();
        assert_eq!(vec![1, 2, 3, 4, 5], index_to_vec(items));
    }

    #[test]
    fn random_sort_btree() {
        // compare 100 random arrays against the standard library sort
        let mut rng = rand::rng();
        let random_item_gen = rand::distr::Uniform::new(1, 100).unwrap();

        for _i in 0..100 {
            // the arrays should have a size from 40 to 50
            let items_size = rng.random_range(40..51);
            let mut items = BtreeIndex::with_capacity(BtreeConfig::default(), items_size).unwrap();
            let mut items_vec = Vec::new();
            for j in 0..items_size {
                let v = random_item_gen.sample(&mut rng);
                items.insert(j, v).unwrap();
                items_vec.push(v);
            }

            let mut sorted_by_stdlib = items_vec;
            sorted_by_stdlib.sort_unstable();
            super::sort_first_n_items(&mut items, items_size, |x, y| Ok(x.cmp(y))).unwrap();
            assert_eq!(index_to_vec(items), sorted_by_stdlib);
        }
    }
}
