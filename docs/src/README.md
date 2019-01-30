# Introduction

The graphANNIS library is a new backend implementation of the [ANNIS linguistic search and visualization system](http://corpus-tools.org/annis/).

It is part of the larger system with a web-based front-end, a REST-service (both written in Java).
![graphANNIS architecture overview](graphannis-architecture.png)
As a backend, it is in charge of performing the actual AQL queries and returning the results, which can be either
the number of matches, the IDs of the matches or sub-graphs for a specific set of matches.