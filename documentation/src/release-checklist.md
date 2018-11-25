# Release checklist

This is a list of steps needed to perform a new graphANNIS release.
A release includes the Rust-based core graphANNIS library, but also the
language bindings for Java and Python.

GraphANNIS uses [semantic versioning](https://semver.org/) and the version numbers for the core and the language bindings should be synchronized.
This means that if a language bindings adds a feature, that has been released in the core library before but was not covered by the binding, also the core library might need to release an update with the matching version number (even if it is not releasing any new features by itself).

## Core library release

1. Make a new **release branch** `release/<version>` either from the `develop` branch for feature releases. If you make a bug-fix release create a branch named `hotfix/<version>` from the `master` branch .
2. **Update version** information, by 
   - changing the `version` field in the `Cargo.toml` file
   - running `cargo build` to update your local `Cargo.lock` file (even if this file is not committed to Git)
3. **Test** with `cargo test` and eventually some manual tests.
4. **Test publishing** with `cargo publish --dry-run --allow-dirty`
5. **Commit and push**, wait for Continuous Integration to finish
6. Close the corresponding **GitHub milestone** and remember its ID
7. Update the **`CHANGELOG.md`** file by executing `./misc/changelog.py <milestone-id>` and pasting the result into the changelog
8. **Tag and push** the latest commit with the prefix `v`, e.g. `v1.4.0`, **merge** the release branch both into the `master` and `develop` branch.
9. Publish to **crates.io** with `cargo publish`
10. Create the **release on GitHub**, copy the changelog entry as release notes. Save the release as draft
11. Wait for Continuous Integration to finish building the release artifacts for all systems and then **publish the drafted release**
