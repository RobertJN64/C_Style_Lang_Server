@RD /S /Q "cov"
set RUSTFLAGS=-Cinstrument-coverage
set LLVM_PROFILE_FILE=cov/coverage-%%p-%%m.profraw
cargo +nightly test

grcov cov --binary-path ./target/debug/deps -s . -t html --branch --ignore-not-existing -o cov

start cov/html/index.html