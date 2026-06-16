set shell := ["bash", "-uc"]

default:
    @just --list

fmt:
    cargo +nightly fmt

fmt-check:
    cargo +nightly fmt --check

check:
    cargo check --workspace --all-targets

test:
    cargo test --workspace

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
