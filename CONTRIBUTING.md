# Contributing To `jk`

Thanks for working on `jk`.

This document is the contributor-facing entry point. It points to the product,
architecture, workflow, and documentation references that shape changes in this
repository.

## Before You Start

- Read [`AGENTS.md`](AGENTS.md) for repository-wide guidance.
- Read [`docs/product-direction.md`](docs/product-direction.md) if your change
  affects user-visible scope, navigation, or product direction.
- Read [`docs/agent/architecture.md`](docs/agent/architecture.md) if your
  change affects ownership, command execution, view behavior, rendering,
  navigation, search, copy, or terminal lifecycle.

## Development Commands

Use the repository `just` commands:

- `just check`: run formatting, Markdown checks, `cargo check`, `cargo test`,
  `cargo clippy -- -D warnings`, and the largest Rust file report.
- `just packet-check`: run the clippy gate and largest Rust file report.
- `just largest-rust-files`: print the top 20 Rust source files by line count.
- `just fmt`: run `rustup run nightly cargo fmt`.
- `just md-fmt`: run `panache format README.md AGENTS.md docs`.
- `just md-check`: run Panache format and lint checks for Markdown.
- `just test`: run `cargo test`.
- `just run`: run the TUI with `cargo run`.

For Markdown-only changes, run `just md-check`. For Rust changes, run
`rustup run nightly cargo fmt` before finishing and `just check` when
practical.

## Repository References

- [`docs/reference/README.md`](docs/reference/README.md) for the current
  product-facing screens, workflows, and view model.
- [`docs/agent/architecture.md`](docs/agent/architecture.md) for the
  implementation ownership map.
- [`docs/agent/workflow.md`](docs/agent/workflow.md) for the implementation
  workflow and completion bar.
- [`docs/agent/documentation.md`](docs/agent/documentation.md) for README,
  Rustdoc, and truthfulness guidance.
- [`docs/agent/testing.md`](docs/agent/testing.md) for validation expectations.
- [`docs/development/rules/README.md`](docs/development/rules/README.md) for
  the copied shared rule index when repo-local guidance is not enough.

## Version Control

This repository uses Jujutsu. Prefer `jj --no-pager` commands for normal
version-control work, and use Git only for transport-level operations that `jj`
does not cover in the current workflow.

For separable work, start in a fresh working-copy change and describe it early:

```sh
jj --no-pager new
jj --no-pager desc --message "Describe the change

Explain the purpose and context in a wrapped body when needed."
```

Keep changes small, atomic, and scoped to one purpose when practical.
