use rand::Rng;
use std;

/// Make sure that the first `n` items of the complete vector are sorted by the given comparision function.
/// 
/// This returns the original items and it is guaranteed that the items (0..n) are
/// sorted and that all of these items are smaller or equal to the n-th item.
pub fn sort_first_n_items<T, F>(items: &mut Vec<T>, n: usize, order_func: F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let item_len = items.len();
    if item_len > 0 {
        quicksort(items, 0, item_len - 1, n, &order_func);
    }
}

/// Classic implementation of a quicksort algorithm, see Cormen et al. 2009 "Introduction to Algorithms" p. 170ff
/// for the specific algorithm used as a base here.
///
/// The algorithm has been modified to accept a `max_size` parameter which allows to abort the algorithm
/// if at least `max_size` items at the beginning of the vector have been sorted.
///
/// The algorithm used a randomized pivot element.
fn quicksort<T, F>(
    items: &mut [T],
    p: usize,
    r: usize,
    max_size: usize,
    order_func: &F,
) where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    if p < r {
        let q = randomized_partition(items, p, r, order_func);
        if q > 0 {
            quicksort(items, p, q - 1, max_size, order_func);
        }
        if (q - p) < max_size {
            // only sort right partition if the left partition is not large enough
            quicksort(items, q + 1, r, max_size, order_func);
        }
    }
}

fn randomized_partition<T, F>(items: &mut [T], p: usize, r: usize, order_func: &F) -> usize
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let mut rng = rand::thread_rng();
    let i = rng.gen_range(p, r+1);
    items.swap(r, i);
    partition(items, p, r, order_func)
}

fn partition<T, F>(items: &mut [T], p: usize, r: usize, order_func: &F) -> usize
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let mut i = p;
    for j in p..r {
        let comparision = order_func(&items[j], &items[r]);
        match comparision {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                items.swap(i, j);
                i = i + 1;
            }
            _ => {}
        }
    }

    items.swap(i, r);

    i
}

#[cfg(test)]
mod test {

    use rand::distributions::{Distribution, Range};
    use rand::Rng;

    #[test]
    fn canary_sort_test() {
        let mut items = vec![4, 10, 100, 4, 5];
        let num_items = items.len();
        super::sort_first_n_items(&mut items, num_items, |x, y| x.cmp(y));
        assert_eq!(vec![4, 4, 5, 10, 100], items);

        let mut items: Vec<usize> = vec![];
        super::sort_first_n_items(&mut items, 0, |x, y| x.cmp(y));
        let empty_items: Vec<usize> = vec![];
        assert_eq!(empty_items, items);

        let mut items: Vec<usize> = vec![1];
        super::sort_first_n_items(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1], items);

        let mut items: Vec<usize> = vec![1, 2];
        super::sort_first_n_items(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![2, 1];
        super::sort_first_n_items(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![1, 2, 3, 4, 5];
        super::sort_first_n_items(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1, 2, 3, 4, 5], items);
    }

    

    #[test]
    fn random_sort_test() {
        // compare 100 random arrays against the standard library sort
        let mut rng = rand::thread_rng();
        let random_item_gen = Range::new(1, 100);

        for _i in 0..100 {
            // the arrays should have a size from 10 to 50
            let items_size = rng.gen_range(10, 51);
            let mut items = Vec::with_capacity(items_size);
            for _j in 0..items_size {
                items.push(random_item_gen.sample(&mut rng));
            }

            let mut sorted_by_stdlib = items.clone();
            sorted_by_stdlib.sort();
            super::sort_first_n_items(&mut items, items_size, |x, y| x.cmp(y));
            assert_eq!(items, sorted_by_stdlib);
        }
    }
}
