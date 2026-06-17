set shell := ["bash", "-uc"]

default:
    @just --list

fmt:
    # rustfmt.toml uses unstable rustfmt options for comment wrapping and import grouping.
    scripts/reflow-rust-comments.py crates
    cargo +nightly fmt --all

fmt-comments:
    scripts/reflow-rust-comments.py crates

fmt-comments-check:
    scripts/reflow-rust-comments.py --check crates

fmt-check:
    # Keep check mode on the same toolchain as fmt so CI and local formatting agree.
    scripts/reflow-rust-comments.py --check crates
    cargo +nightly fmt --all -- --check

check:
    cargo check --workspace --all-targets

test:
    cargo test --workspace

betamax:
    cargo run --manifest-path ../betamax/Cargo.toml -p betamax -- run tapes/jk-log.tape

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

udeps:
    cargo +nightly udeps --workspace --all-targets

doc:
    cargo doc --workspace --no-deps

lint-md:
    markdownlint-cli2 "**/*.md"

package:
    cargo package --workspace --allow-dirty --no-verify

build-release target:
    cargo build --locked --release --target "{{target}}" -p jk

package-release-archive target version:
    scripts/package-release-archive.sh "{{target}}" "{{version}}"

install-smoke:
    tmp="$(mktemp -d)"; \
    trap 'rm -rf "$tmp"' EXIT; \
    CARGO_HOME="$tmp/cargo-home" \
    CARGO_TARGET_DIR="$tmp/target" \
    cargo install --path crates/jk --locked --root "$tmp/install"; \
    "$tmp/install/bin/jk" --version || "$tmp/install/bin/jk"

rust-release-check: fmt-check check test clippy udeps doc package install-smoke

release-check: rust-release-check lint-md
