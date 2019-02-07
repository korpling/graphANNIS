graphANNIS
==========

| Documentation | Build Status
|-------------- |--------------|
| [Developer Guide](https://korpling.github.io/graphANNIS/docs/v0.18/)  |  [![Build Status](https://travis-ci.org/korpling/graphANNIS.svg?branch=develop)](https://travis-ci.org/korpling/graphANNIS) (Linux & MacOS) | 
|  [![API documentation](https://docs.rs/graphannis/badge.svg)](https://docs.rs/graphannis/) (API) | [![Build status](https://ci.appveyor.com/api/projects/status/nc86noerser0bvha/branch/develop?svg=true)](https://ci.appveyor.com/project/thomaskrause/graphannis/branch/develop) (Windows) |

This is a new backend implementation of the ANNIS linguistic search and visualization system (http://corpus-tools.org/annis/). 

While there are test cases, this project is still in a pre-release beta phase. 
**Only a sub-set of the ANNIS Query Langugage (AQL) is supported yet (full support is planned).**
Integration with ANNIS is currently implemented in a special branch: https://github.com/korpling/ANNIS/tree/feature/graphannis

The basic design ideas and data models are described in detail in the PhD-thesis  ["ANNIS: A graph-based query system for deeply annotated text corpora"](https://doi.org/10.18452/19659). The thesis describes a prototype implementation in C++ and not Rust, but the design ideas are the same.
Noteable differences/enhancements compared to the thesis are:
- Graph storages implement querying inverse edges and finding reachable nodes based on them: this allows to implement inverse operators (e.g. for precedence) and  switching operands in situations where it was not possible before.
- The data model has been simplified: the inverse coverage component and inverse edges in the left-/right-most token component have been removed.
- Additional query language features are now supported.


How to compile
---------------

graphANNIS is written in the Rust programming language (https://www.rust-lang.org).
You can install Rust from https://www.rust-lang.org/tools/install.
After you have installed Rust, you can can build the complete project with

```
cargo build --release --all-features
```

3rd party dependencies
----------------------

This software depends on several 3rd party libraries. These are documented in the BOM.txt file in this folder.

Language bindings
------------------

- Java: https://github.com/korpling/graphANNIS-java
- Python 3: https://github.com/korpling/graphANNIS-python

Author(s)
---------

* Thomas Krause (thomaskrause@posteo.de)

How to cite
-----------

```
@phdthesis{Krause2019ANNIS:,
author = {Krause, Thomas},
title = {ANNIS: A graph-based query system for deeply annotated text corpora},
school = {Humboldt-Universität zu Berlin, Mathematisch-Naturwissenschaftliche Fakultät},
year = {2019},
doi = {http://dx.doi.org/10.18452/19659}
}
```
