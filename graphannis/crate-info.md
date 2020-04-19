graphANNIS
==========

This is a new backend implementation of the ANNIS linguistic search and visualization system (http://corpus-tools.org/annis/). 

While there are test cases, this project is still in a pre-release beta phase. 
**Only a sub-set of the ANNIS Query Langugage (AQL) is supported yet (full support is planned).**
Integration with ANNIS is currently implemented in [the development version](https://github.com/korpling/ANNIS/tree/develop) and [released as a beta version](http://corpus-tools.org/annis/download.html).
There is a tutorial in the Developer Guide on how to embedd graphANNIS in your own application.

The basic design ideas and data models are described in detail in the PhD-thesis  ["ANNIS: A graph-based query system for deeply annotated text corpora"](https://doi.org/10.18452/19659). The thesis describes a prototype implementation in C++ and not Rust, but the design ideas are the same.
Noteable differences/enhancements compared to the thesis are:
- Graph storages implement querying inverse edges and finding reachable nodes based on them: this allows to implement inverse operators (e.g. for precedence) and  switching operands in situations where it was not possible before.
- The data model has been simplified: the inverse coverage component and inverse edges in the left-/right-most token component have been removed.
- Additional query language features are now supported.
