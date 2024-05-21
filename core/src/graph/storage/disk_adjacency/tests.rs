use super::*;

#[test]
fn multiple_paths_find_range() {
    /*
    +---+
    | 1 | -+
    +---+  |
        |  |
        |  |
        v  |
    +---+  |
    | 2 |  |
    +---+  |
        |  |
        |  |
        v  |
    +---+  |
    | 3 | <+
    +---+
        |
        |
        v
    +---+
    | 4 |
    +---+
        |
        |
        v
    +---+
    | 5 |
    +---+
    */

    let mut gs = DiskAdjacencyListStorage::new().unwrap();
    gs.add_edge(Edge {
        source: 1,
        target: 2,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 2,
        target: 3,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 3,
        target: 4,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1,
        target: 3,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 4,
        target: 5,
    })
    .unwrap();

    let found: Result<Vec<NodeID>> = gs
        .find_connected(1, 3, std::ops::Bound::Included(3))
        .collect();
    let mut found = found.unwrap();
    assert_eq!(2, found.len());
    found.sort_unstable();

    assert_eq!(4, found[0]);
    assert_eq!(5, found[1]);

    let found: Result<Vec<NodeID>> = gs
        .find_connected(2, 1, std::ops::Bound::Excluded(3))
        .collect();
    let mut found = found.unwrap();

    assert_eq!(2, found.len());
    found.sort();
    assert_eq!(3, found[0]);
    assert_eq!(4, found[1]);
}

#[test]
fn simple_dag_find_all() {
    /*
    +---+     +---+     +---+     +---+
    | 7 | <-- | 5 | <-- | 3 | <-- | 1 |
    +---+     +---+     +---+     +---+
                |         |         |
                |         |         |
                v         |         v
              +---+       |       +---+
              | 6 |       |       | 2 |
              +---+       |       +---+
                          |         |
                          |         |
                          |         v
                          |       +---+
                          +-----> | 4 |
                                  +---+
    */
    let mut gs = DiskAdjacencyListStorage::new().unwrap();

    gs.add_edge(Edge {
        source: 1,
        target: 2,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 2,
        target: 4,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1,
        target: 3,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 3,
        target: 5,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 5,
        target: 7,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 5,
        target: 6,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 3,
        target: 4,
    })
    .unwrap();

    let mut out1 = gs
        .get_outgoing_edges(1)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    out1.sort_unstable();
    assert_eq!(vec![2, 3], out1);

    let mut out3 = gs
        .get_outgoing_edges(3)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    out3.sort_unstable();
    assert_eq!(vec![4, 5], out3);

    let out6 = gs
        .get_outgoing_edges(6)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(0, out6.len());

    let out2 = gs
        .get_outgoing_edges(2)
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(vec![4], out2);

    let reachable: Result<Vec<NodeID>> = gs.find_connected(1, 1, Bound::Included(100)).collect();
    let mut reachable = reachable.unwrap();
    reachable.sort_unstable();
    assert_eq!(vec![2, 3, 4, 5, 6, 7], reachable);

    let reachable: Result<Vec<NodeID>> = gs.find_connected(3, 2, Bound::Included(100)).collect();
    let mut reachable = reachable.unwrap();
    reachable.sort_unstable();
    assert_eq!(vec![6, 7], reachable);

    let reachable: Result<Vec<NodeID>> = gs.find_connected(1, 2, Bound::Included(4)).collect();
    let mut reachable = reachable.unwrap();
    reachable.sort_unstable();
    assert_eq!(vec![4, 5, 6, 7], reachable);

    let reachable: Result<Vec<NodeID>> = gs.find_connected(7, 1, Bound::Included(100)).collect();
    let reachable = reachable.unwrap();
    assert_eq!(true, reachable.is_empty());
}

#[test]
fn indirect_cycle_statistics() {
    let mut gs = DiskAdjacencyListStorage::new().unwrap();

    gs.add_edge(Edge {
        source: 1,
        target: 2,
    })
    .unwrap();

    gs.add_edge(Edge {
        source: 2,
        target: 3,
    })
    .unwrap();

    gs.add_edge(Edge {
        source: 3,
        target: 4,
    })
    .unwrap();

    gs.add_edge(Edge {
        source: 4,
        target: 5,
    })
    .unwrap();

    gs.add_edge(Edge {
        source: 5,
        target: 2,
    })
    .unwrap();

    gs.calculate_statistics().unwrap();
    assert_eq!(true, gs.get_statistics().is_some());
    let stats = gs.get_statistics().unwrap();
    assert_eq!(true, stats.cyclic);
}

#[test]
fn multi_branch_cycle_statistics() {
    let mut gs = DiskAdjacencyListStorage::new().unwrap();

    gs.add_edge(Edge {
        source: 903,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 904,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1174,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1295,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1310,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1334,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1335,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1336,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1337,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1338,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1339,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1340,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1341,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1342,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1343,
        target: 1343,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 903,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 904,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1174,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1295,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1310,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1334,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1335,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1336,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1337,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1338,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1339,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1340,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1341,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1342,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1343,
        target: 1342,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 903,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 904,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1174,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1295,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1310,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1334,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1335,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1336,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1337,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1338,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1339,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1340,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1341,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1342,
        target: 1339,
    })
    .unwrap();
    gs.add_edge(Edge {
        source: 1343,
        target: 1339,
    })
    .unwrap();

    gs.calculate_statistics().unwrap();
    assert_eq!(true, gs.get_statistics().is_some());
    let stats = gs.get_statistics().unwrap();
    assert_eq!(true, stats.cyclic);
}
