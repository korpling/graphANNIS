use crate::{
    graph::storage::{adjacencylist::AdjacencyListStorage, GraphStorage, WriteableGraphStorage},
    types::{AnnoKey, Annotation},
};

use super::*;

/// Creates an example graph storage with the folllowing structure:
///
/// ```
///           0
///          / \
///         1   2
///        /     \
///       3       4
///      / \     / \
///     5   6   7   8
/// ```
fn create_tree_gs() -> Result<AdjacencyListStorage> {
    let mut orig = AdjacencyListStorage::new();

    // First layer
    orig.add_edge((0, 1).into())?;
    orig.add_edge((0, 2).into())?;

    // Second layer
    orig.add_edge((1, 3).into())?;
    orig.add_edge((2, 4).into())?;

    // Third layer
    orig.add_edge((3, 5).into())?;
    orig.add_edge((3, 6).into())?;
    orig.add_edge((4, 7).into())?;
    orig.add_edge((4, 8).into())?;

    // Add annotations to last layer
    let key = AnnoKey {
        name: "example".into(),
        ns: "default_ns".into(),
    };
    let anno = Annotation {
        key,
        val: "last".into(),
    };
    orig.add_edge_annotation((3, 5).into(), anno.clone())?;
    orig.add_edge_annotation((3, 6).into(), anno.clone())?;
    orig.add_edge_annotation((4, 7).into(), anno.clone())?;
    orig.add_edge_annotation((4, 8).into(), anno.clone())?;

    Ok(orig)
}

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
