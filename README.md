# graphANNIS [![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.2598164.svg)](https://doi.org/10.5281/zenodo.2598164) ![Automated tests](https://github.com/korpling/graphANNIS/workflows/Automated%20tests/badge.svg)

This is a new backend implementation of the ANNIS linguistic search and visualization system (http://corpus-tools.org/annis/).

**Only a sub-set of the ANNIS Query Language (AQL) from ANNIS version 3 (based on PostgreSQL) is supported yet.**
More operators can be added in the future, but the ones missing are the ones which have been used less frequent.
There is a tutorial in the Developer Guide on how to embed graphANNIS in your own application.

The basic design ideas and data models are described in detail in the PhD-thesis ["ANNIS: A graph-based query system for deeply annotated text corpora"](https://doi.org/10.18452/19659). The thesis describes a prototype implementation in C++ and not Rust, but the design ideas are the same.
Notable differences/enhancements compared to the thesis are:

- Graph storages implement querying inverse edges and finding reachable nodes based on them: this allows to implement inverse operators (e.g. for precedence) and switching operands in situations where it was not possible before.
- The data model has been simplified: the inverse coverage component and inverse edges in the left-/right-most token component have been removed.
- Additional query language features are now supported.

## Documentation

- [Developer Guide](https://korpling.github.io/graphANNIS/docs/v2/) (including descriptions of the data model and tutorials for the API)
- [API documentation](https://docs.rs/graphannis/)


## Developing graphANNIS

You need to install Rust to compile the project.
We recommend installing the following Cargo subcommands for developing annis-web:

- [cargo-release](https://crates.io/crates/cargo-release) for creating releases
- [cargo-about](https://crates.io/crates/cargo-about) for re-generating the
  third party license file
- [cargo-llvm-cov](https://crates.io/crates/cargo-llvm-cov) for determining the code coverage
- [cargo-dist](https://crates.io/crates/cargo-dist) for configuring the GitHub actions that create the release binaries.

### Execute tests

You can run the tests with the default `cargo test` command.
To calculate the code coverage, you can use `cargo-llvm-cov`:

```bash
cargo llvm-cov --open --all-features --ignore-filename-regex '(tests?\.rs)|(capi/.*)'
```


### Performing a release

You need to have [`cargo-release`](https://crates.io/crates/cargo-release)
installed to perform a release. Execute the follwing `cargo` command once to
install it.

```bash
cargo install cargo-release
```

To perform a release, switch to the main branch and execute:

```bash
cargo release --execute
```

This will also trigger a CI workflow to create release binaries on GitHub.



## 3rd party dependencies

This software depends on several 3rd party libraries. These are documented in the "third-party-licenses.html" file in this folder.

## Language bindings

- Java: https://github.com/korpling/graphANNIS-java
- Python 3: https://github.com/korpling/graphANNIS-python
- Rust (this repository)
- C (this repository)

## Author(s)

- Thomas Krause (thomas.krause@hu-berlin.de)
