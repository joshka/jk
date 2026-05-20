set dotenv-load

default: check

fmt:
    rustup run nightly cargo fmt

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

demo-setup:
    sh docs/demo/setup-demo-repo.sh static-log
    sh docs/demo/setup-demo-repo.sh operation-recovery

demo-static-log: demo-setup
    cargo build --quiet
    vhs docs/demo/static-log.tape

demo-operation-recovery: demo-setup
    cargo build --quiet
    vhs docs/demo/operation-recovery.tape

demo: demo-setup
    cargo build --quiet
    vhs docs/demo/static-log.tape
    vhs docs/demo/operation-recovery.tape
