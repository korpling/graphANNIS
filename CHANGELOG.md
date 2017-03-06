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
