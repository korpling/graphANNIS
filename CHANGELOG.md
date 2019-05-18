# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

### Changed

- `meta::` queries are no deprecated and can only be used in quirks mode

## [0.19.4] - 2019-05-10

### Changed

- Optimize parallel nested loop join by performing less copy operations

### Fixed

- Quirks mode: meta-data nodes are not part of the match result anymore

## [0.19.2] - 2019-04-14

### Fixed

- Escape corpus and document paths with percent encoding when importing them from relANNIS
- Use locale aware sorting of the results in quirks mode (which depends on the system graphANNIS is executed on)
- CLI did not allow to turn quirks mode off once activated

## [0.19.1] - 2019-03-19

### Added

- DOI on Zenodo to cite the Software itself

## [0.19.0] - 2019-03-06

### Added

- Utility function `node_names_from_match` for getting the node identifiers from the matches
- Tutorial for Python, Java and Rust on how to embedd graphANNIS in other programs
- Citation File Format (https://citation-file-format.github.io/) meta-data

### Changed

- **Renamed the "PartOfSubcorpus" component type to more general "PartOf"**
- relANNIS import now takes the sub-corpus structure into account
- Quirks mode now also emulates the component search normalization behavior. 
Search nodes that where part of multiple dominance/pointing relation joins where duplicated and joined with 
the identity operator to work around the issue that nodes of different components could not be joined in relANNIS.
This leads additional output nodes in the find(...) query.
See also the [original JavaDoc](https://github.com/korpling/ANNIS/blob/b7e0e36a0e1ac043e820462dd3f788f5107505a5/annis-service/src/main/java/annis/ql/parser/ComponentSearchRelationNormalizer.java#L32) for an explanation.
- The error_chain crate is no longer used for error reporting, instead a custom Error representation is used

### Fixed

- "NULL" annotation namespaces where imported as "NULL" in relANNIS import
- Result ordering for "find(...)" function was not correct if token helper components where not loaded

## [0.18.1] - 2019-02-08

### Changed

- fixed issue where corpora which contain only tokens could not be queried for a subgraph with context

## [0.18.0] - 2019-02-07

### Added

- Release process is now using the [cargo-release](https://crates.io/crates/cargo-release) script

### Changed

- Separate the update events in smaller chunks for relANNIS import to save memory

## [0.17.2]

### Fixed Bugs

- [#70](https://github.com/korpling/graphANNIS/issues/70) get_all_components() returns all components with matching name if none with the same type exist


## [0.17.1]

### Fixed Bugs

- [#69](https://github.com/korpling/graphANNIS/issues/69) relANNIS-Import: Subgraph query does not work if there is no coverage component.

## [0.17.0]

### Enhancements

- [#68](https://github.com/korpling/graphANNIS/issues/68) Use applyUpdate() API to import legacy relANNIS files
- [#67](https://github.com/korpling/graphANNIS/issues/67) Document the data model of graphANNIS
- [#66](https://github.com/korpling/graphANNIS/issues/66) Automatic creation of inherited coverage edges
- [#65](https://github.com/korpling/graphANNIS/issues/65) Add a new adjecency list based graph storage for dense components.

## [0.16.0]

### Fixed Bugs

- [#62](https://github.com/korpling/graphANNIS/issues/62) Warn about missing coverage edges instead of failing the whole import

### Enhancements

- [#61](https://github.com/korpling/graphANNIS/issues/61) Implement the equal and not equal value operators

## [0.15.0]

### Fixed Bugs

- [#59](https://github.com/korpling/graphANNIS/issues/59) Nodes are not deleted from graph storages via the "applyUpdate" API
- [#55](https://github.com/korpling/graphANNIS/issues/55) Subgraph query does not work if there is no coverage component.
- [#54](https://github.com/korpling/graphANNIS/issues/54) Check all existing matches when checking reflexivity

### Enhancements

- [#58](https://github.com/korpling/graphANNIS/issues/58) Implement ^ (near) operator
- [#57](https://github.com/korpling/graphANNIS/issues/57) Implement ":arity" (number of outgoing edges) unary operator
- [#52](https://github.com/korpling/graphANNIS/issues/52) Use CSV files for query set definition

## [0.14.2]

### Fixed Bugs

- [#50](https://github.com/korpling/graphANNIS/issues/50) Non-reflexive operator join on "any token search" leads to non-empty result
- [#48](https://github.com/korpling/graphANNIS/issues/48) Importing PCC 2.1 corpus hangs at "calculating statistics for component LeftToken/annis/"
- [#46](https://github.com/korpling/graphANNIS/issues/46) Filter not applied for negated annotation search

## [0.14.1]

### Fixed Bugs

- [#45](https://github.com/korpling/graphANNIS/issues/45) Travis configuration used wrong repository and could not deploy release binaries

## [0.14.0]

### Enhancements

- [#44](https://github.com/korpling/graphANNIS/issues/44) Add support for the `_l_` and `_r_` alignment AQL operators
- [#43](https://github.com/korpling/graphANNIS/issues/43) Automatic creation of left- and right-most token edges
- [#42](https://github.com/korpling/graphANNIS/issues/42) Remove inverse coverage and inverse left-/right-most token edges
- [#41](https://github.com/korpling/graphANNIS/issues/41) Add value negation
- [#38](https://github.com/korpling/graphANNIS/issues/38) Add an mdBook based documentation

## [0.13.0]

### Enhancements

- [#36](https://github.com/corpus-tools/graphANNIS/issues/36) Add function to only extract a subgraph with components ofa given type

## [0.12.0]

### Fixed Bugs

- [#34](https://github.com/corpus-tools/graphANNIS/issues/34) Fix loading of edge annotation storages

### Enhancements

- [#33](https://github.com/corpus-tools/graphANNIS/issues/33) Improve memory usage of the relANNIS importer
- [#32](https://github.com/corpus-tools/graphANNIS/issues/32) Faster and more flexible sort of results in "find" function


## [0.11.1]

### Fixed Bugs

- [#31](https://github.com/corpus-tools/graphANNIS/issues/31) Reorder result in find also when acting as a proxy.

release v0.11.0
===============

### Fixed Bugs

- [#30](https://github.com/corpus-tools/graphANNIS/issues/30) Fix most of the queries in the benchmark test test
- [#29](https://github.com/corpus-tools/graphANNIS/issues/29) Use the std::ops::Bound class to mark the upper value instead of relaying on usize::max_value()

### Enhancements

- [#27](https://github.com/corpus-tools/graphANNIS/issues/27) Make the corpus cache more robust and avoid swapping
- [#19](https://github.com/corpus-tools/graphANNIS/issues/19) Check codebase with the clippy tool

release v0.10.1
===============

### Fixed Bugs

- [#26](https://github.com/corpus-tools/graphANNIS/issues/26) Docs.rs does not build because "allocator_api" is not enabled on their rustc


release v0.10.0
===============

### Enhancements

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

### Enhancements

- [#10](https://github.com/corpus-tools/graphANNIS/issues/10) Better error reporting for C-API
- [#8](https://github.com/corpus-tools/graphANNIS/issues/8) Implement AQL parser and replace JSON query representations with AQL

release v0.8.1
==============

### Fixed Bugs

- [#9](https://github.com/corpus-tools/graphANNIS/issues/9) Wait for all background writers before dropping the CorpusStorage

release v0.8.0
==============

### Enhancements

- [#7](https://github.com/corpus-tools/graphANNIS/issues/7) Use error-chain crate for internal error management
- [#6](https://github.com/corpus-tools/graphANNIS/issues/6) Use features of a single crate instead of multiple crates
- [#5](https://github.com/corpus-tools/graphANNIS/issues/5) Allow to delete corpora from the command line
- [#4](https://github.com/corpus-tools/graphANNIS/issues/4) Use file lock to prevent opening the same GraphDB in different processes

release v0.7.1
==============

### Fixed Bugs

- [#3](https://github.com/corpus-tools/graphANNIS/issues/3) Fix automatic creation of binaries using CI for releases

## [0.7.0]

First release of the Rust port of graphANNIS from C++.

## [0.6.0]

### Fixed Bugs

- [#23](https://github.com/thomaskrause/graphANNIS/issues/23) Problems loading the cereal archive under Windows

## [0.5.0]

### Enhancements

- [#22](https://github.com/thomaskrause/graphANNIS/issues/22) Use text-book function for estimating the selectivity for the abstract edge operator
- [#21](https://github.com/thomaskrause/graphANNIS/issues/21) Allow to load query in console from file


## [0.4.0]

### Fixed Bugs

- [#20](https://github.com/thomaskrause/graphANNIS/issues/20) UniqueDFS should output each matched node only once, but still visit each node.
- [#14](https://github.com/thomaskrause/graphANNIS/issues/14) Do not iterate over covered text positions but use the token index 
- [#13](https://github.com/thomaskrause/graphANNIS/issues/13) Fix duplicate matches in case a const anno value is used in a base search

### Enhancements

- [#19](https://github.com/thomaskrause/graphANNIS/issues/19) Update the re2 regex library and make sure it is compiled with -O3 optimizations
- [#18](https://github.com/thomaskrause/graphANNIS/issues/18) Perform more pessimistic estimates for inclusion and overlap operators
- [#17](https://github.com/thomaskrause/graphANNIS/issues/17) Optimize meta data search
- [#16](https://github.com/thomaskrause/graphANNIS/issues/16) Allow base node search by membership in a component
- [#15](https://github.com/thomaskrause/graphANNIS/issues/15) Better handling of Regular Expressions on a RHS of an index join
- [#12](https://github.com/thomaskrause/graphANNIS/issues/12) Add support for relANNIS style multiple segmentation


## [0.3.0]

### Fixed Bugs

- [#8](https://github.com/thomaskrause/graphANNIS/issues/8) Fix shared/unique lock handling in CorpusStorageManager when component needs to be loaded
- [#4](https://github.com/thomaskrause/graphANNIS/issues/4) Node names should include the document name (and the URL specific stuff) when imported from Salt.

### Enhancements

- [#11](https://github.com/thomaskrause/graphANNIS/issues/11) Optimize unbound regex annotation searches
- [#10](https://github.com/thomaskrause/graphANNIS/issues/10) Do some small enhancements to regex handling
- [#9](https://github.com/thomaskrause/graphANNIS/issues/9) Add an API to query subgraphs
- [#7](https://github.com/thomaskrause/graphANNIS/issues/7) Support OR queries
- [#6](https://github.com/thomaskrause/graphANNIS/issues/6) Add metadata query support
- [#5](https://github.com/thomaskrause/graphANNIS/issues/5) Add a SIMD based join


## [0.2.0]

### Fixed Bugs

- [#4](https://github.com/thomaskrause/graphANNIS/issues/4) Node names should include the document name (and the URL specific stuff) when imported from Salt.

### Enhancements

- [#3](https://github.com/thomaskrause/graphANNIS/issues/3) Make the graphANNIS API for Java an OSGi bundle
- [#2](https://github.com/thomaskrause/graphANNIS/issues/2) Avoid local minima when using the random query optimizer
- [#1](https://github.com/thomaskrause/graphANNIS/issues/1) Use "annis" instead of "annis4_internal" as namespace

## [0.1.0]

Initial development release with an actual release number.

There has been the benchmark-journal-2016-07-27 tag before which was used in a benchmark for a paper.
Since then the following improvements have been made:
- using an edge annotation as base for a node search on the LHS of the join
- adding parallel join implementations

This release is also meant to test the release cycle (e.g. Maven Central deployment) itself.
