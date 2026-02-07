# jk

`jk` is a log-first TUI for `jj`.

The default entrypoint is `jk`, which behaves like `jk log`: it opens a full-screen, pager-style
view and keeps you in one interface for inspect, rewrite, bookmark, and remote flows.

## Goals

- Keep `jj log` as the visual and mental baseline.
- Reuse `jj` config and command semantics wherever possible.
- Avoid box-heavy dashboard UIs; favor focused, command-line-pager interaction.
- Let users stay inside `jk` for common daily workflows.

## Current State

This repository is in active development. The current baseline includes:

- Alt-screen + raw-mode runtime loop.
- Log-first rendering with cursor selection.
- Command mode (`:`), confirmation mode, and prompt mode.
- `jj` subprocess execution via `jj --no-pager ...`.
- Configurable keybinds from `config/keybinds.default.toml` and optional user override.

## Command Entry Model

- `jk` defaults to `log`.
- `jk <command> [args...]` starts in the same TUI and runs/plans that command.
- Commands entered with `:` use the same normalization and safety rules.

## Implemented Flow Coverage (Baseline)

- Read: `log`, `status`, `show`, `diff`.
- Daily mutation: `new`, `describe`, `commit`, `next`, `prev`, `edit`.
- Rewrite/recovery: `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`.
- Recovery extras: `restore`, `revert`.
- Bookmarks: `bookmark list/create/set/move/delete/forget/rename/track/untrack`.
- Remote: `git fetch`, `git push`.

Mutating high-risk commands run through confirmation guards.

`jk` also keeps an explicit top-level command registry aligned to the current `jj` command surface so
new flow work can evolve without ambiguity.

## Alias Coverage

Native aliases:

- `gf`, `gp`, `rbm`, `rbt`

Oh My Zsh `jj` plugin compatibility is included for common aliases such as:

- `jjgf`, `jjgp`, `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`,
  `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, `jja`, `jjrt`

## Development

Build and run:

```bash
cargo run
```

Quality checks:

```bash
cargo fmt --all
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
markdownlint-cli2 "*.md" ".plans/*.md" "docs/**/*.md"
```

## Project Docs

Planning and ADR files live in:

- `.plans/`
- `docs/adr/`

These files are maintained alongside code so implementation context is not lost during long runs.
