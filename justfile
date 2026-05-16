# market-flow development tasks
#
# Install tooling (once):
#   cargo install cargo-audit

default:
    @just --list

audit:
    cargo audit

fmt:
    cargo fmt --all

lint:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings

build:
    cargo build --all-targets

run *ARGS:
    cargo run --example read_feed -- {{ARGS}}

test:
    cargo test

# Criterion benchmarks (slower; HTML report under target/criterion/)
bench:
    cargo bench
