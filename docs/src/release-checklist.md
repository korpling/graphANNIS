# Release checklist

This is a list of steps needed to perform a new graphANNIS release.
A release includes the Rust-based core graphANNIS library, but also the
language bindings for Java and Python.

GraphANNIS uses [semantic versioning](https://semver.org/) and the version numbers for the core and the language bindings should be synchronized.
This means that if a language bindings adds a feature, that has been released in the core library before but was not covered by the binding, also the core library might need to release an update with the matching version number (even if it is not releasing any new features by itself).

## Core library release

1. Create and push a new **release branch** `release/<version>` from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch.
2. If this is a major or minor release, **update the link to the Developer Guide** documentation in the `README.md` to point to the new version short version, omitting the patch level and with adding a "v" prefix (e.g. https://korpling.github.io/graphANNIS/docs/v1.0/)
3. **Release** the branch with the [cargo-release plugin](https://crates.io/crates/cargo-release)
   - `cargo release patch` for hotfixes updates (e.g. 1.0.0 to 1.0.1)
   - `cargo release minor` for minor updates (e.g. 1.0.1 to 1.1.0)
   - `cargo release major` for major updates (e.g. 1.1.0 to 2.0.0)
4. **Merge** the release branch both into the master and develop branch then delete the release branch.
5.  Create the **release on GitHub**, copy the changelog entry as release notes. Save the release as draft
6.  Wait for Continuous Integration to finish building the release artifacts for all systems and then **publish the drafted release**

## Python wrapper release

1. Make a new **release branch** `release/<version>` from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch.
2. **Update version** information, by 
    - changing the `version` field in the `setup.py` file
    - specifying the corresponding graphANNIS release tag in the `GRAPHANNIS_VERSION` environment variable in `.travis.yml`
    - committing the changed files
3. **Download** release artifacts from the core library: `./package/download-release-binaries.sh <version-tag>` 
4.  **Test** with 
    - `python3 -m unittest`
    - `./doctest_runner.py`
5. **Tag and push** the latest commit with the prefix `v`, e.g. `v1.4.0`, **merge** the release branch both into the `master` and `develop` branch then delete the release branch.

Continuous Integration will automatically deploy all released versions on the `master` branch.

## Java wrapper release

1. **Start** the release process with `mvn gitflow:release-start` or `mvn gitflow:hotfix-start`
2. **Download** release artifacts from the core library: `./misc/download-release-binaries.sh <version-tag>`
3. **Test** with `mvn test`
4. **Finish** the release process with `mvn gitflow:release-finish` or `mvn gitflow:hotfix-finish`
5. **Release** the closed staging repository to Maven Central with the Nexus interface: [https://oss.sonatype.org/](https://oss.sonatype.org/)
