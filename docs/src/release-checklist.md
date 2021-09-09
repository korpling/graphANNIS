# Release checklist

This is a list of steps needed to perform a new graphANNIS release.

GraphANNIS uses [semantic versioning](https://semver.org/).
Language bindings can have their own version number, but should state which core version is used in its documentation.

1. Check the changelog (`CHANGELOG.md`): note the last release version number and which kind of changes have been made since the last release.
   **Determine if this is a major, minor or patch release** according to [semantic versioning](https://semver.org/). 
2. **Release** the branch with the [cargo-release plugin](https://crates.io/crates/cargo-release)
   - `cargo release release` to release the current development version (e.g. 1.0.0-dev will be released as 1.0.0)
   - `cargo release patch` for patch updates (e.g. 1.0.0 to 1.0.1)
   - `cargo release minor` for minor updates (e.g. 1.0.1 to 1.1.0)
   - `cargo release major` for major updates (e.g. 1.1.0 to 2.0.0)
3.  Create the **release on GitHub**, copy the changelog entry as release notes and **publish the release**.
    The release artifacts will be created and attached to this release automatically by the `deploy` GitHub Actions workflow.
4.  Update and release language bindings 
    -  [Python](https://github.com/korpling/graphANNIS-python#release-process)
    -  [Java](https://github.com/korpling/graphANNIS-java#release-process)

In addition, for the binaries, CI will also build and publish the documentation using the `gh-pages` branch and a sub-folder `docs/<short-version>`, e.g. https://korpling.github.io/graphANNIS/docs/v1.1/.
