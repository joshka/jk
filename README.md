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
- Quick read shortcuts in normal mode: `o` operation log, `L` bookmark list, `v` resolve list,
  `f` file list, `t` tag list, `w` workspace root.
- Help shortcut: `?` opens the command registry directly from normal mode.
- Keymap shortcut: `K` opens the in-app keymap directly from normal mode.
- Alias shortcut: `A` opens the in-app alias catalog directly from normal mode.
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
- `:commands` also annotates top-level `jj` default aliases
  (`bookmark (b)`, `commit (ci)`, `describe (desc)`, `operation (op)`, `status (st)`).
- `:help` mirrors `:commands`; both accept an optional filter (for example `:commands work`).
- The command registry also surfaces local TUI views (`aliases`, `keys`, `keymap`).
- `:aliases` renders an in-app alias catalog and supports filtering (for example `:aliases push`).
- `:keys` renders the active keymap and supports filtering (for example `:keys push`).
- Unfiltered command registry output includes a high-frequency alias hint, discovery tips, and
  group-default hints (for example `resolve -> resolve -l`, `file -> file list`, and
  `operation -> operation log`).
- `status`, `show`, and `diff` use lightweight in-app view headers/shortcuts while preserving
  command output content.
- `show`/`diff` wrappers add section spacing for file headers to improve scanability.
- `status` and `operation log` wrappers include compact summary lines plus section spacing.
- `root` and `workspace root` use a native path-focused wrapper view for quick workspace
  inspection.
- `bookmark list` and `operation log` also use native wrapper headers/tips for faster scanning.
- `workspace list` and `operation show` now use native wrappers with compact summaries and tips.
- `file list` and `tag list` now use native wrappers with compact summaries and empty-state hints.
- `file show`, `file search`, and `file annotate` now use native wrappers with concise summaries.
- `file track`, `file untrack`, and `file chmod` now use native wrappers with mutation-focused
  summaries and follow-up tips.
- `bookmark` mutation subcommands (`create/set/move/track/untrack/delete/forget/rename`) now
  render with native mutation wrappers and verification tips.
- `workspace` mutation subcommands (`add/forget/rename/update-stale`) now render with native
  mutation wrappers and follow-up tips.
- Top-level mutation commands (`new`, `describe`, `commit`, `edit`, `next`, `prev`, `rebase`,
  `squash`, `split`, `abandon`, `undo`, `redo`, `restore`, `revert`) now render with native
  post-action wrappers and command-specific follow-up tips.
  - these wrappers now prefer signal-first summaries (for example `Rebased N commits` or
    `Undid operation ...`) and fall back to output-line counts when no signal line is present.
- `resolve -l` now uses a native wrapper with conflict-count or no-conflicts summary.
- `operation diff` now uses a native wrapper with compact changed-commit summary.
- `operation restore` and `operation revert` now render with native mutation wrappers after
  confirmation.
- `git fetch` and `git push` now use native wrappers with compact summaries and follow-up tips.

## Implemented Flow Coverage (Baseline)

- Read: `log`, `status`, `show`, `diff`.
- Daily mutation: `new`, `describe`, `commit`, `next`, `prev`, `edit`.
- Rewrite/recovery: `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`.
- Recovery extras: `restore`, `revert`.
- Bookmarks: `bookmark list/create/set/move/delete/forget/rename/track/untrack`.
- Remote: `git fetch`, `git push`.
- Command groups: `operation` defaults to `operation log`; `workspace` defaults to
  `workspace list`; `resolve` defaults to `resolve -l`; `file` defaults to `file list`; `tag`
  defaults to `tag list`.
- File read flows: `file list`, `file show`, `file search`, and `file annotate` execute with
  native wrapper rendering.
- File mutation flows: `file track`, `file untrack`, and `file chmod` run as guided prompts.
- Tag mutation flows: `tag set` and `tag delete` are now guided prompts with sensible defaults.
- Operation read flows: `operation show` and `operation diff` execute directly in-app.
- Operation guided prompts: `operation restore`, `operation revert`.
- Workspace guided prompts: `workspace add`, `workspace forget`, `workspace rename`;
  direct actions for `workspace root` and `workspace update-stale`.

Mutating high-risk commands run through confirmation guards.
`git push` confirmation now includes a best-effort `--dry-run` preview in-app when available.
`git fetch` and `git push` output now render through native wrappers instead of raw passthrough
lines.
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
- core `jj` defaults: `b`, `ci`, `op`, `st`, `desc`
- `rbm` defaults to `main` and accepts an optional destination override (for example `rbm release`).
- `rbm`/`rbt` preserve explicit destination flags (for example `rbm -d release` or
  `rbt --to main`) instead of forcing default destinations.

Oh My Zsh `jj` plugin compatibility is included for common aliases such as:

- `jjgf`, `jjgfa`, `jjgp`, `jjgpt`, `jjgpa`, `jjgpd`
- `jjrb`, `jjrbm`, `jjst`, `jjl`, `jjd`, `jjc`, `jjsp`, `jjsq`, `jjrs`, `jja`
- `jjb`, `jjbl`, `jjbs`, `jjbm`, `jjbt`, `jjbu`, `jjrt`
- plus plugin parity aliases including `jjbc`, `jjbd`, `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`,
  `jjgcl`, and `jjla`.

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
