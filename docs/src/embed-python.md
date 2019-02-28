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

Linguistic annotations as represented in graphANNIS as directed graphs (see the [data model](./annotation-graph.md) section for more information).
You can add nodes and edges via the `apply_update(...)` function.
It takes the corpus name and a list of graph updates as argument.
These graph update lists are represented by the class `graphannis.graph.GraphUpdate`.
E.g the following code creates a graph update for the tokenized sentence "That is a Category 3 storm."

```python
from graphannis.graph import GraphUpdate
g = GraphUpdate()

# First argument is the node name.
g.add_node("t1")
# First argument is the node name, 
# then comes the annotation namespace, name and value.
g.add_node_label("t1", "annis", "tok", "That")

g.add_node("t2")
g.add_node_label("t2", "annis", "tok", "is")

g.add_node("t3")
g.add_node_label("t3", "annis", "tok", "a")

g.add_node("t3")
g.add_node_label("t3", "annis", "tok", "Category")

g.add_node("t4")
g.add_node_label("t4", "annis", "tok", "3")

g.add_node("t5")
g.add_node_label("t5", "annis", "tok", "storm")

g.add_node("t6")
g.add_node_label("t6", "annis", "tok", ".")

# Add the ordering edges to specify token order.
# The names of the source and target nodes are given as arguments, 
# followed by the component layer, type and name.
g.add_edge("t1", "t2", "", "Ordering", "")
g.add_edge("t2", "t3", "", "Ordering", "")
g.add_edge("t3", "t4", "", "Ordering", "")
g.add_edge("t4", "t5", "", "Ordering", "")
g.add_edge("t5", "t6", "", "Ordering", "")
```
This `GraphUpdate` object can then be used with the `applyUpdate(...)` function:
```python
with CorpusStorageManager() as cs:
    cs.apply_update("tutorialCorpus", g)
    # this now includes the "tutorialCorpus"
    print(cs.list())
```

## Querying 

TODO

## Getting subgraphs

TODO
