# Release Readiness Audit (2026-02-08)

## Scope

- Repository: `joshka/jk`
- Target: first public release-quality baseline
- Audit method: local source review plus validation commands

Validation run in this pass:

- `cargo fmt --all`
- `cargo check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo package --allow-dirty`

## Current Readiness Summary

`jk` has a strong engineering baseline for development usage, but it is not yet fully release-ready.
Core functionality and tests are healthy. Main remaining gaps are release governance artifacts and
explicit policy decisions.

## What Is In Place

1. Build and test pipeline is stable locally (format, check, tests, clippy pass).
1. Packaging verification succeeds (`cargo package --allow-dirty` compiles packaged source).
1. CI is configured with Rust checks and markdown lint:
   - `.github/workflows/ci.yml`
   - `.github/dependabot.yml`
1. User-facing docs are substantial (`README.md`, ADRs, contributor docs, glossary).

## Release Blockers

1. License is undefined.
   Evidence: no `LICENSE*` file and no `license` or `license-file` in `Cargo.toml`.
   Impact: cannot ship a clear open-source/commercial release contract.
1. Security policy is missing.
   Evidence: no `SECURITY.md`.
   Impact: no disclosure channel or response expectations for vulnerabilities.
1. Changelog/release notes process is missing.
   Evidence: no `CHANGELOG.md` and no release template/process document.
   Impact: hard to publish and communicate stable release deltas.

## High-Priority Gaps (Non-Blocking But Important)

1. Contributor process document is still partial.
   Evidence: snapshot guidance exists, but no unified `CONTRIBUTING.md` covering dev workflow.
1. Cross-platform CI confidence is limited.
   Evidence: CI runs on `ubuntu-latest` only; no macOS/Windows matrix coverage yet.
1. Dependency/vulnerability policy is implicit.
   Evidence: no `cargo audit`/`cargo deny` automation in CI.

## Suggested Next Steps

1. Decide and add license:
   - add `LICENSE` file
   - set `license` (or `license-file`) in `Cargo.toml`
1. Add security policy:
   - create `SECURITY.md` with contact channel and SLA expectations
1. Add release process docs:
   - create `CHANGELOG.md`
   - document versioning and release checklist in `docs/` or `CONTRIBUTING.md`
1. Expand CI coverage:
   - add optional OS matrix job for `cargo check` and smoke tests
   - add scheduled `cargo audit` (or `cargo deny`) job

## Risk If Released Today

Functional risk is moderate-low for developer preview use, but governance/compliance risk is high
because licensing and security-response policy are not yet defined.
