use super::*;
use crate::{
    graph::storage::{adjacencylist::AdjacencyListStorage, WriteableGraphStorage},
    types::{AnnoKey, Annotation},
};
use pretty_assertions::assert_eq;

/// Creates an example graph storage with the folllowing structure:
///
/// ```
/// 0   1   2   3  4    5
///  \ /     \ /    \  /
///   6       7       8
///   |       |       |
///   9      10      11
///    \      |      /
///     \     |     /
///      \    |    /
///       \   |   /
///        \  |  /
///           12
///   
/// ```
fn create_topdown_gs() -> Result<AdjacencyListStorage> {
    let mut orig = AdjacencyListStorage::new();

    // First layer
    orig.add_edge((0, 6).into())?;
    orig.add_edge((1, 6).into())?;
    orig.add_edge((2, 7).into())?;
    orig.add_edge((3, 7).into())?;
    orig.add_edge((4, 8).into())?;
    orig.add_edge((5, 8).into())?;

    // Second layer
    orig.add_edge((6, 9).into())?;
    orig.add_edge((7, 10).into())?;
    orig.add_edge((8, 11).into())?;

    // Third layer
    orig.add_edge((9, 12).into())?;
    orig.add_edge((10, 12).into())?;
    orig.add_edge((11, 12).into())?;

    // Add annotations to last layer
    let key = AnnoKey {
        name: "example".into(),
        ns: "default_ns".into(),
    };
    let anno = Annotation {
        key,
        val: "last".into(),
    };
    orig.add_edge_annotation((9, 12).into(), anno.clone())?;
    orig.add_edge_annotation((10, 12).into(), anno.clone())?;
    orig.add_edge_annotation((11, 12).into(), anno.clone())?;

    Ok(orig)
}

#[test]
fn test_source_nodes() {
    // Create an example graph storage to copy the value from
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    let result: Result<Vec<_>> = target.source_nodes().collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], result);
}

#[test]
fn test_outgoing_edges() {
    // Create an example graph storage to copy the value from
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    let result: Result<Vec<_>> = target.get_outgoing_edges(0).collect();
    assert_eq!(vec![6], result.unwrap());

    let result: Result<Vec<_>> = target.get_outgoing_edges(3).collect();
    assert_eq!(vec![7], result.unwrap());

    let result: Result<Vec<_>> = target.get_outgoing_edges(7).collect();
    assert_eq!(vec![10], result.unwrap());

    let result: Result<Vec<_>> = target.get_outgoing_edges(11).collect();
    assert_eq!(vec![12], result.unwrap());

    let result: Result<Vec<_>> = target.get_outgoing_edges(12).collect();
    assert_eq!(0, result.unwrap().len());

    let result: Result<Vec<_>> = target.get_outgoing_edges(100).collect();
    assert_eq!(0, result.unwrap().len());
}

#[test]
fn test_ingoing_edges() {
    // Create an example graph storage to copy the value from
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    let result: Result<Vec<_>> = target.get_ingoing_edges(12).collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![9, 10, 11], result);

    let result: Result<Vec<_>> = target.get_ingoing_edges(10).collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![7], result);

    let result: Result<Vec<_>> = target.get_ingoing_edges(8).collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![4, 5], result);

    let result: Result<Vec<_>> = target.get_ingoing_edges(0).collect();
    assert_eq!(0, result.unwrap().len());

    let result: Result<Vec<_>> = target.get_ingoing_edges(1).collect();
    assert_eq!(0, result.unwrap().len());

    let result: Result<Vec<_>> = target.get_ingoing_edges(100).collect();
    assert_eq!(0, result.unwrap().len());
}

#[test]
fn test_find_connected() {
    // Create an example graph storage to copy the value from
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    let result: Result<Vec<_>> = target
        .find_connected(0, 0, std::ops::Bound::Unbounded)
        .collect();
    assert_eq!(vec![0, 6, 9, 12], result.unwrap());

    let result: Result<Vec<_>> = target
        .find_connected(0, 1, std::ops::Bound::Unbounded)
        .collect();
    assert_eq!(vec![6, 9, 12], result.unwrap());

    let result: Result<Vec<_>> = target
        .find_connected(1, 0, std::ops::Bound::Unbounded)
        .collect();
    assert_eq!(vec![1, 6, 9, 12], result.unwrap());

    let result: Result<Vec<_>> = target
        .find_connected(7, 1, std::ops::Bound::Included(2))
        .collect();
    assert_eq!(vec![10, 12], result.unwrap());

    let result: Result<Vec<_>> = target
        .find_connected(7, 1, std::ops::Bound::Included(1))
        .collect();
    assert_eq!(vec![10], result.unwrap());

    let result: Result<Vec<_>> = target
        .find_connected(7, 1, std::ops::Bound::Excluded(1))
        .collect();
    // Excluding distance 1 means there can't be any valid resut
    assert_eq!(0, result.unwrap().len());

    let result: Result<Vec<_>> = target
        .find_connected(10, 1, std::ops::Bound::Unbounded)
        .collect();
    assert_eq!(vec![12], result.unwrap());
}

#[test]
fn test_find_connected_inverse() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    let result: Result<Vec<_>> = target
        .find_connected_inverse(12, 0, Bound::Unbounded)
        .collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], result);

    let result: Result<Vec<_>> = target
        .find_connected_inverse(12, 1, Bound::Excluded(2))
        .collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![9, 10, 11], result);

    let result: Result<Vec<_>> = target
        .find_connected_inverse(10, 1, Bound::Included(2))
        .collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![2, 3, 7], result);

    let result: Result<Vec<_>> = target
        .find_connected_inverse(12, 3, Bound::Included(3))
        .collect();
    let mut result = result.unwrap();
    result.sort();
    assert_eq!(vec![0, 1, 2, 3, 4, 5], result);
}

#[test]
fn test_distance() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    assert_eq!(None, target.distance(7, 7).unwrap());
    assert_eq!(None, target.distance(12, 1).unwrap());
    assert_eq!(Some(1), target.distance(0, 6).unwrap());
    assert_eq!(Some(1), target.distance(3, 7).unwrap());
    assert_eq!(Some(1), target.distance(4, 8).unwrap());
    assert_eq!(Some(2), target.distance(4, 11).unwrap());
    assert_eq!(Some(2), target.distance(6, 12).unwrap());
    assert_eq!(Some(3), target.distance(2, 12).unwrap());
    assert_eq!(Some(3), target.distance(3, 12).unwrap());
}

#[test]
fn test_is_connected() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    assert_eq!(
        false,
        target.is_connected(7, 7, 0, Bound::Unbounded).unwrap()
    );
    assert_eq!(
        false,
        target.is_connected(12, 1, 0, Bound::Unbounded).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(0, 6, 1, Bound::Included(1)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(3, 7, 1, Bound::Excluded(2)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(4, 8, 1, Bound::Unbounded).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(4, 11, 2, Bound::Excluded(4)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(6, 12, 1, Bound::Included(2)).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(2, 12, 3, Bound::Unbounded).unwrap()
    );
    assert_eq!(
        true,
        target.is_connected(3, 12, 3, Bound::Included(3)).unwrap()
    );
}

#[test]
fn test_save_load() {
    // Create an example graph storage to copy the value from
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut save_gs = PathStorage::new().unwrap();
    save_gs.copy(&node_annos, &orig).unwrap();

    let tmp_location = tempfile::TempDir::new().unwrap();
    save_gs.save_to(tmp_location.path()).unwrap();

    let new_gs = PathStorage::load_from(tmp_location.path()).unwrap();

    let result: Result<Vec<_>> = new_gs.source_nodes().collect();
    let mut result = result.unwrap();
    result.sort();

    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], result);

    for source in 9..=11 {
        let edge_anno = new_gs
            .get_anno_storage()
            .get_annotations_for_item(&(source, 12).into())
            .unwrap();
        assert_eq!(1, edge_anno.len());
        assert_eq!("default_ns", edge_anno[0].key.ns);
        assert_eq!("example", edge_anno[0].key.name);
        assert_eq!("last", edge_anno[0].val);
    }
}

#[test]
fn test_has_ingoing_edges() {
    let node_annos = AnnoStorageImpl::new();
    let orig = create_topdown_gs().unwrap();
    let mut target = PathStorage::new().unwrap();
    target.copy(&node_annos, &orig).unwrap();

    // Test first layer
    for n in 0..=5 {
        assert_eq!(false, target.has_ingoing_edges(n).unwrap());
    }
    // Test all other nodes
    for n in 6..=12 {
        assert_eq!(true, target.has_ingoing_edges(n).unwrap());
    }
    // Test some non-existing nodes
    assert_eq!(false, target.has_ingoing_edges(123).unwrap());
    assert_eq!(false, target.has_ingoing_edges(2048).unwrap());
}
