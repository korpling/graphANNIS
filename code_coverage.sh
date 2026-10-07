#!/bin/bash

# Stop the script if any command exits with a non-zero return code
set -e

export LC_COLLATE="en_US.utf8"

# Execute tests and calculate the code coverage as HTML report
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report --release
cargo llvm-cov --no-report --release --tests -- --ignored
cargo llvm-cov report --ignore-filename-regex '(tests?\.rs)|(capi/.*)' --release --html --open
