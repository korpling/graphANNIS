# Make sure you installed grcov via cargo first and that the llvm tools are available
# 
# cargo install grcov
# rustup component add llvm-tools-preview

# Set some environment variables needed by grcov
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'

mkdir -p target/coverage/

# Run all tests
LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
LLVM_PROFILE_FILE='cargo-ignored-test-%p-%m.profraw' cargo test -- --ignored

# Generate HTML report in target/debug/coverage/index.html
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '*/tests/*' --ignore 'target/*' --ignore '../*' --ignore "/*" -o target/coverage/html/
# Also generate a lcov file for further processing
grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '*/tests/*' --ignore 'target/*'  --ignore '../*' --ignore "/*" -o target/coverage/tests.lcov

# Cleanup
find . -name '*.profraw' -delete
