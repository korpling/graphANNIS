use crate::{
    graph::storage::{adjacencylist::AdjacencyListStorage, WriteableGraphStorage},
    types::{AnnoKey, Annotation},
};

use super::*;

/// Creates an example graph storage with the folllowing structure:
///
/// ```
///  0 -> 1 -> 2 -> 3 -> 4
///  5 -> 6 -> 7 -> 8
///  9 -> 10
/// ```
fn create_linear_gs() -> Result<AdjacencyListStorage> {
    let mut orig = AdjacencyListStorage::new();

    // First component
    orig.add_edge((0, 1).into())?;
    orig.add_edge((1, 2).into())?;
    orig.add_edge((2, 3).into())?;
    orig.add_edge((3, 4).into())?;

    // Second component
    orig.add_edge((5, 6).into())?;
    orig.add_edge((6, 7).into())?;
    orig.add_edge((7, 8).into())?;

    // Third component
    orig.add_edge((9, 10).into())?;

    // Add annotations to edge of the third component
    let key = AnnoKey {
        name: "example".into(),
        ns: "default_ns".into(),
    };
    let anno = Annotation {
        key,
        val: "last".into(),
    };
    orig.add_edge_annotation((9, 10).into(), anno.clone())?;

    Ok(orig)
}

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
