# Rust Tutorial

## Installation

Add a dependency to graphANNIS in you `Cargo.toml` file:
```toml
graphannis = "3.8.0"
```

## API documentation

The API documentation is available at [https://docs.rs/graphannis/](https://docs.rs/graphannis/).

## Corpus data directory

Data is organized in corpora, where each corpus has a name and annotations can only refer to other annotations in the same corpus.
A `CorpusStorage` is used to access a collection corpora by their name.

```rust,no_run,noplaypen
{{#include ../../examples/tutorial/src/bin/list.rs}}
```

This will print an empty list, because no corpora have been created yet.
In this example, the `CorpusStorage` uses the sub-directory `data` of the current working directory to store the corpora.
You can also use an absolute path as argument:
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
E.g the following code creates a graph update for the tokenized sentence “That is a Category 3 storm.”.
Normally, you would not add all events manually in the source code, which gets a bit verbose, but have input data that you map to update events.
The resulting `GraphUpdate` object can then be used with the `apply_update(...)` function to insert the changes into the corpus.

```rust,noplaypen
{{#include ../../examples/tutorial/src/bin/apply_update.rs}}
```

You could add additional annotations like part of speech as labels on nodes.
For labels on edges, you can use the `UpdateEvent::AddEdgeLabel` enumeration variant.


## Querying 

There are two functions to query a corpus with AQL:
- `count(...)` returns the number of matches, and
- `find(...)` returns a paginated list of matched node IDs.

You have to give the list of corpora and the query as arguments to both functions.
The following example searches for all tokens that contain a `s` character.[^aql]

```rust,no_run,noplaypen
{{#include ../../examples/tutorial/src/bin/query.rs}}
```
Output:
```ignore
Number of matches: 2
Match 0: tutorial/doc1#t2
Match 1: tutorial/doc1#t6
```

## Getting subgraphs

The result from the `find(...)` function can be used to generate a subgraph for the matches.
It will contain all covered nodes of the matches and additionally a given context (defined in tokens).

```rust,no_run,noplaypen
{{#include ../../examples/tutorial/src/bin/find_subgraph.rs}}
```
Output:
```ignore
tutorial/doc1#t1 tutorial/doc1#t2
Number of nodes in subgraph: 4
tutorial/doc1#t2 tutorial/doc1#t3
Number of nodes in subgraph: 5
tutorial/doc1#t3 tutorial/doc1#t4
Number of nodes in subgraph: 6
tutorial/doc1#t4 tutorial/doc1#t5
Number of nodes in subgraph: 6
tutorial/doc1#t5 tutorial/doc1#t6
Number of nodes in subgraph: 5
tutorial/doc1#t6 tutorial/doc1#t7
Number of nodes in subgraph: 4
```
The result object of the `subgraph(...)` function is the type `Graph`, which provides basic graph access functions (see the API documentation for details).

**Note:** The `subgraph(...)` function takes a single corpus name as argument instead of a list, so you need to know to which corpus a matched node belongs to.

Normally a corpus is structured into subcorpora and documents.
GraphANNIS uses node types and relations of type `PartOf` to [model the corpus structure](../data-model/annotation-graph.md#corpus-structure).
If you have document nodes and the `PartOf` relation between the annotation nodes and its document, you can use the
`subcorpus_graph(...)` function to get all annotation nodes for a given list of document names.

```rust,no_run,noplaypen
{{#include ../../examples/tutorial/src/bin/subcorpus_graph.rs}}
```
Output:
```ignore
tutorial/doc1#t2
tutorial/doc1#t4
tutorial/doc1#t5
tutorial/doc1#t6
tutorial/doc1#t7
tutorial/doc1#t1
tutorial/doc1#t3
```

[^aql]: You can get an overview of AQL [here](http://corpus-tools.org/annis/aql.html) or detailed information in the
[User Guide](http://korpling.github.io/ANNIS/3.6/user-guide/aql.html).
