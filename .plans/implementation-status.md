# Implementation Status Tracker

## Legend

- `not-started`: no implementation work yet.
- `in-progress`: currently being built/refined.
- `blocked`: waiting on decision or dependency.
- `done`: implemented and validated by tests.

## Current snapshot (2026-02-07)

### Documentation and planning

- Master plan (`.plans/jk-tui-command-plan.md`): `done`
- Research context (`.plans/research-context.md`): `done`
- Per-command deep plan (`.plans/command-deep-plans.md`): `done`
- Gold command detail matrix (`.plans/gold-command-details.md`): `done`
- ADR scaffolding: `done`

### Foundation implementation

- CLI entrypoint (`jk` == `jk log`): `done`
- Alt-screen + raw mode runtime loop: `done`
- Keybinding system + TOML defaults: `done`
- Command registry and alias normalization: `done` (includes OMZ alias baseline + 44-command registry)
- Log view rendering/parsing bridge: `done` (metadata-backed row mapping + fallback parsing)

### Core command flows (Phase 1 target)

- `log`, `status`, `show`, `diff`: `done` (log-first baseline + selection-aware show/diff)
- `new`, `describe`, `commit`, `next`, `prev`, `edit`: `done` (selection-aware planning wired)
- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`: `done` (guided flows + danger confirms)
- `bookmark` core subset: `done` (list/create/set/move/track/untrack prompts wired)
- `git fetch`, `git push`: `done` (guided prompts + alias coverage + push confirmation)

### Extended guided flows (Phase 2 seed)

- `restore`, `revert`: `done` (guided prompt defaults wired)
- `bookmark rename`, `bookmark delete`, `bookmark forget`: `done` (guided prompts wired)
- `operation` / `workspace` defaults: `done` (`operation log` and `workspace list`)
- `operation show`, `operation diff`, `operation restore`, `operation revert`: `done` (direct read
  flows for show/diff, guided prompts for restore/revert)
- `workspace add`, `workspace forget`, `workspace rename`: `done` (guided prompts)

### Testing baseline

- Unit test harness for parsing/alias normalization: `done`
- `insta` visual snapshots for main screens: `done`
- Command assembly tests (`FlowAction`/`PromptKind`): `done`
- Safety-guard tests for Tier C commands: `done`

## Implementation checklist

1. Build runtime loop and rendering pipeline.
2. Add keybinding config loading and default mapping.
3. Implement read-only command flows.
4. Implement mutation flows with preview/confirm safety layers.
5. Add alias compatibility (`gf`/`gp`/`rbm`/`rbt` + OMZ variants).
6. Add tests and snapshots.
7. Validate with `cargo fmt`, `cargo test`, and markdown lint.

## Notes

- Keep this file updated after each meaningful implementation step.
- Every status transition should include tests or clear rationale in commit/body text.
- Latest checkpoint:
  - expanded OMZ alias support and `rbm` default-to-`main` behavior
  - added guided flows for rewrite/recovery, bookmark, and remote commands in `src/flows.rs`
  - retained confirmation gating in `src/app.rs` for Tier C commands
  - added explicit test coverage for dangerous-command gating and alias argument fidelity
  - added explicit top-level `jj` command registry and safety-tier lookup in `src/commands.rs`
  - expanded guided prompts for `restore`, `revert`, and additional bookmark mutations
  - improved revision selection with metadata-backed row mapping for `jj log` output
  - added in-app `:commands` registry view from command metadata
  - routed startup `jk <command>` through the same flow planner used by command mode
  - added startup regression tests for confirm-gated and render-only startup actions
  - added confirmation preview rendering with `git push --dry-run` support
  - added filtered command registry lookup via `:commands <query>` and `:help <query>`
  - added guided operation subcommand prompts and operation/workspace safety overrides
  - added guided workspace prompts with direct handling for `workspace root` and `update-stale`
  - added confirmation previews for `operation restore` and `operation revert`
  - aligned command registry modes so `operation`/`workspace`/`restore`/`revert` report guided
    coverage
  - expanded alias and flow tests for OMZ high-frequency variants (`jjgfa`, `jjgpt`, `jjgpa`,
    `jjgpd`, `jjst`, `jjl`)
  - documented command-registry parity and safety routing in `docs/adr/0003-command-registry-parity.md`
  - broadened Tier C confirmation previews to cover rebase/squash/split/abandon/restore/revert,
    bookmark mutations, and undo/redo operation context
  - added Tier C fallback preview to operation log when a command has no custom preview renderer
  - improved `rbm`/`rbt` alias normalization to support optional destination overrides without
    duplicate destination args
  - added default normal-mode shortcuts for high-frequency remote/rebase flows (`F`, `P`, `M`,
    `T`)
  - added default normal-mode action shortcuts (`n`, `c`, `D`, `b`, `a`) for new/commit/describe,
    bookmark set, and abandon flows
  - added log patch toggle shortcut (`p`) with argument-preserving `log` patch on/off behavior
  - added normal-mode rewrite/recovery shortcuts (`B`, `S`, `X`, `O`, `R`, `u`, `U`) that route
    through existing prompt/confirm safety flows
  - added `s` status shortcut and sectioned status rendering for improved in-app scanability
  - surfaced high-frequency alias hints directly in unfiltered `:commands`/`:help` output
  - switched `:` command parsing to shell-style tokenization with invalid-quote status feedback
  - added native header/shortcut wrappers for `show` and `diff` outputs to improve scanability
  - added normal-mode `?` shortcut that opens the in-app command registry/help view
  - added command-mode history navigation (`Up`/`Down`) with draft restoration
  - added normal-mode `.` shortcut to re-run the last executed command
  - added native wrapper view for `root` output with workspace-path-focused presentation
  - added native wrapper views for `bookmark list` and `operation log` outputs
  - added normal-mode `]`/`[`/`e` shortcuts for `next`/`prev`/`edit` revision actions
  - added explicit gold-set flow contract test covering all core commands in one matrix
  - added explicit OMZ gold-alias flow contract test matrix for high-frequency compatibility
  - added `insta` snapshot coverage for bookmark-list and operation-log wrapper views
  - added quick read-mode shortcuts (`o`, `L`, `w`) for operation log/bookmark list/root views
  - aligned `workspace root` rendering with the native root wrapper presentation
  - added in-app `:aliases` catalog view with optional query filtering
  - improved `show`/`diff` wrapper rendering with automatic section spacing between file blocks
  - added in-app `:keys` keymap view with query filtering and startup support (`jk keys`)
  - added normal-mode `K` shortcut for direct keymap access
  - added `:commands` tips that point to `:aliases` and `:keys` discovery views
  - added normal-mode `A` shortcut for direct alias-catalog access
  - refined `status`/`operation log` wrappers with compact summaries and section-spacing rules
  - added `insta` snapshot coverage for the updated status wrapper presentation
  - surfaced local discovery views (`aliases`, `keys`, `keymap`) in the `:commands` catalog
  - added native wrapper views for `workspace list` and `operation show` output
  - added `insta` snapshot coverage for workspace-list and operation-show wrapper presentation
  - added native wrapper views for `file list` and `tag list` output
  - added `insta` snapshot coverage for file-list and tag-list wrapper presentation
  - defaulted `file` and `tag` command groups to `file list` and `tag list` flows
  - defaulted `resolve` command group to `resolve -l` for list-first conflict inspection
  - switched `operation show` and `operation diff` to direct read execution without prompt
  - added `tag` short-subcommand canonicalization (`l`/`s`/`d`) in flow planning
  - refined command safety mapping for read-only `file`/`tag` subcommands
  - refined command safety mapping for read-only `resolve -l`
  - surfaced command-group default hints in `:commands` unfiltered tips
  - added normal-mode quick-read shortcut for `resolve -l` (`v`)
  - added normal-mode quick-read shortcuts for `file list` (`f`) and `tag list` (`t`)
  - added native wrapper view and snapshot coverage for `resolve -l` output
  - added native wrapper view and snapshot coverage for `operation diff` output
  - added native wrapper views and coverage tests for `git fetch` and `git push` output
  - added `insta` snapshot coverage for `git fetch` and `git push` wrapper views
  - added native wrapper views and coverage for `file show`, `file search`, and `file annotate`
    output
  - added `insta` snapshot coverage for `file show`, `file search`, and `file annotate` wrappers
  - added guided prompt flows for `file track`, `file untrack`, and `file chmod`
  - added native wrapper views and tests for `file track`, `file untrack`, and `file chmod`
    output
  - added `insta` snapshot coverage for `file track`, `file untrack`, and `file chmod` wrappers
  - added guided prompt flows for `tag set` and `tag delete` with selected-revision defaults
    for tag set
  - aligned command-registry metadata so `file` and `tag` now report `guided` mode
  - added native wrapper views for `bookmark` mutation flows
    (`create/set/move/track/untrack/delete/forget/rename`)
  - added native wrapper views for `workspace` mutation flows
    (`add/forget/rename/update-stale`)
  - added native wrapper views for `operation restore` and `operation revert`
  - added native wrapper views for top-level mutation commands:
    `new`, `describe`, `commit`, `edit`, `next`, `prev`, `rebase`, `squash`, `split`, `abandon`,
    `undo`, `redo`, `restore`, `revert`
  - added targeted decorator/render tests and snapshots for
    `bookmark set`, `workspace add`, and `operation restore`
  - added targeted decorator/render tests and snapshots for
    top-level `commit` and `rebase` wrapper rendering
  - refined `rbm`/`rbt` alias normalization to preserve explicit destination flags (`-d`/`--to`)
    without forcing default destinations
  - expanded alias-catalog parity with the installed OMZ `jj` plugin aliases (`jjbc`, `jjbd`,
    `jjbf`, `jjbr`, `jjcmsg`, `jjdmsg`, `jjgcl`, `jjla`) and added direct catalog coverage tests
  - reran full validation checkpoint with green `markdownlint-cli2`, `cargo fmt`, `cargo check`,
    `cargo test` (156 passed), and strict `cargo clippy`
- Workflow order for each change:
  1. write/update docs first when design context changes;
  2. lint Markdown immediately;
  3. write code;
  4. run `cargo fmt` + `cargo check`;
  5. run targeted tests for changed behavior.
- At each larger checkpoint, run full `cargo test` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
- Latest pass:
  - added native wrapper views for high-frequency mutation outputs:
    `bookmark` mutations, `workspace` mutations, and `operation restore/revert`.
  - added targeted decorator/render tests and `insta` snapshots for:
    `bookmark set`, `workspace add`, and `operation restore` wrapper rendering.
- Latest pass:
  - added native wrappers for top-level daily/rewrite mutation commands so command results stay in
    the same scannable in-app format as the rest of the gold command set.
- Latest pass:
  - added broad regression tests ensuring every top-level mutation command routes through a native
    wrapper.
  - added command-specific tip assertions for `commit`, `undo`, `rebase`, and `next` wrapper
    output.
- Latest pass:
  - refined top-level mutation wrapper summaries to prefer signal-first action lines (for example
    rebase/undo/restore results) before falling back to generic output-line counts.
  - updated top-level `commit` and `rebase` snapshots to lock in signal-first summary rendering.
- Latest pass:
  - added a gold-command wrapper matrix regression test over
    `status/show/diff`, top-level mutation commands, bookmark core commands, and `git fetch/push`
    wrapper headers.
  - updated the gold command detail contract to require this wrapper matrix coverage.
