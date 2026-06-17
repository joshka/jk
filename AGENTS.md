# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 workspace for `jk`, a log-first terminal UI for Jujutsu. The root
`Cargo.toml` is workspace-only. The publishable crates live under `crates/`:

- `crates/jk`: binary crate and current default workspace member.
- `crates/jk-cli`, `crates/jk-core`, `crates/jk-tui`: reserved library boundaries.

Release and CI automation lives in `.github/`, `release-plz.toml`, `cliff.toml`, `deny.toml`, and
`scripts/package-release-archive.sh`. Short-term planning notes live in `.plans/`.

## Build, Test, and Development Commands

Use `just` for local tasks:

```sh
just --list
just check
just test
just clippy
just udeps
just lint-md
just release-check
```

`just release-check` is the broad local gate: formatting, check, tests, clippy, unused dependency
checks, docs, packaging, install smoke, and Markdown lint. `just build-release <target>` and
`just package-release-archive <target> <version>` support release asset testing.

## Coding Style & Naming Conventions

Run `cargo +nightly fmt` before finishing Rust changes. Keep Rust module names lowercase with
underscores. Prefer clear, small modules over broad utility buckets. Avoid `unsafe`; the workspace
forbids it. Markdown uses `markdownlint-cli2`, 100-character prose, and aligned tables.

## Testing Guidelines

Use Rust unit tests close to the module they describe. Name tests by behavior, for example
`refresh_keeps_selected_change_when_still_visible`. Run focused tests while editing and
`just release-check` before release-oriented changes.

## Commit & Pull Request Guidelines

Use plain imperative commit summaries, not conventional commits. Pull requests should explain
user-visible behavior, list validation run, and call out release or crates.io impact. Link related
issues when they exist. Include terminal screenshots only for meaningful TUI rendering changes.

## Security & Release Notes

Do not add long-lived crates.io tokens to workflows. The crates are bootstrapped; future publishing
should use crates.io trusted publishing through the `release-plz.yml` workflow and `release`
environment.
