use std;

pub fn sort_first_n_items<T, F>(items: &mut Vec<T>, n: usize, order_func: F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let item_len = items.len();
    quicksort(items, 0, item_len - 1, n, &order_func);
}

fn quicksort<T, F>(items: &mut Vec<T>, p: usize, r: usize, max_size: usize, order_func: &F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    if p < r {
        let q = partition(items, p, r, order_func);
        quicksort(items, p, q - 1, max_size, order_func);
        if (q-p) < max_size { 
            // only sort right partition if the left partition is not large enough
            quicksort(items, q + 1, r, max_size, order_func);
        }
    }
}

fn partition<T, F>(items: &mut Vec<T>, p: usize, r: usize, order_func: &F) -> usize
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let mut i = if p == 0 { None } else { Some(p - 1) };
    for j in p..r {
        let comparision = order_func(&items[j], &items[r]);
        match comparision {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                i = if let Some(i) = i {
                    Some(i + 1)
                } else {
                    Some(0)
                };
                items.swap(i.unwrap(), j);
            }
            _ => {}
        }
    }
    i = if let Some(i) = i {
        Some(i + 1)
    } else {
        Some(0)
    };

    items.swap(i.unwrap(), r);

    i.unwrap()
}

#[cfg(test)]
mod test {

    #[test]
    fn canary_sort_test() {
        let mut items = vec![4, 10, 100, 4, 5];
        let num_items = items.len();
        super::sort_first_n_items(&mut items, num_items, |x, y| x.cmp(y));
        assert_eq!(vec![4, 4, 5, 10, 100], items);
    }
}
