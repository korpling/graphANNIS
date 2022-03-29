# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Refactored the basic `GraphStorage` and `AnnotationStorage` APIs to handle
  errors. Previously, we used mostly main memory containers which had an API
  that could not fail. This was reflected in the `GraphStorage` and
  `AnnotationStorage`  APIs, which directly returned the result or an iterator
  over the non-fallible results. With the addition of disk-based
  implementations, this API model was not possible to implement without using
  panics when repeated access to the disk failed. Some of the API that was
  changed was user visible when using the `graphannis-core` crate (and thus the
  C-API), so this release is not technically backwards-compatible. Adapting to
  the updated API should be restricted to handle the errors returned by the
  functions.
- The changes to the error handling also affects the C-API. These following 
  functions have now a `ErrorList` argument:
  * `annis_cs_list_node_annotations`
  * `annis_cs_list_edge_annotations`
  * `annis_cs_list_components_by_type`
  * `annis_cs_unload`
  * `annis_iter_nodeid_next`
  * `annis_graph_annotations_for_node`
  * `annis_graph_outgoing_edges`
  * `annis_graph_annotations_for_edge`
- Renamed the Criterion-based benchmark CLI to `bench_queries` and synchronize
  its arguments to the current version of Criterion.

### Fixed

- More efficient node path extraction in `count_extra` function and when sorting
  the matches.
- Avoid large memory consumption when importing GraphML files by resetting an
  internal buffer on each XML event.
- Limit the number of disk maps for the `GraphUpdate` so there are less issues
  with large corpora where the maximum number of open files per process might be
  reached.
- Performance improvements when importing large corpora in disk-based mode. This
  optimizes the DiskMap to use a C0 (normal in memory BTree), a C1 (on disk
  BTree) and a C2 map when serialized to disk. On compacting, the entries are
  only written to C1 in O(n*log(n)). Before, multiple on disk maps might need to
  be merged, which had a much worse complexity. The C1 file uses the
  transient-btree-index crate.
- Trim mapping entries when importing relANNIS resolver files (#222).

## [1.5.0] - 2022-01-06

### Fixed

- RelANNIS version 3.3 files with segmentation might also have a missing "span" column.
  In case the "span" column is null, always attempt to reconstruct the actual value from
  the corresponding node annotation instead of failing directly.
  
### Changed

- Avoid unnecessary compacting of disk tables when collecting graph updates during import.
  This speeds up both the GraphML and the relANNIS importer and can also reduce the
  used main memory during import.
- Use release optimization of some of the performance sensitive crates even for debug builds.
  This allows faster builds and debugging of our own code, while balancing performance.

## [1.4.1] - 2021-12-07

### Fixed

- Avoid unnecessary memory allocation when checking if a node has outgoing edges in
  adjacency lists. This improves search for tokens because the Coverage components
  are typically adjacency lists, and we need to make sure the token nodes don't
  have any outgoing edges.
- Fixed miscalculation of whitespace string capacity which could lead to 
  `memory allocation failed` error.

## [1.4.0] - 2021-12-03

### Added

- Added `clear()` method to the `WriteableGraphStorage` trait.

### Fixed

- Limit the used main memory cache per `DiskTable` by only using a disk block cache for the C1 table.
  Since we use a lot of disk-based maps during import of relANNIS files, the previous behavior could 
  add up to > 1GB easily, wich amongst other issues caused #205 to happen.
  With this change, during relANNIS import the main memory usage should be limited to be less than 4GB, 
  which seams more reasonable than the previous 20+GB
- Reduce memory footprint during import when corpus contains a lot of escaped strings (as in #205)
- Avoid creating small fragmented main memory when importing corpora from relANNIS to help to fix #205

### Changed

- Improved overall import speed of relANNIS corpora and when applying graph updates

## [1.3.0] - 2021-09-20

### Added

- The webservice endpoint `/search/node-descriptions` now returns
  wether a node in the query is optional or not.

## [1.2.2] - 2021-09-20

### Fixed

- Queries with optional nodes with a smaller index than the last non-optional node could fail.
  If the execution nodes re-order the match result vector internally, the query node index is
  used to define the mapping. Unfortunately the largest index could be larger than the size of mappings,
  which used to be used to create the output vector. By allowing empty elements in the output vector
  and using the maximum value, we can still map the results properly.

## [1.2.1] - 2021-09-16

### Fixed

- Don't allow optional operands for non-negated operators

## [1.2.0] - 2021-09-16

### Added

- Added generic operator negation without existence assumption, 
  if only one side of the negated operator is optional  (#187).

## [1.1.0] - 2021-09-09

### Added

- Added generic operator negation with existence assumption by adding `!` before the binary operator (#186)

### Changed

- Compile releases on Ubuntu 18.04 instead of 16.04, which means the minimal GLIBC version is 2.27
- Updated dependencies
- Improved compile time by disabling some dependency features.
  This also removes some optional features from the command line parser
  (used in webservice and CLI binaries).
- Don't use RIDGES corpus in search tests and fail search tests when corpus does not exist.

### Fixed

- Use the correct `set-disk-based on` command in the documentation for the CLI
- Optimize node annotation storage and graph implementations when importing GraphML files

## [1.0.2] - 2021-08-20

### Fixed

- Fix issue when deploying release artifacts on GitHub

## [1.0.1] - 2021-08-20

## Fixed

- Assume that the `annis::node_name` annotation is unique when estimating match size.
  This should improve e.g. subgraph-queries, where the intermediate result sizes are now better estimated.


## [1.0.0] - 2021-08-17

### Changed

- The default context sizes in the corpus configuration now include 0 (#181)

## [0.32.0] - 2021-08-09

### Added

- C-API now implements exporting corpora

### Changed

- Renamed (public) function `export_corpus_zip` in `CorpusStorage` to `export_to_zip` to align with the other export function name.

### Fixed

- Exporting a corpus without a "files" directory failed

## [0.31.2] - 2021-04-01

### Fixed

- Synchronize REST API error output for bad AQL requests with the OpenAPI specification.

## [0.31.1] - 2021-03-05

### Fixed

- Fix compilation issues in interaction with lalrpop v0.19.5

## [0.31.0] - 2021-02-18

### Changed

- Using the new `SmallVec`-based `MatchGroup` type instead of `Vec<Match>`.
- The `FixedMaxMemory` `CacheStrategy` now uses Megabytes instead of bytes.
- The graphannis and core crates now use their own error type instead of the one provided by the `anyhow` crate.
- Bundle commonly used search query parameters in `SearchQuery` struct.
- Query execution methods now have an optional `timeout` after which an query is aborted.
- Annotation keys and values in the `AnnoKey` and `Annotation` structs now use inlined strings from the `smartstrings` crate.

### Removed

- Replaced the `update_statistics` function in `CorpusStorage` with the more general `reoptimize_implementation` function. 
  The new function is available via the `re-optimize` command in the CLI.

### Added

- The webservice configuration now allows to configure the size of the in-memory corpus cache.
- There can be multiple `--cmd` arguments for the CLI, which are executed in the order they are given.

### Fixed

- Importing a relANNIS corpus could fail because the integer would wrap around from negative to a large value when calculating the `tok-whitespace-after` annotation value. This large value would then be used to allocate memory, which will fail. 
- Adding `\$` to the escaped input sequence in the relANNIS import, fixing issues with some old SFB 632 corpora
- Unbound near-by-operator (`^*`) was not limited to 50 in quirks mode
- Workaround for duplicated document names when importing invalid relANNIS corpora
- Corpus names with non-ASCII characters where not listed with their decoded name
- Fix memory consumption of AQL parser in repeated calls (like the webservice).
- Limit the memory which is reserved for an internal result vector to avoid out-of-memory errors when the estimation is wrong.

## [0.30.0] - 2020-09-30

### Changed

- JWT secret configuration now supports RS256 in addition to HS256. This enables support of applications which use Keycloak as their identity provider, since they only provide public keys.
- JWT tokens now should have the `roles` field instead of using the `admin` field. This enhances compatibility with Keycloak.
- Pull requests are now checked with the Clippy static code analyis tool
- Updated Actix Web dependency for webservice to version 3

### Removed

- The REST API does not act as an identity provider anymore and the `/local-login` endpoint has been removed

## [0.29.2] - 2020-08-25

### Fixed

- Travis did add the webservice executables to the release

## [0.29.1] - 2020-08-25

### Fixed

- `cargo release` did not release all crates

## [0.29.0] - 2020-08-25

### Changed

- Node IDs in matches don't have the `salt:/` prefix anymore

### Added

- Add non-tokenized primary text segments as special labels "tok-whitespace-before" and "tok-whitespace-after" to the existing token
  when importing from relANNIS. This allows to re-construct the original relANNIS primary text by iterating over all token in order
  and be prepending or append these labels to the token values.
- Add a REST based web-service replacing the legacy annis-service

### Fixed

- Load all components when extracting a subgraph using an AQL query

## [0.28.0] - 2020-07-02

### Addded

- Web Service with REST API for the corpus storage
- Copy and link files from the ExtData folder when importing relANNIS.
- Map `resolver_vis_map.annis`, `example_queries.annis` and `corpus.properties` from relANNIS files
  to a new unified corpus configuration stored as [TOML]() file. This corpus configuration
  is also exported to GraphML.
- Export and import ZIP files containing multiple corpora.

### Removed

- Removed Brotli support: use the ZIP file export instead

## [0.27.0] - 2020-06-08

### Changed

- Backward incompatible: Return opaque [anyhow](https://github.com/dtolnay/anyhow) `Error` type in all
  functions instead of our own enum.
  The new `Error` type also implements `std::error::Error` and is equivalent to using `Box<dyn std:error::Error>`.
- Upgraded parser generator lalrpop to version 0.18.x

### Added

- Disk-based implementation of an adjacency list is used when a corpus is configured to be prefer disk over memory.
- Ability to export and import GraphML files. This follows the [Neo4j dialect of GraphML](https://neo4j.com/docs/labs/apoc/current/import/graphml/).
  It is also possible to compress the GraphML files with Brotli.

### Fixed

- The dense adjacency list implementation did not implement the `source_nodes` function properly

## [0.26.0] - 2020-03-05

### Removed

- Removed the unintentionally public `size_of_cached` function of `Graph` from the API.

### Changed

- Backward incompatible: the `AnnotationStorage` and `WriteableGraphStorage` interfaces have been adjusted to return `Result` types for mutable functions.
  This change is necessary because on-disk annotation storage implementations might fail, and we want to handle it when modifying the annotation storage.
- Improved main memory usage when importing relANNIS files.
  The implementation now uses temporary disk-based maps instead of memory-intensive maps.
  This change also affects the `GraphUpdate` class, which is now disk-based, too.

### Added

- Added disk-based annotation storage for nodes as an alternative to the memory-only variant.
  On the console, use `use_disk <on|off>` to set if newly imported corpora prefer disk-based annotation storage.
  `disk_based` parameters are also added to the various "import relANNIS" API functions.

### Fixed

- Reconstruct coverage edges with the correct component, if the actual edges are omitted in rank.annis,
  but the ones without a parent node are still present. [#125](https://github.com/korpling/graphANNIS/issues/125)

## [0.25.1] - 2020-01-03

### Fixed

- Inverted sort order did not reverse the corpus name list for multiple corpora
- Workaround for docs.rs problems seem to have caused other problems and graphANNIS was not recognized as library

## [0.25.0] - 2019-11-25

### Changed

- Backward incompatible: the several search functions (`find`, `count`, etc.) not take several corpus names as argument.
  This is especially important for `find`, where the implementation can be optimized to correctly skip over a given offset
  using the internal state.
  Such an optimization is impossible from outside when calling the API and not having access to the iterator.

### Fixed

- Don't assume inverse operator has the same cost when fan-out is too different.
  Subgraph queries could be very slow for corpora with large documents due to an estimation error from this assumption
  the `@` operator.

## [0.24.0] - 2019-11-15

### Changed

- The annotation storage is now a complete interface which provides all functions necessary to write and read annotations.
  To make this less dependent on the current implementation of the
  in-memory annotation storage, the annotation key symbol (an integer) has been removed.
  This annotation key symbol has been used in the `Match` class as well, which is now using an `Arc<AnnoKey>` instead. The `AnnoKey` contains
  the fully qualified name as `String`.
  Several functions of the annotation storage that used to have `String` parameters now take `&str` and resulting string values are now returned as `Cow<str>`. The latter change is also meant to enable more flexible implementations, that can choose to allocate new strings (e.g. from disk) or return references to existing memory locations.
- The `Graph` uses a boxed instance of the general `AnnotationStorage` trait.
  Before, this was an `Arc` to the specific implementation, which made it possible to simply clone the node annotation storage.
  Now, references to it must be used, e.g. in the operators. This changes a lot of things in the `BinaryOperator` trait, like
  the signature of `get_inverse_operator()` and the filter functions that are used as conditions for the node search (these
  need an argument to the node annotation storage now)
- `Graph` does not implement the `AnnotationStorage<NodeID>` trait anymore,
  but provides a getter to reference its field.
- Data source nodes are now included when querying for a subgraph with context. This is needed for parallel text support in ANNIS 4.

### Added

- Show the used main memory for the node annotations

## [0.23.1] - 2019-10-16

### Fixed

- Deploying release artifacts by CI was broken due to invalid condition

## [0.23.0] - 2019-10-16

### Added

- Subgraph queries can now define the context using ordering relation names (segmentation)
  instead of the default context in tokens. **This changes the function signature of the `subgraph(...)` function.**

### Changed

- For performance and stylistic reasons, the GraphStorage API has been changed to accept integer node IDs instead of references to integers.
- Windows DLL in releases is now created by Travis CI instead of Appveyor

## [0.22.0] - 2019-07-22

### Fixed

- Windows DLL generated by CI was empty

### Changed

- Updated several dependencies
- Organize documentation topics in sub-folders.
  Previously, mdbook did not updated the images on these sites on the print.html.
  Since mdbook >0.3.1 this is fixed and we can use the better layout.

## [0.21.0] - 2019-05-26

### Changed

- C API now has an argument to return error messages when creating a corpus storage

### Added

- C API now also allows to unload a corpus from the cache manually

### Fixed

- CorpusStorageManager: Escape the corpus name when writing it to its disk location to support e.g. corpora with slash
  in their name.
- Quirks mode: sort matches by reversed document path (document first)
- Node names/paths where double encoded both when importing them and when executing the "find" function
- Quirks mode: use default collation of Rust for corpora imported from relANNIS 3.3

## [0.20.0] - 2019-05-19

### Deprecated

- `meta::` queries are now deprecated and can only be used in quirks mode

### Fixed

- Output annotations with the namespace "annis" in find function
- Quirks mode: add additional identity joins in the order as the nodes are defined in the query
- Encode ",", " " and ":" in the Salt ID output of the `find(...)` function
- Sort longer vectors ("more specific") before shorter ones in `find(...)` output

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

# release v0.11.0

### Fixed Bugs

- [#30](https://github.com/corpus-tools/graphANNIS/issues/30) Fix most of the queries in the benchmark test test
- [#29](https://github.com/corpus-tools/graphANNIS/issues/29) Use the std::ops::Bound class to mark the upper value instead of relaying on usize::max_value()

### Enhancements

- [#27](https://github.com/corpus-tools/graphANNIS/issues/27) Make the corpus cache more robust and avoid swapping
- [#19](https://github.com/corpus-tools/graphANNIS/issues/19) Check codebase with the clippy tool

# release v0.10.1

### Fixed Bugs

- [#26](https://github.com/corpus-tools/graphANNIS/issues/26) Docs.rs does not build because "allocator_api" is not enabled on their rustc

# release v0.10.0

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

# release v0.9.0

### Enhancements

- [#10](https://github.com/corpus-tools/graphANNIS/issues/10) Better error reporting for C-API
- [#8](https://github.com/corpus-tools/graphANNIS/issues/8) Implement AQL parser and replace JSON query representations with AQL

# release v0.8.1

### Fixed Bugs

- [#9](https://github.com/corpus-tools/graphANNIS/issues/9) Wait for all background writers before dropping the CorpusStorage

# release v0.8.0

### Enhancements

- [#7](https://github.com/corpus-tools/graphANNIS/issues/7) Use error-chain crate for internal error management
- [#6](https://github.com/corpus-tools/graphANNIS/issues/6) Use features of a single crate instead of multiple crates
- [#5](https://github.com/corpus-tools/graphANNIS/issues/5) Allow to delete corpora from the command line
- [#4](https://github.com/corpus-tools/graphANNIS/issues/4) Use file lock to prevent opening the same GraphDB in different processes

# release v0.7.1

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
