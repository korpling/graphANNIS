# Release checklist

This is a list of steps needed to perform a new graphANNIS release.
A release includes the Rust-based core graphANNIS library, but also the
language bindings for Java and Python.

GraphANNIS uses [semantic versioning](https://semver.org/) and the version numbers for the core and the language bindings should be synchronized.
This means that if a language bindings adds a feature, that has been released in the core library before but was not covered by the binding, also the core library might need to release an update with the matching version number (even if it is not releasing any new features by itself).

## Core library release

1. Make a new **release branch** `release/<version>` from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch.
2. **Update version** information, by 
   - changing and committing the `version` field in the `Cargo.toml` file
   - changing and committing the link to the developer guide in `README.md`
   - running `cargo build` to update your local `Cargo.lock` file (even if this file is not committed to Git)
3. **Update BOM.txt** by executing `cargo bom > BOM.txt`
4. **Test** with `cargo test` and eventually some manual tests.
5. **Test publishing** with `cargo publish --dry-run --allow-dirty`
6. Close the corresponding **GitHub milestone** and remember its ID
7. Update and commit the **`CHANGELOG.md`** file by executing `./misc/changelog.py <milestone-id>` and pasting the result into the changelog
8. **Tag** the latest commit with the prefix `v`, e.g. `v1.4.0`, **merge** the release branch both into the `master` and `develop`, branch then delete the release branch and **push** all changes.
9. Publish the checked out `master` branch to **crates.io** with `cargo publish`
10. Create the **release on GitHub**, copy the changelog entry as release notes. Save the release as draft
11. Wait for Continuous Integration to finish building the release artifacts for all systems and then **publish the drafted release**

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

1. **Start** the release process with `mvn jgitflow:release-start`
2. **Download** release artifacts from the core library: `./misc/download-release-binaries.sh <version-tag>`
3. **Test** with `mvn test`
4. **Finish** the release process with `mvn jgitflow:release-finish`
5. **Close and Release** the staging repository to Maven Central with the Nexus interface: [https://oss.sonatype.org/](https://oss.sonatype.org/)
