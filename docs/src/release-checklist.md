# Release checklist

This is a list of steps needed to perform a new graphANNIS release.

GraphANNIS uses [semantic versioning](https://semver.org/).
Language bindings can have their own version number, but should state which core version is used in its documentation.

1. Create and push a new **release branch** `release/<version>` from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch.
2. Make sure the **third-party license file** is up-to-date by executing [cargo-about](https://crates.io/crates/cargo-about): `cargo about generate about.hbs > THIRD-PARTY.html` and committing any changes.
3. If this is a major or minor release, **update the link to the Developer Guide** documentation in the `README.md` to point to the new version short version, omitting the patch level and with adding a "v" prefix (e.g. https://korpling.github.io/graphANNIS/docs/v1.0/)
4. **Release** the branch with the [cargo-release plugin](https://crates.io/crates/cargo-release)
   - `cargo release release` to release the current development version (e.g. 1.0.0-dev will be released as 1.0.0)
   - `cargo release patch` for hotfixes updates (e.g. 1.0.0 to 1.0.1)
   - `cargo release minor` for minor updates (e.g. 1.0.1 to 1.1.0)
   - `cargo release major` for major updates (e.g. 1.1.0 to 2.0.0)
5. **Merge** the release branch both into the master and develop branch then delete the release branch.
6.  Create the **release on GitHub**, copy the changelog entry as release notes. Save the release as draft
7.  Wait for Continuous Integration (CI) to finish building the release artifacts for all systems and then **publish the drafted release**
8.  Update and release languagage bindings 
    -  [Python](https://github.com/korpling/graphANNIS-python#release-process)
    -  [Java](https://github.com/korpling/graphANNIS-java#release-process)

In addition for the binaries, CI will also build and publish the documentation using the gh-pages branch and a sub-folder `docs/<short-version>`, e.g. https://korpling.github.io/graphANNIS/docs/v0.22/.
