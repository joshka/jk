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
- `operation show`, `operation diff`, `operation restore`, `operation revert`: `done` (guided prompts)
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
- Workflow order for each change:
  1. write/update docs first when design context changes;
  2. lint Markdown immediately;
  3. write code;
  4. run `cargo fmt` + `cargo check`;
  5. run targeted tests for changed behavior.
- At each larger checkpoint, run full `cargo test` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
