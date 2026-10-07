use crate::graph::storage::{
    WriteableGraphStorage, adjacencylist::AdjacencyListStorage, union::UnionEdgeContainer,
};

use super::*;

#[test]
fn dfs_with_cycle() {
    // Create a very simple graph with three nodes and a cycle
    let mut gs = AdjacencyListStorage::new();
    gs.add_edge((0, 1).into()).unwrap();
    gs.add_edge((1, 2).into()).unwrap();
    gs.add_edge((2, 0).into()).unwrap();

    // Iterate over the graph storage, each node should only be output once
    let mut dfs = CycleSafeDFS::new(&gs, 0, 1, usize::MAX);
    let step1 = dfs.next().unwrap().unwrap();
    assert_eq!(1, step1.node);

    let step2 = dfs.next().unwrap().unwrap();
    assert_eq!(2, step2.node);

    assert_eq!(true, dfs.next().is_none());
}

#[test]
fn dfs_with_cycle_in_unionedgecontainer() {
    // Create a very simple graph with three nodes and a cycle.
    // Distribute the edges on three seperate graph storages
    let mut gs1 = AdjacencyListStorage::new();
    gs1.add_edge((0, 1).into()).unwrap();

    let mut gs2 = AdjacencyListStorage::new();
    gs2.add_edge((1, 2).into()).unwrap();

    let mut gs3 = AdjacencyListStorage::new();
    gs3.add_edge((2, 0).into()).unwrap();

    // Create an union edge container over these 3 graph storages
    let gs = UnionEdgeContainer::new(vec![&gs1, &gs2, &gs3]);

    // Iterate over the graph storage, each node should only be output once
    let mut dfs = CycleSafeDFS::new(&gs, 0, 1, usize::MAX);
    let step1 = dfs.next().unwrap().unwrap();
    assert_eq!(1, step1.node);

    let step2 = dfs.next().unwrap().unwrap();
    assert_eq!(2, step2.node);

    assert_eq!(true, dfs.next().is_none());
}
