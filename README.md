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
- High-frequency normal-mode shortcuts: `s` status, `F` fetch, `P` push, `M` rebase to main,
  `T` rebase to trunk.
- Quick read shortcuts in normal mode: `o` operation log, `L` bookmark list, `w` workspace root.
- Help shortcut: `?` opens the command registry directly from normal mode.
- Repeat shortcut: `.` re-runs the last executed command in-place.
- Log shortcut: `p` toggles `--patch` for the active log command arguments.
- Action shortcuts in normal mode: `n` new, `c` commit, `D` describe selected change, `b` bookmark
  set for selected change, `a` abandon selected change.
- Navigation/action shortcuts in normal mode: `]` next, `[` prev, `e` edit selected change.
- Rewrite/recovery shortcuts in normal mode: `B` rebase selected, `S` squash selected, `X` split
  selected, `O` restore into selected, `R` revert selected, `u` undo, `U` redo.

## Command Entry Model

- `jk` defaults to `log`.
- `jk <command> [args...]` starts in the same TUI and runs/plans that command.
- Commands entered with `:` use the same normalization and safety rules.
- `:` command parsing supports shell-style quoting for multi-word arguments.
- Command mode supports history navigation with `Up`/`Down`.
- `:commands` renders an in-app command registry with mode/tier coverage.
- `:help` mirrors `:commands`; both accept an optional filter (for example `:commands work`).
- Unfiltered command registry output also includes a high-frequency alias hint line.
- `status`, `show`, and `diff` use lightweight in-app view headers/shortcuts while preserving
  command output content.
- `root` and `workspace root` use a native path-focused wrapper view for quick workspace
  inspection.
- `bookmark list` and `operation log` also use native wrapper headers/tips for faster scanning.

## Implemented Flow Coverage (Baseline)

- Read: `log`, `status`, `show`, `diff`.
- Daily mutation: `new`, `describe`, `commit`, `next`, `prev`, `edit`.
- Rewrite/recovery: `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`.
- Recovery extras: `restore`, `revert`.
- Bookmarks: `bookmark list/create/set/move/delete/forget/rename/track/untrack`.
- Remote: `git fetch`, `git push`.
- Command groups: `operation` defaults to `operation log`; `workspace` defaults to `workspace list`.
- Operation guided prompts: `operation show`, `operation diff`, `operation restore`,
  `operation revert`.
- Workspace guided prompts: `workspace add`, `workspace forget`, `workspace rename`;
  direct actions for `workspace root` and `workspace update-stale`.

Mutating high-risk commands run through confirmation guards.
`git push` confirmation now includes a best-effort `--dry-run` preview in-app when available.
`operation restore`/`operation revert` confirmation includes an operation summary preview.
Rewrite, recovery, and bookmark Tier `C` flows also render targeted log/show previews before
confirmation when enough command context is available.
Unhandled Tier `C` commands fall back to a short `operation log` preview.

`jk` also keeps an explicit top-level command registry aligned to the current `jj` command surface so
new flow work can evolve without ambiguity.
Log-row selection uses metadata-backed revision mapping to stay stable across multi-line log views.

## Alias Coverage

Native aliases:

- `gf`, `gp`, `rbm`, `rbt`
- `rbm` defaults to `main` and accepts an optional destination override (for example `rbm release`).

Oh My Zsh `jj` plugin compatibility is included for common aliases such as:

- `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`, `jjrs`, `jja`
- `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, `jjrt`

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
