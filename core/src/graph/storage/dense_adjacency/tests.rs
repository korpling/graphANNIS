use super::*;
use crate::{graph::NODE_TYPE_KEY, types::Annotation, util::example_graphs::create_linear_gs};

#[test]
fn find_connections_dense() {
    let orig = create_linear_gs().unwrap();
    let mut gs = DenseAdjacencyListStorage::new();
    let mut node_annos = AnnoStorageImpl::new();
    for i in 0..=10 {
        node_annos
            .insert(
                i,
                Annotation {
                    key: NODE_TYPE_KEY.as_ref().clone(),
                    val: "node".into(),
                },
            )
            .unwrap();
    }
    gs.copy(&node_annos, &orig).unwrap();

    let found: Result<Vec<NodeID>> = gs.find_connected(0, 2, Bound::Included(3)).collect();
    let mut found = found.unwrap();
    found.sort();
    assert_eq!(vec![2, 3], found);
    assert_eq!(true, gs.is_connected(0, 2, 2, Bound::Included(3)).unwrap());
    assert_eq!(true, gs.is_connected(0, 3, 2, Bound::Included(3)).unwrap());
    assert_eq!(false, gs.is_connected(0, 4, 2, Bound::Included(3)).unwrap());
    assert_eq!(false, gs.is_connected(0, 8, 2, Bound::Included(3)).unwrap());

    let found: Result<Vec<NodeID>> = gs.find_connected(5, 1, Bound::Excluded(3)).collect();
    let mut found = found.unwrap();
    found.sort();
    assert_eq!(vec![6, 7], found);
    assert_eq!(true, gs.is_connected(5, 6, 1, Bound::Excluded(3)).unwrap());
    assert_eq!(true, gs.is_connected(5, 7, 1, Bound::Excluded(3)).unwrap());
    assert_eq!(false, gs.is_connected(5, 8, 1, Bound::Excluded(3)).unwrap());

    let found: Result<Vec<NodeID>> = gs
        .find_connected(0, 2, std::ops::Bound::Unbounded)
        .collect();
    let mut found = found.unwrap();
    found.sort();
    assert_eq!(vec![2, 3, 4], found);
    assert_eq!(true, gs.is_connected(0, 2, 2, Bound::Unbounded).unwrap());
    assert_eq!(true, gs.is_connected(0, 3, 2, Bound::Unbounded).unwrap());
    assert_eq!(true, gs.is_connected(0, 4, 2, Bound::Unbounded).unwrap());
    assert_eq!(false, gs.is_connected(0, 8, 2, Bound::Unbounded).unwrap());
}

#[test]
fn find_connections_inverse_dense() {
    let orig = create_linear_gs().unwrap();
    let mut gs = DenseAdjacencyListStorage::new();
    let mut node_annos = AnnoStorageImpl::new();
    for i in 0..=10 {
        node_annos
            .insert(
                i,
                Annotation {
                    key: NODE_TYPE_KEY.as_ref().clone(),
                    val: "node".into(),
                },
            )
            .unwrap();
    }
    gs.copy(&node_annos, &orig).unwrap();

    let found: Result<Vec<NodeID>> = gs
        .find_connected_inverse(4, 2, std::ops::Bound::Included(3))
        .collect();
    let mut found = found.unwrap();
    found.sort();
    assert_eq!(vec![1, 2], found);

    let found: Result<Vec<NodeID>> = gs
        .find_connected_inverse(3, 1, std::ops::Bound::Excluded(3))
        .collect();
    let mut found = found.unwrap();
    found.sort();
    assert_eq!(vec![1, 2], found);
}
