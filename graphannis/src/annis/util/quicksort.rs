use std::ops::Range;

use crate::errors::Result;
use rand::Rng;

use super::sortablecontainer::SortableContainer;

/// Make sure all items of the complete vector are sorted by the given comparision function.
pub fn sort<T, F>(items: &mut dyn SortableContainer<T>, order_func: F) -> Result<()>
where
    T: Clone + Send,
    F: Fn(&T, &T) -> Result<std::cmp::Ordering>,
{
    let item_len = items.try_len()?;
    if item_len > 0 {
        quicksort(items, 0..item_len, item_len, &order_func)?;
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
    order_func: F,
) -> Result<()>
where
    T: Clone + Send,
    F: Fn(&T, &T) -> Result<std::cmp::Ordering>,
{
    let item_len = items.try_len()?;
    if item_len > 0 {
        quicksort(items, 0..item_len, n, &order_func)?;
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
    order_func: &F,
) -> Result<()>
where
    T: Clone,
    F: Fn(&T, &T) -> Result<std::cmp::Ordering>,
{
    if (items_range.end - items_range.start) > 1 {
        let q = randomized_partition(items, items_range.clone(), order_func)?;
        let low_range = items_range.start..q;
        let high_range = q..items_range.end;

        quicksort(items, low_range, max_size, order_func)?;
        if q < max_size {
            // only sort right partition if the left partition is not large enough
            quicksort(items, high_range, max_size, order_func)?;
        }
    }
    Ok(())
}

fn randomized_partition<T, F>(
    items: &mut dyn SortableContainer<T>,
    item_range: Range<usize>,
    order_func: &F,
) -> Result<usize>
where
    T: Clone,
    F: Fn(&T, &T) -> Result<std::cmp::Ordering>,
{
    if (item_range.end - item_range.start) == 1 {
        Ok(item_range.start)
    } else {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(item_range.clone());
        items.try_swap(item_range.end - 1, i)?;
        partition(items, item_range, order_func)
    }
}

fn partition<T, F>(
    items: &mut dyn SortableContainer<T>,
    item_range: Range<usize>,
    order_func: &F,
) -> Result<usize>
where
    T: Clone,
    F: Fn(&T, &T) -> Result<std::cmp::Ordering>,
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

    use rand;
    use rand::distributions::Distribution;
    use rand::Rng;

    #[test]
    fn canary_sort_test() {
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
    fn random_sort_test() {
        // compare 100 random arrays against the standard library sort
        let mut rng = rand::thread_rng();
        let random_item_gen = rand::distributions::Uniform::from(1..100);

        for _i in 0..100 {
            // the arrays should have a size from 10 to 50
            let items_size = rng.gen_range(10..51);
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
}
