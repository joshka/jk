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
- ADR scaffolding: `done`

### Foundation implementation

- CLI entrypoint (`jk` == `jk log`): `done`
- Alt-screen + raw mode runtime loop: `done`
- Keybinding system + TOML defaults: `done`
- Command registry and alias normalization: `done` (includes OMZ alias baseline + 44-command registry)
- Log view rendering/parsing bridge: `in-progress`

### Core command flows (Phase 1 target)

- `log`, `status`, `show`, `diff`: `in-progress` (log-first baseline implemented; richer views pending)
- `new`, `describe`, `commit`, `next`, `prev`, `edit`: `done` (selection-aware planning wired)
- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`: `done` (guided flows + danger confirms)
- `bookmark` core subset: `done` (list/create/set/move/track/untrack prompts wired)
- `git fetch`, `git push`: `done` (guided prompts + alias coverage + push confirmation)

### Extended guided flows (Phase 2 seed)

- `restore`, `revert`: `done` (guided prompt defaults wired)
- `bookmark rename`, `bookmark delete`, `bookmark forget`: `done` (guided prompts wired)
- `operation` / `workspace` defaults: `done` (`operation log` and `workspace list`)

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
- Workflow order for each change:
  1. write/update docs first when design context changes;
  2. lint Markdown immediately;
  3. write code;
  4. run `cargo fmt` + `cargo check`;
  5. run targeted tests for changed behavior.
- At each larger checkpoint, run full `cargo test` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
