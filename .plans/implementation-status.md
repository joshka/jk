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
- Command registry and alias normalization: `in-progress`
- Log view rendering/parsing bridge: `in-progress`

### Core command flows (Phase 1 target)

- `log`, `status`, `show`, `diff`: `in-progress` (baseline implemented; richer views pending).
- `new`, `describe`, `commit`, `next`, `prev`, `edit`: `in-progress` (dedicated flows pending).
- `rebase`, `squash`, `split`, `abandon`, `undo`, `redo`: `in-progress` (Tier C gate in place).
- `bookmark` core subset: `in-progress` (routing works; dedicated UI pending).
- `git fetch`, `git push`: `in-progress` (aliases + push confirmation baseline implemented).

### Testing baseline

- Unit test harness for parsing/alias normalization: `done`
- `insta` visual snapshots for main screens: `done`
- Command assembly tests (`CommandSpec`): `not-started`
- Safety-guard tests for Tier C commands: `in-progress`

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
- Workflow order for each change:
  1. write/update docs first when design context changes;
  2. lint Markdown immediately;
  3. write code;
  4. run `cargo fmt` + `cargo check`;
  5. run targeted tests for changed behavior.
- At each larger checkpoint, run full `cargo test` and
  `cargo clippy --all-targets --all-features -- -D warnings`.
