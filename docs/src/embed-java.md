# Java Tutorial

## Installation

GraphANNIS works with applications written for Java 8 or later.
If you are using Apache Maven as your build system, you can add a dependency to graphANNIS with

```xml
<dependency>
  <groupId>org.corpus-tools</groupId>
  <artifactId>graphannis-api</artifactId>
  <version>0.20.0</version>
</dependency>
```

## API documentation

The API documentation is available at 
[http://www.javadoc.io/doc/org.corpus-tools/graphannis-api/](http://www.javadoc.io/doc/org.corpus-tools/graphannis-api/).

## Corpus data directory

Data is organized in corpora, where each corpus has a name and annotations can only refer to other annotations in the same corpus.
A `CorpusStorageManager` is used to access a collection corpora by their name.
```java
package org.corpus_tools;

import org.corpus_tools.graphannis.CorpusStorageManager;
import org.corpus_tools.graphannis.errors.GraphANNISException;

public class ListCorpora {
    public static void main(String[] args) throws GraphANNISException {
        CorpusStorageManager cs = new CorpusStorageManager("data");
        String[] corpora = cs.list();
        System.out.println(corpora.length);
    }
}
```
This will print an `0`, because no corpora have been created yet.
In this example, the `CorpusStorageManager` uses the sub-directory `data` of the current working directory to store the corpora.
You can also use an absolute path as argument
```java
CorpusStorageManager cs = new CorpusStorageManager("/tmp/graphannis-data");
```
Only one process can access a graphANNIS data directory, other processes will fail to open it if there is another process holding a lock.
The `CorpusStorageManager` is thread-safe, thus multiple threads of the same process can call all functions in parallel.

## Adding corpus data

Linguistic annotations as represented in graphANNIS as directed graphs (see the [data model](./annotation-graph.md) section for more information).
You can add nodes and edges via the `applyUpdate(...)` function.
It takes the corpus name and a list of graph updates as argument.
These graph update lists are represented by the class `GraphUpdate`.
E.g the following code creates a graph update for the tokenized sentence "That is a Category 3 storm.".
Normally, you would not add all events manually in the source code, which gets a bit verbose, but have input data that you map to update events.
The resulting `GraphUpdate` object can then be used with the `applyUpdate(...)` function to insert the changes into the corpus.

```java
package org.corpus_tools;

import org.corpus_tools.graphannis.CorpusStorageManager;
import org.corpus_tools.graphannis.GraphUpdate;
import org.corpus_tools.graphannis.errors.GraphANNISException;

public class ApplyUpdate {
    public static void main(String[] args) throws GraphANNISException {
        CorpusStorageManager cs = new CorpusStorageManager("data");

        GraphUpdate g = new GraphUpdate();

        // First argument is the node name.
        g.addNode("tutorial/doc1#t1");
        // First argument is the node name, 
        // then comes the annotation namespace, name and value.
        g.addNodeLabel("tutorial/doc1#t1", "annis", "tok", "That");

        g.addNode("tutorial/doc1#t2");
        g.addNodeLabel("tutorial/doc1#t2", "annis", "tok", "is");

        g.addNode("tutorial/doc1#t3");
        g.addNodeLabel("tutorial/doc1#t3", "annis", "tok", "a");

        g.addNode("tutorial/doc1#t4");
        g.addNodeLabel("tutorial/doc1#t4", "annis", "tok", "Category");

        g.addNode("tutorial/doc1#t5");
        g.addNodeLabel("tutorial/doc1#t5", "annis", "tok", "3");

        g.addNode("tutorial/doc1#t6");
        g.addNodeLabel("tutorial/doc1#t6", "annis", "tok", "storm");

        g.addNode("tutorial/doc1#t7");
        g.addNodeLabel("tutorial/doc1#t7", "annis", "tok", ".");

        // Add the ordering edges to specify token order.
        // The names of the source and target nodes are given as arguments, 
        // followed by the component layer, type and name.
        g.addEdge("tutorial/doc1#t1", "tutorial/doc1#t2", "annis", "Ordering", "");
        g.addEdge("tutorial/doc1#t2", "tutorial/doc1#t3", "annis", "Ordering", "");
        g.addEdge("tutorial/doc1#t3", "tutorial/doc1#t4", "annis", "Ordering", "");
        g.addEdge("tutorial/doc1#t4", "tutorial/doc1#t5", "annis", "Ordering", "");
        g.addEdge("tutorial/doc1#t5", "tutorial/doc1#t6", "annis", "Ordering", "");
        g.addEdge("tutorial/doc1#t6", "tutorial/doc1#t7", "annis", "Ordering", "");

        cs.applyUpdate("tutorial", g);
        String[] corpora = cs.list();
        if(corpora.length > 0) {
            System.out.println(corpora[0]);
        } else {
            System.out.println("No corpus found");
        }
        
    }
}
```
You could add additional annotations like part of speech as labels on nodes.
For labels on edges, you can use the `addEdgeLabel(...)` function.


## Querying 

There are two functions to query a corpus with AQL:
- `count(...)` returns the number of matches, and
- `find(...)` returns a paginated list of matched node IDs.

You have to give the list of corpora and the query as arguments to both functions.
The following example searches for all tokens that contain a `s` character.[^aql]
```java
package org.corpus_tools;

import java.util.Arrays;

import org.corpus_tools.graphannis.CorpusStorageManager;
import org.corpus_tools.graphannis.errors.GraphANNISException;

public class Query {
    public static void main(String[] args) throws GraphANNISException {
        CorpusStorageManager cs = new CorpusStorageManager("data");
        long number_of_matches = cs.count(Arrays.asList("tutorial"), "tok=/.*s.*/");
        System.out.println("Number of matches: " + number_of_matches);
        String[] matches = cs.find(Arrays.asList("tutorial"), "tok=/.*s.*/", 0, 100);
        for (int i = 0; i < matches.length; i++) {
            System.out.println("Match " + i + ": " + matches[i]);
        }
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
```java
package org.corpus_tools;

import java.util.Arrays;
import java.util.List;

import org.corpus_tools.graphannis.CorpusStorageManager;
import org.corpus_tools.graphannis.Util;
import org.corpus_tools.graphannis.errors.GraphANNISException;
import org.corpus_tools.graphannis.model.Graph;
import org.corpus_tools.graphannis.model.Node;

public class FindSubgraph {
    public static void main(String[] args) throws GraphANNISException {
        CorpusStorageManager cs = new CorpusStorageManager("data");
        String[] matches = cs.find(Arrays.asList("tutorial"), "tok . tok", 0, 100);
        for (String m : matches) {
            System.out.println(m);
            // convert the match string to a list of node IDs
            List<String> node_names = Util.nodeNamesFromMatch(m);
            Graph g = cs.subgraph("tutorial", node_names, 2, 2);
            // iterate over all nodes of type "node" and output the name
            int numberOfNodes = 0;
            for (Node n : g.getNodesByType("node")) {
                numberOfNodes++;
            }
            System.out.println("Number of nodes in subgraph: " + numberOfNodes);
        }
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
The result object of the `subgraph(...)` function is the type `Graph`, which provides basic graph access functions (see the JavaDoc for details).

**Note:** The `subgraph(...)` function takes a single corpus name as argument instead of a list, so you need to know to which corpus a matched node belongs to.

Normally a corpus is structured into subcorpora and documents.
GraphANNIS uses node types and relations of type `PartOf` to [model the corpus structure](annotation-graph.md#corpus-structure).
If you have document nodes and the `PartOf` relation between the annotation nodes and its document, you can use the
`subcorpus_graph(...)` function to get all annotation nodes for a given list of document names.

```java
package org.corpus_tools;

import java.util.Arrays;

import org.corpus_tools.graphannis.CorpusStorageManager;
import org.corpus_tools.graphannis.GraphUpdate;
import org.corpus_tools.graphannis.errors.GraphANNISException;
import org.corpus_tools.graphannis.model.Graph;
import org.corpus_tools.graphannis.model.Node;

public class SubcorpusGraph {
    public static void main(String[] args) throws GraphANNISException {
        CorpusStorageManager cs = new CorpusStorageManager("data");
        GraphUpdate g = new GraphUpdate();
        // create the corpus and document node
        g.addNode("tutorial", "corpus");
        g.addNode("tutorial/doc1", "corpus");
        g.addEdge("tutorial/doc1", "tutorial", "annis", "PartOf", "");
        // add the corpus structure to the existing nodes
        g.addEdge("tutorial/doc1#t1", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t2", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t3", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t4", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t5", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t6", "tutorial/doc1", "annis", "PartOf", "");
        g.addEdge("tutorial/doc1#t7", "tutorial/doc1", "annis", "PartOf", "");
        // apply the changes
        cs.applyUpdate("tutorial", g);
        // get the whole document as graph
        Graph subgraph = cs.subcorpusGraph("tutorial", Arrays.asList("tutorial/doc1"));
        for (Node n : subgraph.getNodesByType("node")) {
            System.out.println(n.getName());
        }
    }
}

```
Output:
```ignore
tutorial/doc1#t1
tutorial/doc1#t2
tutorial/doc1#t3
tutorial/doc1#t4
tutorial/doc1#t5
tutorial/doc1#t6
tutorial/doc1#t7
```

[^aql]: You can get an overview of AQL [here](http://corpus-tools.org/annis/aql.html) or detailled information in the
[User Guide](http://korpling.github.io/ANNIS/3.6/user-guide/aql.html).