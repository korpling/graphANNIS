cargo llvm-cov clean --workspace
cargo llvm-cov --no-report --release
cargo llvm-cov --no-report --release -- --ignored
cargo llvm-cov report --ignore-filename-regex '(tests?\.rs)|(capi/.*) --release --open