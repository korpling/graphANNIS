Linux & MacOS: [![Build Status Linux & MacOS X](https://travis-ci.org/corpus-tools/graphANNIS.svg?branch=develop)](https://travis-ci.org/corpus-tools/graphANNIS)
Windows: [![Build status Windows](https://ci.appveyor.com/api/projects/status/27axqoanq6rj3xps/branch/develop?svg=true)](https://ci.appveyor.com/project/thomaskrause/graphannis/branch/develop)

graphANNIS
==========

This is a prototype for a new backend implementation of the ANNIS linguistic search and visualization system (http://github.com/korpling/ANNIS/). 

While there are test cases it is **highly experimental code and it is not ready to be used by end-users yet**!
Integration with ANNIS is currently implemented in a special branch: https://github.com/thomaskrause/ANNIS/tree/feature/graphannis


How to compile
---------------

graphANNIS is written in the Rust programming language (https://www.rust-lang.org).
You can install Rust from https://www.rust-lang.org/en-US/install.html.
After you have installed Rust, you can can build the complete project with

```
cargo build --release
```

3rd party dependencies
----------------------

This software depends on several 3rd party libraries. These are documented in the BOM.txt file in this folder.

Language bindings
------------------

- Java: https://github.com/corpus-tools/graphANNIS-java
- Python 3: https://github.com/corpus-tools/graphANNIS-python

Author(s)
---------

* Thomas Krause (thomaskrause@posteo.de)
