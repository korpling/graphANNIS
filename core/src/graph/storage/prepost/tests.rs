use std::ops::Bound;

use crate::{graph::storage::GraphStorage, util::example_graphs::create_tree_gs};

use super::*;

#[test]
fn find_edges_prepost() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_tree_gs().unwrap();
    let mut target = PrePostOrderStorage::<u8, u8>::new();
    target.copy(&node_annos, &orig).unwrap();

    let mut result = target
        .find_connected(3, 1, Bound::Included(1))
        .collect::<Result<Vec<_>>>()
        .unwrap();
    result.sort();
    assert_eq!(vec![5, 6], result);

    let mut result = target
        .find_connected(0, 1, Bound::Excluded(3))
        .collect::<Result<Vec<_>>>()
        .unwrap();
    result.sort();
    assert_eq!(vec![1, 2, 3, 4], result);
}

#[test]
fn inverse_edges_prepost() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_tree_gs().unwrap();
    let mut target = PrePostOrderStorage::<u8, u8>::new();
    target.copy(&node_annos, &orig).unwrap();
    assert_eq!(
        vec![1],
        target
            .find_connected_inverse(3, 1, Bound::Included(1))
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );

    assert_eq!(
        vec![2],
        target
            .find_connected_inverse(7, 2, Bound::Excluded(3))
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
}

#[test]
fn is_connected_prepost() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_tree_gs().unwrap();
    let mut target = PrePostOrderStorage::<u8, u8>::new();
    target.copy(&node_annos, &orig).unwrap();

    assert_eq!(
        true,
        target.is_connected(3, 5, 1, Bound::Included(1)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(3, 6, 1, Bound::Included(1)).unwrap()
    );
    assert_eq!(
        false,
        target.is_connected(4, 6, 1, Bound::Included(1)).unwrap()
    );

    assert_eq!(
        true,
        target.is_connected(0, 1, 1, Bound::Excluded(3)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(0, 2, 1, Bound::Excluded(3)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(0, 3, 1, Bound::Excluded(3)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(0, 4, 1, Bound::Excluded(3)).unwrap()
    );
    assert_eq!(
        false,
        target.is_connected(0, 7, 1, Bound::Excluded(3)).unwrap()
    );
}
