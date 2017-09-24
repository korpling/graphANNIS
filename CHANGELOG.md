release 0.4.0
=============

Fixed Bugs
----------

- [#14](https://github.com/thomaskrause/graphANNIS/issues/14) Do not iterate over covered text positions but use the token index 
- [#13](https://github.com/thomaskrause/graphANNIS/issues/13) Fix duplicate matches in case a const anno value is used in a base search

Enhancements
------------

- [#18](https://github.com/thomaskrause/graphANNIS/issues/18) Perform more pessimistic estimates for inclusion and overlap operators
- [#17](https://github.com/thomaskrause/graphANNIS/issues/17) Optimize meta data search
- [#16](https://github.com/thomaskrause/graphANNIS/issues/16) Allow base node search by membership in a component
- [#15](https://github.com/thomaskrause/graphANNIS/issues/15) Better handling of Regular Expressions on a RHS of an index join
- [#12](https://github.com/thomaskrause/graphANNIS/issues/12) Add support for relANNIS style multiple segmentation

Other
-----

- [#19](https://github.com/thomaskrause/graphANNIS/issues/19) Update the re2 regex library and make sure it is compiled with -O3 optimizations

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
