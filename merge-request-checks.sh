#!/bin/bash

# Stop the script if any command exits with a non-zero return code
set -e

# Run static code checks
cargo fmt --check
cargo clippy

# Execute tests and calculate the code coverage both as lcov and HTML report
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report --release
cargo llvm-cov --no-report --release --tests -- --ignored
mkdir -p target/llvm-cov/
cargo llvm-cov report --ignore-filename-regex '(tests?\.rs)|(capi/.*)' --release --lcov --output-path target/llvm-cov/tests.lcov

# Use diff-cover (https://github.com/Bachmann1234/diff_cover) and output code coverage compared to main branch
mkdir -p target/llvm-cov/html/
diff-cover target/llvm-cov/tests.lcov --html-report target/llvm-cov/html/patch.html
echo "HTML report available at $PWD/target/llvm-cov/html/patch.html"
