set dotenv-load

default: check

fmt:
    cargo +nightly fmt

md-fmt:
    panache format README.md AGENTS.md docs

md-check:
    panache format --check README.md AGENTS.md docs
    panache lint --check README.md AGENTS.md docs

check: fmt md-check
    cargo check
    cargo test

test:
    cargo test

run:
    cargo run
