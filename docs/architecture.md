# Architecture and Internals

This document captures implementation-facing details that are intentionally kept out of the
user-focused README.

## Module Map

- `src/app/`: runtime state, mode transitions, input handling, rendering, and selection logic
- `src/flow/`: command planning, guided prompt contracts, prompt token builders
- `src/commands/`: top-level command registry, safety tier metadata, in-app command overview
- `src/alias/`: alias normalization and alias catalog rendering
- `src/config/`: keybind schema, merge/validation, default + user override loading
- `src/jj.rs`: centralized `jj --no-pager` execution and output capture

## Runtime Model

`App` owns a single session state machine:

1. Start in normal mode with `log` as the default view.
1. Dispatch keys by mode (`normal`, `command`, `prompt`, `confirm`).
1. Plan actions via `flow` for command-mode and shortcut paths.
1. Run `jj` subprocess commands and decorate output with view wrappers.
1. Keep selection, scroll, and row-to-revision mapping in sync for item-based navigation.

## Command Entry Model

- `jk` defaults to `log`.
- `jk <command> ...` runs inside the same TUI, not as a separate one-shot process.
- `:` command mode uses the same planner and safety model as startup command routing.
- Local views (`:commands`, `:keys`, `:aliases`) render without shelling out to `jj`.

## Navigation and Selection Semantics

- Log-like views are item-based (revision-to-revision), not raw rendered-line stepping.
- Paging in log-like views targets viewport rows, then snaps to nearest item boundary.
- Screen history supports back/forward traversal.
- ANSI-colored output is preserved in the content body via `ansi-to-tui`.

## Safety Model Implementation

- Tier-C commands enter confirmation mode before execution.
- Confirm previews are best-effort and do not block confirmation if preview fails.
- High-risk flows include rewrite/mutation paths such as rebase/squash/split/abandon,
  restore/revert, and bookmark/workspace/operation mutation commands.

## Flow Coverage (Current Baseline)

### Read Flows

- `log`, `status`, `show`, `diff`
- `operation log/show/diff`
- `bookmark list`, `resolve -l`, `file list/show/search/annotate`, `tag list`, `workspace root`

### Guided Mutation Flows

- Top-level: `new`, `commit`, `describe`, `edit`, `next`, `prev`
- Rewrite/recovery: `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`, `restore`, `revert`
- File: `track`, `untrack`, `chmod`
- Tag: `set`, `delete`
- Bookmark: `create/set/move/track/untrack/delete/forget/rename`
- Workspace: `add/forget/rename/update-stale`
- Operation: `restore`, `revert`
- Remote: `git fetch`, `git push`

## Alias Coverage

### Native Shortcuts

- `gf`, `gp`, `rbm`, `rbt`
- Core `jj` defaults: `b`, `ci`, `desc`, `op`, `st`

### Oh My Zsh Plugin Compatibility

High-frequency aliases are normalized, including:

- `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`, `jjrs`, `jja`
- `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, `jjrt`
- parity aliases such as `jjbc`, `jjbd`, `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`, `jjgcl`, `jjla`

## Build and Validation

Typical local cycle:

```bash
cargo fmt --all
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
markdownlint-cli2 "*.md" ".plans/*.md" "docs/**/*.md"
```
