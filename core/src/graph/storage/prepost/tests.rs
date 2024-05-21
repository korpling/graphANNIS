use std::ops::Bound;

use crate::{graph::storage::GraphStorage, util::example_graphs::create_tree_gs};

use super::*;

#[test]
fn test_inverse_edges() {
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
