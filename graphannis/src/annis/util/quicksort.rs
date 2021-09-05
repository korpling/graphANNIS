use rand::Rng;
/// Make sure that the first `n` items of the complete vector are sorted by the given comparision function.
///
/// This returns the original items and it is guaranteed that the items (0..n) are
/// sorted and that all of these items are smaller or equal to the n-th item.
pub fn sort_first_n_items<T, F>(items: &mut Vec<T>, n: usize, order_func: F)
where
    T: Send,
    F: Fn(&T, &T) -> std::cmp::Ordering + Sync,
{
    let item_len = items.len();
    if item_len > 0 {
        quicksort(items, n, &order_func);
    }
}

/// Classic implementation of a quicksort algorithm, see Cormen et al. 2009 "Introduction to Algorithms" p. 170ff
/// for the specific algorithm used as a base here.
///
/// The algorithm has been modified to accept a `max_size` parameter which allows to abort the algorithm
/// if at least `max_size` items at the beginning of the vector have been sorted.
///
/// The algorithm used a randomized pivot element and is executed in parallel.
fn quicksort<T, F>(items: &mut [T], max_size: usize, order_func: &F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    if items.len() > 1 {
        let q = randomized_partition(items, order_func);
        let (lo, hi) = items.split_at_mut(q);

        quicksort(lo, max_size, order_func);
        if q < max_size {
            // only sort right partition if the left partition is not large enough
            quicksort(hi, max_size, order_func);
        }
    }
}

/// Make sure that the first `n` items of the complete vector are sorted by the given comparision function.
///
/// This returns the original items and it is guaranteed that the items (0..n) are
/// sorted and that all of these items are smaller or equal to the n-th item.
pub fn sort_first_n_items_parallel<T, F>(items: &mut Vec<T>, n: usize, order_func: F)
where
    T: Send,
    F: Fn(&T, &T) -> std::cmp::Ordering + Sync,
{
    let item_len = items.len();
    if item_len > 0 {
        quicksort_parallel(items, n, &order_func);
    }
}

/// Classic implementation of a quicksort algorithm, see Cormen et al. 2009 "Introduction to Algorithms" p. 170ff
/// for the specific algorithm used as a base here.
///
/// The algorithm has been modified to accept a `max_size` parameter which allows to abort the algorithm
/// if at least `max_size` items at the beginning of the vector have been sorted.
///
/// The algorithm used a randomized pivot element and is executed in parallel.
fn quicksort_parallel<T, F>(items: &mut [T], max_size: usize, order_func: &F)
where
    T: Send,
    F: Fn(&T, &T) -> std::cmp::Ordering + Sync,
{
    if items.len() > 1 {
        let q = randomized_partition(items, order_func);
        let (lo, hi) = items.split_at_mut(q);

        rayon::join(
            || quicksort_parallel(lo, max_size, order_func),
            || {
                if q < max_size {
                    // only sort right partition if the left partition is not large enough
                    quicksort_parallel(hi, max_size, order_func);
                }
            },
        );
    }
}

fn randomized_partition<T, F>(items: &mut [T], order_func: &F) -> usize
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let items_len = items.len();
    if items_len == 0 {
        0
    } else {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..items_len);
        items.swap(items_len - 1, i);
        partition(items, order_func)
    }
}

fn partition<T, F>(items: &mut [T], order_func: &F) -> usize
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let r = items.len() - 1;

    let mut i = 0;

    for j in 0..(items.len() - 1) {
        let comparision = order_func(&items[j], &items[r]);
        match comparision {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                items.swap(i, j);
                i += 1;
            }
            _ => {}
        }
    }

    items.swap(i, r);

    i
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
    fn canary_sort_test_parallel() {
        let mut items = vec![4, 10, 100, 4, 5];
        let num_items = items.len();
        super::sort_first_n_items_parallel(&mut items, num_items, |x, y| x.cmp(y));
        assert_eq!(vec![4, 4, 5, 10, 100], items);

        let mut items: Vec<usize> = vec![];
        super::sort_first_n_items_parallel(&mut items, 0, |x, y| x.cmp(y));
        let empty_items: Vec<usize> = vec![];
        assert_eq!(empty_items, items);

        let mut items: Vec<usize> = vec![1];
        super::sort_first_n_items_parallel(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1], items);

        let mut items: Vec<usize> = vec![1, 2];
        super::sort_first_n_items_parallel(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![2, 1];
        super::sort_first_n_items_parallel(&mut items, 0, |x, y| x.cmp(y));
        assert_eq!(vec![1, 2], items);

        let mut items: Vec<usize> = vec![1, 2, 3, 4, 5];
        super::sort_first_n_items_parallel(&mut items, 0, |x, y| x.cmp(y));
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
            sorted_by_stdlib.sort();
            super::sort_first_n_items(&mut items, items_size, |x, y| x.cmp(y));
            assert_eq!(items, sorted_by_stdlib);
        }
    }

    #[test]
    fn random_sort_test_parallel() {
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
            super::sort_first_n_items_parallel(&mut items, items_size, |x, y| x.cmp(y));
            assert_eq!(items, sorted_by_stdlib);
        }
    }
}
