use crate::errors::Result;
use crate::graph::storage::adjacencylist::AdjacencyListStorage;
use crate::graph::storage::WriteableGraphStorage;
use crate::types::{AnnoKey, Annotation, Edge};

/// Creates an example graph storage with the folllowing structure:
///
/// ```plain
/// +---+
/// | 1 | -+
/// +---+  |
///     |  |
///     |  |
///     v  |
/// +---+  |
/// | 2 |  |
/// +---+  |
///     |  |
///     |  |
///     v  |
/// +---+  |
/// | 3 | <+
/// +---+
///     |
///     |
///     v
/// +---+
/// | 4 |
/// +---+
///     |
///     |
///     v
/// +---+
/// | 5 |
/// +---+
/// ```
pub(crate) fn create_multiple_paths_dag() -> Result<AdjacencyListStorage> {
    let mut gs = AdjacencyListStorage::new();

    gs.add_edge(Edge {
        source: 1,
        target: 2,
    })?;
    gs.add_edge(Edge {
        source: 2,
        target: 3,
    })?;
    gs.add_edge(Edge {
        source: 3,
        target: 4,
    })?;
    gs.add_edge(Edge {
        source: 1,
        target: 3,
    })?;
    gs.add_edge(Edge {
        source: 4,
        target: 5,
    })?;

    Ok(gs)
}

/// Creates an example graph storage with the folllowing structure:
///
/// ```plain
///  +---+     +---+     +---+     +---+
///  | 7 | <-- | 5 | <-- | 3 | <-- | 1 |
///  +---+     +---+     +---+     +---+
///              |         |         |
///              |         |         |
///              v         |         v
///            +---+       |       +---+
///            | 6 |       |       | 2 |
///            +---+       |       +---+
///                        |         |
///                        |         |
///                        |         v
///                        |       +---+
///                        +-----> | 4 |
///                                +---+
/// ```
pub(crate) fn create_simple_dag() -> Result<AdjacencyListStorage> {
    let mut gs = AdjacencyListStorage::new();

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

    Ok(gs)
}

/// Creates an example graph storage with the folllowing structure:
///
/// ```plain
///  0 -> 1 -> 2 -> 3 -> 4
///  5 -> 6 -> 7 -> 8
///  9 -> 10
/// ```
pub(crate) fn create_linear_gs() -> Result<AdjacencyListStorage> {
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

/// Creates an example graph storage with the folllowing structure:
///
/// ```plain
///           0
///          / \
///         1   2
///        /     \
///       3       4
///      / \     / \
///     5   6   7   8
/// ```
pub(crate) fn create_tree_gs() -> Result<AdjacencyListStorage> {
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
