# jk Unified TUI Plan (jk-as-log, jj-parity)

## Summary

`jk` is a single TUI application, not a CLI/TUI split. Running `jk` enters the log view by default,
matching `jj log` output and config semantics. Running `jk <command>` keeps users in TUI and routes
to an inline action or focused flow for that command.

This plan is built from `jj --help` plus per-command help across the full top-level surface
(44 commands) and discovered subcommands (59). The implementation prioritizes daily workflows while
keeping a complete command-disposition map so the tool can expand without ambiguity.

## Product Contract

### Command entry model

- `jk` is equivalent to `jk log`.
- `jk log` opens the same default view as `jk`.
- `jk <command> [args...]` enters TUI with that command preselected and arguments prefilled.
- No `jk tui` namespace exists.

### In-session command model

- User lives in the log view, navigates revisions/changes, selects context, then acts.
- Commands run as TUI flows when interaction materially improves safety or speed.
- Commands can run as direct `jj` passthrough when no additional UX is needed.

### Shortcut aliases in TUI

- Core short aliases:
  - `gf` => `git fetch` flow.
  - `gp` => `git push` flow.
  - `rbm` => rebase current branch onto `main` (or `main@origin` fallback).
  - `rbt` => rebase current branch onto `trunk()`.
- Oh My Zsh `jj` compatibility aliases are also accepted in command mode and normalized:
  - `jjgf` -> `gf`, `jjgp` -> `gp`, `jjrb` -> `rebase`.
  - `jjrbm` (plugin meaning: `rebase -d "trunk()"`) -> `rbt`.
  - `jjst` -> `status`, `jjl` -> `log`, `jjd` -> `diff`, `jjc` -> `commit`.
  - `jjgfa`, `jjgpt`, `jjgpa`, `jjgpd` map to fetch/push variants with equivalent flags.
- Shell-only alias behavior:
  - `jjrt` (plugin: `cd "$(jj root || echo .)"`) is mapped to an in-app `root` action that shows
    root path and offers copy/open, but does not attempt shell `cd`.

## UX and Interaction

### Log-first layout

- Full-screen pager-style revision view, minimal chrome.
- No persistent panel grid, box-heavy layout, or visual diagram UI.
- Transient overlays only: command palette, preview/confirm, error details.

### Modes and key precedence

- `Normal` mode (default): vim-style motion and action keys.
- `Command` mode (`:`): command entry with completion.
- `Input` mode: text fields for args/messages/revsets.
- `Confirm` mode: explicit approval screens for risky ops.

Key precedence rules:

1. If a text input is focused, text editing keys win.
2. Else if overlay is open, overlay bindings win.
3. Else normal-mode bindings apply.
4. `Esc` always backs out one layer and never mutates state.

Default navigation keys:

- `j`/`k`: next/previous visible row.
- `h`/`l`: collapse/expand detail or move focus within transient views.
- `gg`/`G`: top/bottom.
- `Enter`: inspect selected rev (`show`).
- `d`: diff selected rev.
- `:`: command prompt.

## Architecture

### Runtime model

- Single process TUI event loop.
- Command runner executes `jj --no-pager ...` subprocesses.
- Stdout/stderr streamed to in-app buffers.
- All command invocations recorded in a command history pane/log for traceability.

### Selection and identity mapping

- Visible log rendered from user-facing `jj log` output.
- Parallel metadata query uses machine template to map row -> change/revision IDs.
- Selection is bound to stable IDs, not raw line numbers, to survive refreshes.

### Command specification registry

Define a `CommandSpec` table for all supported commands:

- name and aliases (`gf`, `gp`, `rbm`, `rbt`, plus `jj` plugin compatibility forms)
- argument schema and defaults
- required context (selected rev, working copy, remote, bookmark)
- execution mode (`direct`, `guided_flow`, `danger_flow`, `external_editor_required`)
- preview policy (`none`, `summary`, `full_command_preview`)
- confirmation policy (`none`, `single`, `explicit_phrase`)
- alias normalization source (`native`, `omz_jj`, `custom`) for telemetry and docs generation

## Safety model

- Tier A (read-only): execute immediately.
- Tier B (normal mutations): single confirmation when command can abandon/move content.
- Tier C (history rewrite / remote push): full command preview + impact summary + explicit confirm.

Tier C commands include at minimum:

- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`, `restore`, `revert`.
- `git push` and bookmark-moving operations that affect remotes.

## Full `jj` Command Coverage Plan

This is the decision-complete disposition for top-level `jj` commands.

### Phase 1: native TUI flows (core daily)

- `log`, `status`, `show`, `diff`, `new`, `describe`, `commit`, `next`, `prev`, `edit`.
- `bookmark list`, `bookmark set`, `bookmark move`, `bookmark track`, `bookmark untrack`.
- `git fetch`, `git push` (with `gf`/`gp` and `jjgf`/`jjgp` compatibility aliases).
- `rebase` (`rbm` and `rbt` presets), `squash`, `split`, `abandon`, `undo`, `redo`.

### Phase 2: guided flows (common but advanced)

- `restore`, `revert`, `duplicate`, `parallelize`, `interdiff`, `evolog`, `metaedit`.
- `operation log`, `operation show`, `operation diff`, `operation restore`, `operation revert`.
- `bookmark create`, `bookmark rename`, `bookmark delete`, `bookmark forget`.
- `workspace list`, `workspace add`, `workspace forget`, `workspace rename`, `workspace root`.

### Phase 3: passthrough-in-TUI first, native later

- `resolve`, `diffedit`, `fix`, `sparse *`, `file *`, `tag *`, `sign`, `unsign`.
- `git clone`, `git import`, `git export`, `git init`, `git remote *`, `git colocation`, `git root`.
- `config *`, `util *`, `gerrit upload`, `operation integrate`, `operation abandon`.
- `bisect run`.

Passthrough-in-TUI means:

- command still launched from `jk` command prompt,
- output and errors displayed in `jk`,
- no shell escape required,
- richer custom flow can replace it incrementally.

## jj-Like Dependency Baseline

Adopt a crate baseline that mirrors jj ecosystem choices where relevant:

- CLI/arg model: `clap`.
- Terminal I/O: `crossterm`.
- Graph/pager ergonomics: `sapling-renderdag`, `sapling-streampager`.
- Interactive hunk selection/edit support: `scm-record`.
- Serialization/config: `serde`, `toml`, `toml_edit`.
- Errors and tracing: `thiserror`, `tracing`, `tracing-subscriber`.
- Text/path handling used broadly in jj: `bstr`, `regex`, `itertools`, `indexmap`.

Testing stack baseline:

- Snapshot/visual assertions: `insta`.
- CLI assertions: `assert_cmd`.
- Scenario/property tests where useful: `test-case`, `proptest`.

## Milestones

1. **Foundation and command registry**
   - Build TUI shell, mode system, command prompt, command runner.
   - Implement `CommandSpec` registry with alias mapping and safety tiers.
2. **Log-native core**
   - Implement `jk`/`jk log` default experience with row-to-revision mapping.
   - Add inspect/diff/status flows and vim navigation.
3. **Daily mutation workflows**
   - Implement `new`, `describe`, `commit`, `next`, `prev`, `edit`.
   - Implement Tier C previews for rewrite commands.
4. **Remote/bookmark productivity**
   - Add `gf`, `gp`, `rbm`, `rbt` and command-palette integration.
   - Add Oh My Zsh compatibility parsing for common `jj*` aliases.
   - Add safe push/fetch and bookmark actions with conflict/error handling.
5. **Long-tail coverage and passthrough lift**
   - Add Phase 2 guided flows.
   - Add Phase 3 passthrough wrappers, then progressively replace with native flows.

## Tests and Acceptance

### Required tests

- Snapshot tests proving default log view parity with `jj log` under custom templates.
- Visual interaction snapshots for key overlays and confirmation flows (`insta`).
- Command assembly tests for every `CommandSpec` variant and alias normalization path.
- Safety tests ensuring Tier C commands never execute without explicit confirmation.
- End-to-end tests for `gf`, `gp`, `rbm`, and `rbt`.
- End-to-end alias-compat tests for `jjgf`, `jjgp`, `jjrb`, `jjrbm`, `jjst`, and `jjrt`.

### Acceptance criteria

- `jk` opens directly to log view and behaves as `jk log`.
- User can stay inside `jk` for common daily workflows without shelling out.
- Vim-style navigation is default with deterministic mode precedence.
- Alias commands (`gf`, `gp`, `rbm`, `rbt`) and OMZ compatibility aliases are documented in help.
- All 44 top-level `jj` commands have an explicit disposition in the registry.
- Core acceptance checks are automated via:
  - `markdownlint-cli2 "*.md" ".plans/*.md" "docs/**/*.md"`
  - `cargo fmt --all --check`
  - `cargo check`
  - targeted `cargo test` for changed modules
  - checkpoint `cargo clippy --all-targets --all-features -- -D warnings`

## Locked defaults and assumptions

- No separate `jk tui` command will exist.
- `jk` is the primary entrypoint and default log view command.
- Risky rewrite and remote operations always require preview + explicit confirm.
- Remote defaults follow jj behavior (`origin`/tracked semantics) unless user overrides.
- Command compatibility is scoped to semantic parity, not byte-for-byte output parity.
- Plan and status files are re-read at each checkpoint and updated before further implementation.
