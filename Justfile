set dotenv-load

default: check

fmt:
    cargo +nightly fmt

check: fmt
    cargo check
    cargo test

test:
    cargo test

run:
    cargo run
