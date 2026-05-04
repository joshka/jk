# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 binary crate for `jk`, a ratatui-based TUI for `jj`.
Source lives in `src/`; tests are colocated in each module under `#[cfg(test)]`.
The code is organized by vertical slices where practical:

- `app.rs` owns the terminal event loop, key dispatch, modal state, and view stack.
- `graph.rs`, `show.rs`, and `diff.rs` own their view behavior, rendering, and tests.
- `jj.rs` builds `jj` commands and parses rendered CLI output.
- `rendered_jj.rs` and `sticky_file_view.rs` handle rendered jj output and
  sticky file context.
- `tui.rs` contains shared title/status/modal chrome only.

## Build, Test, and Development Commands

- `just check`: format with nightly rustfmt, then run `cargo check` and `cargo test`.
- `just fmt`: run `cargo +nightly fmt`.
- `just test`: run `cargo test`.
- `just run`: run the TUI with `cargo run`.

Use `cargo +nightly fmt` before finishing changes. The project uses
`rustfmt.toml` from this repo.

## Coding Style & Naming Conventions

Prefer feature-oriented modules over horizontal buckets. Use named files such
as `graph.rs` and `show.rs`; avoid `mod.rs` re-export layers unless there is a
strong reason.

Write Rustdoc/module comments for durable intent: jj CLI compatibility,
navigation policy, sticky scroll behavior, and other non-obvious constraints.
Avoid comments that restate simple code. Keep visibility narrow. Do not
introduce `pub(crate)`, `pub(super)`, or `pub(in ...)` unless there is a
concrete need and no cleaner local structure.

## Testing Guidelines

Use Rust unit tests colocated with the module they describe. Prefer
behavior-oriented test names, for example
`document_search_wraps_without_reselecting_current_line`.

Use inline insta snapshots for multi-line rendered/projection transitions. Run
`cargo test` for focused validation and `just check` before handing off.

## Commit & Pull Request Guidelines

This repository uses jujutsu. Prefer `jj --no-pager` commands for
version-control inspection. Commit descriptions should be imperative and
concise, for example `Scaffold jj TUI`.

Pull requests should summarize user-visible behavior, note jj command/config
assumptions, and list the validation run. Include terminal screenshots only for
meaningful TUI rendering changes.

## Architecture Notes

`jk` intentionally shells out to `jj` and treats rendered jj output as
canonical. This preserves user config, templates, colors, graph symbols, and jj
CLI behavior. Navigation should prefer change ids from graph rows; commit ids
are exposed for copying.
