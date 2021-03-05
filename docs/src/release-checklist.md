# Release checklist

This is a list of steps needed to perform a new graphANNIS release.

GraphANNIS uses [semantic versioning](https://semver.org/).
Language bindings can have their own version number, but should state which core version is used in its documentation.

1. Create and push a new **release branch** `release/<version>` (e.g. `release/0.31.0`) from the `develop` branch for feature releases. If you make a bug-fix release, create a branch named `hotfix/<version>` (e.g. `hotfix/0.31.1`) from the `master` branch. Do not prefix `<version>` with `v`.
2. If this is a major or minor release, **update the link to the Developer Guide** documentation in the `README.md` to point to the new version short version, omitting the patch level and with adding a “v” prefix (e.g. https://korpling.github.io/graphANNIS/docs/v1.0/)
3. **Release** the branch with the [cargo-release plugin](https://crates.io/crates/cargo-release)
   - `cargo release release` to release the current development version (e.g. 1.0.0-dev will be released as 1.0.0)
   - `cargo release patch` for hotfixes updates (e.g. 1.0.0 to 1.0.1)
   - `cargo release minor` for minor updates (e.g. 1.0.1 to 1.1.0)
   - `cargo release major` for major updates (e.g. 1.1.0 to 2.0.0)
5. **Merge** the release branch both into the master and develop branch then delete the release branch.
6.  Create the **release on GitHub**, copy the changelog entry as release notes and **publish the release**.
    The release artifacts will be created and attached to this release automatically by the `deploy` GitHub Actions workflow.
7.  Update and release language bindings 
    -  [Python](https://github.com/korpling/graphANNIS-python#release-process)
    -  [Java](https://github.com/korpling/graphANNIS-java#release-process)

In addition, for the binaries, CI will also build and publish the documentation using the `gh-pages` branch and a sub-folder `docs/<short-version>`, e.g. https://korpling.github.io/graphANNIS/docs/v0.22/.
