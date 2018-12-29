release 0.14.0
==============

Enhancements
------------

- [#44](https://github.com/korpling/graphANNIS/issues/44) Add support for the `_l_` and `_r_` alignment AQL operators
- [#43](https://github.com/korpling/graphANNIS/issues/43) Automatic creation of left- and right-most token edges
- [#42](https://github.com/korpling/graphANNIS/issues/42) Remove inverse coverage and inverse left-/right-most token edges
- [#41](https://github.com/korpling/graphANNIS/issues/41) Add value negation
- [#38](https://github.com/korpling/graphANNIS/issues/38) Add an mdBook based documentation

release 0.13.0
==============

Enhancements
------------

- [#36](https://github.com/corpus-tools/graphANNIS/issues/36) Add function to only extract a subgraph with components ofa given type

release 0.12.0
==============

Fixed Bugs
----------

- [#34](https://github.com/corpus-tools/graphANNIS/issues/34) Fix loading of edge annotation storages

Enhancements
------------

- [#33](https://github.com/corpus-tools/graphANNIS/issues/33) Improve memory usage of the relANNIS importer
- [#32](https://github.com/corpus-tools/graphANNIS/issues/32) Faster and more flexible sort of results in "find" function


release 0.11.1
==============

Fixed Bugs
----------

- [#31](https://github.com/corpus-tools/graphANNIS/issues/31) Reorder result in find also when acting as a proxy.

release v0.11.0
===============

Fixed Bugs
----------

- [#30](https://github.com/corpus-tools/graphANNIS/issues/30) Fix most of the queries in the benchmark test test
- [#29](https://github.com/corpus-tools/graphANNIS/issues/29) Use the std::ops::Bound class to mark the upper value instead of relaying on usize::max_value()

Enhancements
------------

- [#27](https://github.com/corpus-tools/graphANNIS/issues/27) Make the corpus cache more robust and avoid swapping
- [#19](https://github.com/corpus-tools/graphANNIS/issues/19) Check codebase with the clippy tool

release v0.10.1
===============

Fixed Bugs
----------

- [#26](https://github.com/corpus-tools/graphANNIS/issues/26) Docs.rs does not build because "allocator_api" is not enabled on their rustc


release v0.10.0
===============

Enhancements
------------

- [#24](https://github.com/corpus-tools/graphANNIS/issues/24) Implement regular expression search for edge annotations.
- [#23](https://github.com/corpus-tools/graphANNIS/issues/23) Update the C-API to reflect the changes in the Rust API
- [#22](https://github.com/corpus-tools/graphANNIS/issues/22) Use the published graphannis-malloc_size_of crate
- [#21](https://github.com/corpus-tools/graphANNIS/issues/21) Restructure and document the public API
- [#15](https://github.com/corpus-tools/graphANNIS/issues/15) Move all modules into a private "annis" sub-module
- [#14](https://github.com/corpus-tools/graphANNIS/issues/14) Simplify the code for the graph storage registry
- [#13](https://github.com/corpus-tools/graphANNIS/issues/13) Save memory in the annotation storage
- [#12](https://github.com/corpus-tools/graphANNIS/issues/12) Improve speed of loading adjacency list graph storages
- [#11](https://github.com/corpus-tools/graphANNIS/issues/11) Use criterion.rs library for benchmarks


release v0.9.0
==============

Enhancements
------------

- [#10](https://github.com/corpus-tools/graphANNIS/issues/10) Better error reporting for C-API
- [#8](https://github.com/corpus-tools/graphANNIS/issues/8) Implement AQL parser and replace JSON query representations with AQL

release v0.8.1
==============

Fixed Bugs
----------

- [#9](https://github.com/corpus-tools/graphANNIS/issues/9) Wait for all background writers before dropping the CorpusStorage

release v0.8.0
==============

Enhancements
------------

- [#7](https://github.com/corpus-tools/graphANNIS/issues/7) Use error-chain crate for internal error management
- [#6](https://github.com/corpus-tools/graphANNIS/issues/6) Use features of a single crate instead of multiple crates
- [#5](https://github.com/corpus-tools/graphANNIS/issues/5) Allow to delete corpora from the command line
- [#4](https://github.com/corpus-tools/graphANNIS/issues/4) Use file lock to prevent opening the same GraphDB in different processes

release v0.7.1
==============

Fixed Bugs
----------

- [#3](https://github.com/corpus-tools/graphANNIS/issues/3) Fix automatic creation of binaries using CI for releases

release 0.7.0
=============

First release of the Rust port of graphANNIS from C++.

release 0.6.0
=============

Fixed Bugs
----------

- [#23](https://github.com/thomaskrause/graphANNIS/issues/23) Problems loading the cereal archive under Windows

release 0.5.0
=============

Enhancements
------------

- [#22](https://github.com/thomaskrause/graphANNIS/issues/22) Use text-book function for estimating the selectivity for the abstract edge operator
- [#21](https://github.com/thomaskrause/graphANNIS/issues/21) Allow to load query in console from file


release 0.4.0
=============

Fixed Bugs
----------

- [#20](https://github.com/thomaskrause/graphANNIS/issues/20) UniqueDFS should output each matched node only once, but still visit each node.
- [#14](https://github.com/thomaskrause/graphANNIS/issues/14) Do not iterate over covered text positions but use the token index 
- [#13](https://github.com/thomaskrause/graphANNIS/issues/13) Fix duplicate matches in case a const anno value is used in a base search

Enhancements
------------

- [#19](https://github.com/thomaskrause/graphANNIS/issues/19) Update the re2 regex library and make sure it is compiled with -O3 optimizations
- [#18](https://github.com/thomaskrause/graphANNIS/issues/18) Perform more pessimistic estimates for inclusion and overlap operators
- [#17](https://github.com/thomaskrause/graphANNIS/issues/17) Optimize meta data search
- [#16](https://github.com/thomaskrause/graphANNIS/issues/16) Allow base node search by membership in a component
- [#15](https://github.com/thomaskrause/graphANNIS/issues/15) Better handling of Regular Expressions on a RHS of an index join
- [#12](https://github.com/thomaskrause/graphANNIS/issues/12) Add support for relANNIS style multiple segmentation


release 0.3.0
=============

Fixed Bugs
----------

- [#8](https://github.com/thomaskrause/graphANNIS/issues/8) Fix shared/unique lock handling in CorpusStorageManager when component needs to be loaded
- [#4](https://github.com/thomaskrause/graphANNIS/issues/4) Node names should include the document name (and the URL specific stuff) when imported from Salt.

Enhancements
------------

- [#11](https://github.com/thomaskrause/graphANNIS/issues/11) Optimize unbound regex annotation searches
- [#10](https://github.com/thomaskrause/graphANNIS/issues/10) Do some small enhancements to regex handling
- [#9](https://github.com/thomaskrause/graphANNIS/issues/9) Add an API to query subgraphs
- [#7](https://github.com/thomaskrause/graphANNIS/issues/7) Support OR queries
- [#6](https://github.com/thomaskrause/graphANNIS/issues/6) Add metadata query support
- [#5](https://github.com/thomaskrause/graphANNIS/issues/5) Add a SIMD based join


release 0.2.0
=============

Fixed Bugs
----------

- [#4](https://github.com/thomaskrause/graphANNIS/issues/4) Node names should include the document name (and the URL specific stuff) when imported from Salt.

Enhancements
------------

- [#3](https://github.com/thomaskrause/graphANNIS/issues/3) Make the graphANNIS API for Java an OSGi bundle
- [#2](https://github.com/thomaskrause/graphANNIS/issues/2) Avoid local minima when using the random query optimizer
- [#1](https://github.com/thomaskrause/graphANNIS/issues/1) Use "annis" instead of "annis4_internal" as namespace

release 0.1.0
=============

Initial development release with an actual release number.

There has been the benchmark-journal-2016-07-27 tag before which was used in a benchmark for a paper.
Since then the following improvements have been made:
- using an edge annotation as base for a node search on the LHS of the join
- adding parallel join implementations

This release is also meant to test the release cycle (e.g. Maven Central deployment) itself.
