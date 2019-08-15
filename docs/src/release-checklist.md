# Release checklist

This is a list of steps needed to perform a new graphANNIS release.

GraphANNIS uses [semantic versioning](https://semver.org/).
Language bindings can have their own version number, but should state which core version is used in its documentation.

1. Create and push a new **release branch** `release/<version>` from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch.
2. If this is a major or minor release, **update the link to the Developer Guide** documentation in the `README.md` to point to the new version short version, omitting the patch level and with adding a "v" prefix (e.g. https://korpling.github.io/graphANNIS/docs/v1.0/)
3. **Release** the branch with the [cargo-release plugin](https://crates.io/crates/cargo-release)
   - `cargo release patch` for hotfixes updates (e.g. 1.0.0 to 1.0.1)
   - `cargo release minor` for minor updates (e.g. 1.0.1 to 1.1.0)
   - `cargo release major` for major updates (e.g. 1.1.0 to 2.0.0)
4. **Merge** the release branch both into the master and develop branch then delete the release branch.
5.  Create the **release on GitHub**, copy the changelog entry as release notes. Save the release as draft
6.  Wait for Continuous Integration (CI) to finish building the release artifacts for all systems and then **publish the drafted release**
7.  Update and release languagage bindings 
    -  [Python](https://github.com/korpling/graphANNIS-python#release-process)
    -  [Java](https://github.com/korpling/graphANNIS-java#release-process)

In addition for the binaries, CI will also build and publish the documentation using the gh-pages branch and a sub-folder `docs/<short-version>`, e.g. https://korpling.github.io/graphANNIS/docs/v0.22/.
