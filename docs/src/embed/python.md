# Python Tutorial

## Installation

GraphANNIS works with applications written for Python 3.6 or later.
You can download the graphANNIS library with pip
```bash
pip install graphannis
```
In some environments (e.g. Ubuntu Linux), you have to use `pip3` instead to select Python 3.
```bash
pip3 install graphannis
``` 

## API documentation

The API documentation is available at [http://graphannis-python.readthedocs.io/](http://graphannis-python.readthedocs.io/).

## Corpus data directory

Data is organized in corpora, where each corpus has a name and annotations can only refer to other annotations in the same corpus.
A `CorpusStorageManager` is used to access a collection corpora by their name.
```python
from graphannis.cs import CorpusStorageManager
with CorpusStorageManager() as cs:
    print(cs.list())
```
This will print an empty list, because no corpora have been created yet.
Per default, the `CorpusStorageManager` uses the sub-directory `data` of the current working directory to store the corpora.
You can change the location with the `db_dir` parameter
```python
from graphannis.cs import CorpusStorageManager
with CorpusStorageManager(db_dir='/tmp/graphannis-data') as cs:
    print(cs.list())
```
Only one process can access a graphANNIS data directory, other processes will fail to open it if there is another process holding a lock.
The `CorpusStorageManager` is thread-safe, thus multiple threads of the same process can call all functions in parallel.

## Adding corpus data

Linguistic annotations as represented in graphANNIS as directed graphs (see the [data model](../data-model/annotation-graph.md) section for more information).
You can add nodes and edges via the `apply_update(...)` function.
It takes the corpus name and a list of graph updates as argument.
These graph update lists are represented by the class `graphannis.graph.GraphUpdate`.
E.g the following code creates a graph update for the tokenized sentence "That is a Category 3 storm."
Normally, you would not add all events manually in the source code, which gets a bit verbose, but have input data that you map to update events.

```python
from graphannis.graph import GraphUpdate
g = GraphUpdate()

# First argument is the node name.
g.add_node("tutorial/doc1#t1")
# First argument is the node name, 
# then comes the annotation namespace, name and value.
g.add_node_label("tutorial/doc1#t1", "annis", "tok", "That")

g.add_node("tutorial/doc1#t2")
g.add_node_label("tutorial/doc1#t2", "annis", "tok", "is")

g.add_node("tutorial/doc1#t3")
g.add_node_label("tutorial/doc1#t3", "annis", "tok", "a")

g.add_node("tutorial/doc1#t4")
g.add_node_label("tutorial/doc1#t4", "annis", "tok", "Category")

g.add_node("tutorial/doc1#t5")
g.add_node_label("tutorial/doc1#t5", "annis", "tok", "3")

g.add_node("tutorial/doc1#t6")
g.add_node_label("tutorial/doc1#t6", "annis", "tok", "storm")

g.add_node("tutorial/doc1#t7")
g.add_node_label("tutorial/doc1#t7", "annis", "tok", ".")

# Add the ordering edges to specify token order.
# The names of the source and target nodes are given as arguments, 
# followed by the component layer, type and name.
g.add_edge("tutorial/doc1#t1", "tutorial/doc1#t2", "annis", "Ordering", "")
g.add_edge("tutorial/doc1#t2", "tutorial/doc1#t3", "annis", "Ordering", "")
g.add_edge("tutorial/doc1#t3", "tutorial/doc1#t4", "annis", "Ordering", "")
g.add_edge("tutorial/doc1#t4", "tutorial/doc1#t5", "annis", "Ordering", "")
g.add_edge("tutorial/doc1#t5", "tutorial/doc1#t6", "annis", "Ordering", "")
g.add_edge("tutorial/doc1#t6", "tutorial/doc1#t7", "annis", "Ordering", "")
```
You could add additional annotations like part of speech as labels on nodes.
For labels on edges, you can use the `add_edge_label(...)` function.

This `GraphUpdate` object can then be used with the `applyUpdate(...)` function:
```python
with CorpusStorageManager() as cs:
    cs.apply_update("tutorial", g)
    # this now includes the "tutorial"
    print(cs.list())
```

## Querying 

There are two functions to query a corpus with AQL:
- `count(...)` returns the number of matches, and
- `find(...)` returns a paginated list of matched node IDs.

You have to give the corpus name and the query as arguments to both functions.
The following example searches for all tokens that contain a `s` character.[^aql]
```python
with CorpusStorageManager() as cs: 
    number_of_matches = cs.count("tutorial", 'tok=/.*s.*/')
    print(number_of_matches)
    matches = cs.find("tutorial", 'tok=/.*s.*/', offset=0, limit=100)
    print(matches)
```
Output:
```ignore
2
[['salt:/tutorial/doc1#t2'], ['salt:/tutorial/doc1#t6']]
```
GraphANNIS will add the URI scheme `salt:/` to your node names automatically.

## Getting subgraphs

The result from the `find(...)` function can be used to generate a subgraph for the matches.
It will contain all covered nodes of the matches and additionally a given context (defined in tokens).
```python
from graphannis.util import node_name_from_match
with CorpusStorageManager() as cs: 
    matches = cs.find("tutorial", 'tok . tok', offset=0, limit=100)
    for m in matches:
        print(m)
        G = cs.subgraph("tutorial", node_name_from_match(m), ctx_left=2, ctx_right=2)
        print("Number of nodes in subgraph: " + str(len(G.nodes)))
```
Output:
```ignore
['salt:/tutorial/doc1#t1', 'salt:/tutorial/doc1#t2']
Number of nodes in subgraph: 4
['salt:/tutorial/doc1#t2', 'salt:/tutorial/doc1#t3']
Number of nodes in subgraph: 5
['salt:/tutorial/doc1#t3', 'salt:/tutorial/doc1#t4']
Number of nodes in subgraph: 6
['salt:/tutorial/doc1#t4', 'salt:/tutorial/doc1#t5']
Number of nodes in subgraph: 6
['salt:/tutorial/doc1#t5', 'salt:/tutorial/doc1#t6']
Number of nodes in subgraph: 5
['salt:/tutorial/doc1#t6', 'salt:/tutorial/doc1#t7']
Number of nodes in subgraph: 4

```
The result object of the `subgraph(...)` function is the type [NetworkX MultiDiGraph](https://networkx.github.io/documentation/stable/reference/classes/multidigraph.html).
[NetworkX](https://networkx.github.io/documentation/stable/tutorial.html) is a graph library, that provides basic graph access and manipulation functions, but also implements graph traversal and other graph algorithms.

**Note:** The `subgraph(...)` function takes a single corpus name as argument instead of a list, so you need to know to which corpus a matched node belongs to.

Normally a corpus is structured into subcorpora and documents.
GraphANNIS uses node types and relations of type `PartOf` to [model the corpus structure](../data-model/annotation-graph.md#corpus-structure).
If you have document nodes and the `PartOf` relation between the annotation nodes and its document, you can use the
`subcorpus_graph(...)` function to get all annotation nodes for a given list of document names.

```python
with CorpusStorageManager() as cs:
    g = GraphUpdate()
    # create the corpus and document node
    g.add_node('tutorial', node_type='corpus')
    g.add_node('tutorial/doc1', node_type='corpus')
    g.add_edge('tutorial/doc1','tutorial', 'annis', 'PartOf', '')
    # add the corpus structure to the existing nodes
    g.add_edge('tutorial/doc1#t1','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t2','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t3','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t4','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t5','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t6','tutorial/doc1', 'annis', 'PartOf', '')
    g.add_edge('tutorial/doc1#t7','tutorial/doc1', 'annis', 'PartOf', '')
    # apply the changes
    cs.apply_update('tutorial', g)
    # get the whole document as graph
    G = cs.subcorpus_graph('tutorial', ['tutorial/doc1'])
    print(G.nodes)
```
Output:
```ignore
['salt:/tutorial/doc1#t1', 'salt:/tutorial/doc1#t6', 'salt:/tutorial/doc1#t4', 'salt:/tutorial/doc1#t2', 'salt:/tutorial/doc1#t7', 'salt:/tutorial/doc1#t5', 'salt:/tutorial/doc1#t3']
```

[^aql]: You can get an overview of AQL [here](http://corpus-tools.org/annis/aql.html) or detailled information in the
[User Guide](http://korpling.github.io/ANNIS/3.6/user-guide/aql.html).
