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
    assert_eq!(true, dfs.is_cyclic());
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
    assert_eq!(true, dfs.is_cyclic());
}

#[test]
fn dfs_with_cycle_insubgraph() {
    // Create a graph with 4 nodes and a cycle
    // 0 -> 1 -> 2 -> 3
    //      ^       /
    //      |      /
    //       +---+
    let mut gs = AdjacencyListStorage::new();
    gs.add_edge((0, 1).into()).unwrap();
    gs.add_edge((1, 2).into()).unwrap();
    gs.add_edge((2, 3).into()).unwrap();
    gs.add_edge((3, 1).into()).unwrap();

    // Iterate over the graph storage, each node should only be in the output
    // once
    let mut dfs = CycleSafeDFS::new(&gs, 0, 1, usize::MAX);

    let step1 = dfs.next().unwrap().unwrap();
    assert_eq!(1, step1.node);

    let step2 = dfs.next().unwrap().unwrap();
    assert_eq!(2, step2.node);

    let step3 = dfs.next().unwrap().unwrap();
    assert_eq!(3, step3.node);

    assert_eq!(true, dfs.next().is_none());
    assert_eq!(true, dfs.is_cyclic());
}

#[test]
fn dfs_with_double_cycle() {
    // Create a graph with two cycles
    //
    //        +----+
    //       /      \
    //      /        v
    // 0 -> 1 -> 2 -> 3 -> 4
    //      ^        /
    //      |       /
    //       +-----+
    let mut gs = AdjacencyListStorage::new();
    gs.add_edge((0, 1).into()).unwrap();
    gs.add_edge((1, 2).into()).unwrap();
    gs.add_edge((2, 3).into()).unwrap();
    gs.add_edge((3, 4).into()).unwrap();
    // Add the edges that create the cycle
    gs.add_edge((3, 1).into()).unwrap();
    gs.add_edge((1, 3).into()).unwrap();

    // Iterate over the graph storage and collect the steps
    let mut dfs = CycleSafeDFS::new(&gs, 0, 1, usize::MAX);
    let mut result = Vec::default();
    for step in &mut dfs {
        let step = step.unwrap();
        result.push((step.node, step.distance));
    }
    assert_eq!(true, dfs.is_cyclic());
    result.sort();
    assert_eq!(result, vec![(1, 1), (2, 2), (3, 2), (3, 3), (4, 3), (4, 4)]);
}
