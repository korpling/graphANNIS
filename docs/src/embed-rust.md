# Rust Tutorial

## Installation

Add a dependency to graphANNIS in you `Cargo.toml` file:
```toml
graphannis = "0.24.0"
```

## API documentation

The API documentation is available at [https://docs.rs/graphannis/](https://docs.rs/graphannis/).

## Corpus data directory

Data is organized in corpora, where each corpus has a name and annotations can only refer to other annotations in the same corpus.
A `CorpusStorage` is used to access a collection corpora by their name.
```rust,noplaypen,ignore
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let corpora = cs.list().unwrap();
    let corpus_names : Vec<String> = corpora.into_iter().map(|corpus_info| corpus_info.name).collect();
    println!("{:?}", corpus_names);
}
```
This will print an empty list, because no corpora have been created yet.
In this example, the `CorpusStorage` uses the sub-directory `data` of the current working directory to store the corpora.
You can also use an absolute path as argument
```rust,noplaypen,ignore
let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("/tmp/graphannis-data"), true)?;
```
Only one process can access a graphANNIS data directory, other processes will fail to open it if there is another process holding a lock.
The `CorpusStorage` is thread-safe, thus multiple threads of the same process can call all functions in parallel.

## Adding corpus data

Linguistic annotations as represented in graphANNIS as directed graphs (see the [data model](annotation-graph.md) section for more information).
You can add nodes and edges via the `apply_update(...)` function.
It takes the corpus name and a list of graph updates as argument.
These graph update lists are represented by the class `GraphUpdate`.
E.g the following code creates a graph update for the tokenized sentence "That is a Category 3 storm.".
Normally, you would not add all events manually in the source code, which gets a bit verbose, but have input data that you map to update events.
The resulting `GraphUpdate` object can then be used with the `apply_update(...)` function to insert the changes into the corpus.

```rust,noplaypen,ignore
use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();

    let mut g = GraphUpdate::new();

    // First add the node (with the default type "node"),
    // then all node labels for the node.
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t1".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t1".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "That".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t2".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t2".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "is".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t3".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t3".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "a".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t4".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t4".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "Category".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t5".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t5".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "3".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t6".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t6".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: "storm".to_string(),
    });

    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1#t7".to_string(),
        node_type: "node".to_string(),
    });
    g.add_event(UpdateEvent::AddNodeLabel {
        node_name: "tutorial/doc1#t7".to_string(),
        anno_ns: "annis".to_string(),
        anno_name: "tok".to_string(),
        anno_value: ".".to_string(),
    });

    // Add the ordering edges to specify token order.
    // The names of the source and target nodes are given as in the enum as fields,
    // followed by the component layer, type and name.
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t1".to_string(),
        target_node: "tutorial/doc1#t2".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t2".to_string(),
        target_node: "tutorial/doc1#t3".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t3".to_string(),
        target_node: "tutorial/doc1#t4".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t4".to_string(),
        target_node: "tutorial/doc1#t5".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t5".to_string(),
        target_node: "tutorial/doc1#t6".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t6".to_string(),
        target_node: "tutorial/doc1#t7".to_string(),
        layer: "annis".to_string(),
        component_type: "Ordering".to_string(),
        component_name: "".to_string(),
    });

    // Insert the changes in the corpus with the name "tutorial"
    cs.apply_update("tutorial", &mut g).unwrap();

    // List newly created corpus
    let corpora = cs.list().unwrap();
    let corpus_names: Vec<String> = corpora
        .into_iter()
        .map(|corpus_info| corpus_info.name)
        .collect();
    println!("{:?}", corpus_names);
}
```
You could add additional annotations like part of speech as labels on nodes.
For labels on edges, you can use the `UpdateEvent::AddEdgeLabel` enum variant.


## Querying 

There are two functions to query a corpus with AQL:
- `count(...)` returns the number of matches, and
- `find(...)` returns a paginated list of matched node IDs.

You have to give the list of corpora and the query as arguments to both functions.
The following example searches for all tokens that contain a `s` character.[^aql]
```rust,noplaypen,ignore
use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let number_of_matches = cs
        .count("tutorial", "tok=/.*s.*/", QueryLanguage::AQL)
        .unwrap();
    println!("Number of matches: {}", number_of_matches);

    let matches = cs
        .find(
            "tutorial",
            "tok=/.*s.*/",
            QueryLanguage::AQL,
            0,
            100,
            ResultOrder::Normal,
        )
        .unwrap();
    for i in 0..matches.len() {
        println!("Match {}: {}", i, matches[i]);
    }
}
```
Output:
```ignore
Number of matches: 2
Match 0: salt:/tutorial/doc1#t2
Match 1: salt:/tutorial/doc1#t6
```
GraphANNIS will add the URI scheme `salt:/` to your node names automatically.

## Getting subgraphs

The result from the `find(...)` function can be used to generate a subgraph for the matches.
It will contain all covered nodes of the matches and additionally a given context (defined in tokens).
```rust,noplaypen,ignore
use graphannis::corpusstorage::{QueryLanguage, ResultOrder};
use graphannis::graph::AnnotationStorage;
use graphannis::util;
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let matches = cs
        .find(
            "tutorial",
            "tok . tok",
            QueryLanguage::AQL,
            0,
            100,
            ResultOrder::Normal,
        )
        .unwrap();
    for m in matches {
        println!("{}", m);
        // convert the match string to a list of node IDs
        let node_names = util::node_names_from_match(&m);
        let g = cs.subgraph("tutorial", node_names, 2, 2).unwrap();
        // find all nodes of type "node" (regular annotation nodes)
        let node_search = g.exact_anno_search(
            Some("annis".to_string()),
            "node_type".to_string(),
            Some("node".to_string()).into(),
        );
        println!("Number of nodes in subgraph: {}", node_search.count());
    }
}
```
Output:
```ignore
salt:/tutorial/doc1#t1 salt:/tutorial/doc1#t2
Number of nodes in subgraph: 4
salt:/tutorial/doc1#t2 salt:/tutorial/doc1#t3
Number of nodes in subgraph: 5
salt:/tutorial/doc1#t3 salt:/tutorial/doc1#t4
Number of nodes in subgraph: 6
salt:/tutorial/doc1#t4 salt:/tutorial/doc1#t5
Number of nodes in subgraph: 6
salt:/tutorial/doc1#t5 salt:/tutorial/doc1#t6
Number of nodes in subgraph: 5
salt:/tutorial/doc1#t6 salt:/tutorial/doc1#t7
Number of nodes in subgraph: 4
```
The result object of the `subgraph(...)` function is the type `Graph`, which provides basic graph access functions (see the API documentation for details).

**Note:** The `subgraph(...)` function takes a single corpus name as argument instead of a list, so you need to know to which corpus a matched node belongs to.

Normally a corpus is structured into subcorpora and documents.
GraphANNIS uses node types and relations of type `PartOf` to [model the corpus structure](../data-model/annotation-graph.md#corpus-structure).
If you have document nodes and the `PartOf` relation between the annotation nodes and its document, you can use the
`subcorpus_graph(...)` function to get all annotation nodes for a given list of document names.

```rust,noplaypen,ignore
use graphannis::graph::AnnotationStorage;
use graphannis::update::{GraphUpdate, UpdateEvent};
use graphannis::CorpusStorage;
use std::path::PathBuf;

fn main() {
    let cs = CorpusStorage::with_auto_cache_size(&PathBuf::from("data"), true).unwrap();
    let mut g = GraphUpdate::new();
    // create the corpus and document node
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial".to_string(),
        node_type: "corpus".to_string(),
    });
    g.add_event(UpdateEvent::AddNode {
        node_name: "tutorial/doc1".to_string(),
        node_type: "corpus".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1".to_string(),
        target_node: "tutorial".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    // add the corpus structure to the existing nodes
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t1".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t2".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t3".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t4".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t5".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t6".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    g.add_event(UpdateEvent::AddEdge {
        source_node: "tutorial/doc1#t7".to_string(),
        target_node: "tutorial/doc1".to_string(),
        layer: "annis".to_string(),
        component_type: "PartOf".to_string(),
        component_name: "".to_string(),
    });
    // apply the changes
    cs.apply_update("tutorial", &mut g).unwrap();

    // get the whole document as graph
    let subgraph = cs
        .subcorpus_graph("tutorial", vec!["tutorial/doc1".to_string()])
        .unwrap();
    let node_search = subgraph.exact_anno_search(
        Some("annis".to_string()),
        "node_type".to_string(),
        Some("node".to_string()).into(),
    );
    for m in node_search {
        // get the numeric node ID from the match
        let id = m.get_node();
        // get the node name from the ID by searching for the label with the name "annis::node_name"
        let matched_node_name = subgraph
            .get_annotations_for_item(&id)
            .into_iter()
            .filter(|anno| anno.key.ns == "annis" && anno.key.name == "node_name")
            .map(|anno| anno.val)
            .next()
            .unwrap();
        println!("{}", matched_node_name);
    }
}
```
Output:
```ignore
tutorial/doc1#t5
tutorial/doc1#t7
tutorial/doc1#t1
tutorial/doc1#t6
tutorial/doc1#t4
tutorial/doc1#t2
tutorial/doc1#t3
```

[^aql]: You can get an overview of AQL [here](http://corpus-tools.org/annis/aql.html) or detailled information in the
[User Guide](http://korpling.github.io/ANNIS/3.6/user-guide/aql.html).
