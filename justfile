set shell := ["bash", "-uc"]

betamax := env_var_or_default("BETAMAX", "betamax")

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

betamax: betamax-log betamax-diff

readme-media: readme-log-media readme-diff-media

readme-log-media:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh --readme-media tapes/readme-log.tape

readme-diff-media:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh --readme-media tapes/readme-diff.tape

betamax-log:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/jk-log.tape

betamax-diff:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/jk-diff.tape

betamax-release-smoke:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/release-smoke.tape

betamax-action-menu:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/action-menu.tape

betamax-action-menu-demo:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/action-menu-demo.tape

betamax-workspace-lifecycle:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/workspace-lifecycle.tape

betamax-abandon-confirmation:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/abandon-confirmation.tape
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/abandon-confirmation-long.tape

betamax-restore-preview:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/restore-preview.tape

betamax-external-command:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/external-command-mode.tape

betamax-cancellable-refresh:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/cancellable-refresh.tape

betamax-design:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/design-proof-normal.tape
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/design-proof-narrow.tape
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/design-proof-light.tape

# Exercise the integrated workflows in isolated local fixtures, never this checkout's graph.
betamax-workflows:
    BETAMAX="{{betamax}}" bash scripts/run-betamax.sh tapes/workflows-integration.tape

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
