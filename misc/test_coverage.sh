# Make sure you installed grcov via cargo first
# cargo install grcov

# Set some environment variables needed by grcov
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off"

# Run all tests
cargo +nightly clean
cargo +nightly test
cargo +nightly test -- --ignored

# Generate HTML report in target/debug/coverage/index.html
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/
