use crate::util::example_graphs::create_linear_gs;

use super::*;

#[test]
fn source_nodes_linear() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_linear_gs().unwrap();
    let mut target = LinearGraphStorage::<u8>::new();
    target.copy(&node_annos, &orig).unwrap();

    let nodes: Result<Vec<_>> = target.source_nodes().collect();
    let mut nodes = nodes.unwrap();
    nodes.sort();
    assert_eq!(vec![0, 1, 2, 3, 5, 6, 7, 9], nodes);
}

#[test]
fn outgoing_linear() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_linear_gs().unwrap();
    let mut target = LinearGraphStorage::<u8>::new();
    target.copy(&node_annos, &orig).unwrap();

    // First component
    assert_eq!(
        vec![1],
        target
            .get_outgoing_edges(0)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![2],
        target
            .get_outgoing_edges(1)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![3],
        target
            .get_outgoing_edges(2)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![4],
        target
            .get_outgoing_edges(3)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        0,
        target
            .get_outgoing_edges(4)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );

    // Second component
    assert_eq!(
        vec![6],
        target
            .get_outgoing_edges(5)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![7],
        target
            .get_outgoing_edges(6)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![8],
        target
            .get_outgoing_edges(7)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );

    assert_eq!(
        0,
        target
            .get_outgoing_edges(8)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );

    // Third component
    assert_eq!(
        vec![10],
        target
            .get_outgoing_edges(9)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        0,
        target
            .get_outgoing_edges(10)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );
}

#[test]
fn ingoing_linear() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_linear_gs().unwrap();
    let mut target = LinearGraphStorage::<u8>::new();
    target.copy(&node_annos, &orig).unwrap();

    // First component
    assert_eq!(
        vec![3],
        target
            .get_ingoing_edges(4)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![2],
        target
            .get_ingoing_edges(3)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![1],
        target
            .get_ingoing_edges(2)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![0],
        target
            .get_ingoing_edges(1)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        0,
        target
            .get_ingoing_edges(0)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );

    // Second component
    assert_eq!(
        vec![7],
        target
            .get_ingoing_edges(8)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![6],
        target
            .get_ingoing_edges(7)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        vec![5],
        target
            .get_ingoing_edges(6)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );

    assert_eq!(
        0,
        target
            .get_ingoing_edges(5)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );

    // Third component
    assert_eq!(
        vec![9],
        target
            .get_ingoing_edges(10)
            .collect::<Result<Vec<_>>>()
            .unwrap()
    );
    assert_eq!(
        0,
        target
            .get_ingoing_edges(9)
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .len()
    );
}
