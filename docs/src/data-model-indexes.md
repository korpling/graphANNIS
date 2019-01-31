# Indexes for faster query execution

GraphANNIS uses helper edges to speed-up AQL query execution.
These are automatically created or updated when a graph update is executed via the `apply_update(...)` function of the `CorpusStorage` structure.
Users of the API do not have to add these edges by their-self.

## Inherited coverage

Dominance relations imply text-coverage and thus any node connected inherits the covered token from its child nodes.
Instead of traversing the dominance edges on query time, inherited coverage relations are explicitly added to the nodes.

![Example of inherited coverage edges for a constituent tree](images/constituent-index.png)

## Left-most and right-most token of a node

These components of type `LeftToken` and `RightToken` allow providing faster implementations for some operators that deal with text coverage or precedence.
Their target node is the left-most or right-most covered token of a node.
The covered token are ordered by their position in the chain of `Ordering` relations.
If there is a path of any length from token \\({t_1}\\) to \\({t_2}\\), \\({t_1}\\) is more "left" than \\({t_2}\\).

![LeftToken and RightToken example for span](images/span-index.png)

While the coverage edges are similar to the `SSpanningRelation`, the left and right token edges are inspired from the two columns of the `node` table in relANNIS (the legacy relational database implementation of ANNIS) with the same name.
Each node of the annotation graph that is not a token must have a left and right token edge because AQL implicitly requires all nodes to be connected to tokens.
Lookup for all left- or right-aligned nodes of a token is possible by accesssing the inverse edges of the `LeftToken` and `RightToken` component.
