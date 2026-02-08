# jk

`jk` is a full-screen companion for `jj` that keeps your day-to-day flow in one place.

If you already use `jj`, you know the rhythm: run a command, inspect output, run another command,
repeat. `jk` keeps that rhythm but removes the shell ping-pong so history, status, rewrite actions,
and recovery tools stay in one interface.

## Why This Exists

`jj` is powerful, but context-switching between many small commands can slow down common work.
`jk` gives you a log-first home screen where you can inspect, act, and verify without losing
context.

## What You Get

- A log-first home view that behaves like `jj log`.
- Item-based navigation for commit rows, not line-by-line noise.
- Fast key-driven workflows for inspect, rewrite, sync, and recovery.
- Confirmation gates for high-risk actions.
- Exact-command escape hatch with `:` when you want raw precision.

## Quick Start

Prerequisites:

- Rust + `cargo` in `PATH`
- `jj` in `PATH`
- Terminal with alt-screen and ANSI color support

Run:

```bash
cargo run
```

Then:

1. Move through revisions with `j/k` (or arrows).
1. Press `Enter` for `show` and `d` for `diff`.
1. Press `:` and run `status`.
1. Press `q` to quit.

## First 5 Minutes In `jk`

Concrete example: review your current stack, inspect one revision, then prep a safe push.

1. Press `l` to ensure you are on log home.
1. Use `j`/`k` to select the revision you want to inspect.
1. Press `Enter` for `show`, then press `d` for `diff`.
1. Press `Left` to go back to your previous screen.
1. Press `s` for status and confirm your working copy is clean.
1. Press `P` to start push flow and review the confirmation preview before accepting.

If you only remember one loop, make it this one:
`log -> inspect -> back -> status -> push preview`.

## A Day With `jk`

1. Start in `log` and scan your stack quickly.
1. Jump into details with `show`/`diff`.
1. Apply rewrite actions (`D`, `B`, `S`, `X`, `a`) with safety prompts.
1. Check `status`, run `F`/`P` for remote sync, and verify.
1. If needed, inspect `operation log` and recover with `undo`/`redo`.

This keeps the feedback loop tight: inspect -> act -> verify.

## Workflow Guide

You can open scoped in-app workflow help with:
`:help inspect`, `:help rewrite`, `:help sync`, `:help recover`.

### Inspect History Fast

- `l` / `:log`: home timeline
- `Enter` / `:show <rev>`: inspect one revision
- `d` / `:diff -r <rev>`: inspect patch
- `PageUp`/`PageDown` and `Ctrl+u/d`: move by viewport

### Rewrite With Safety

- `n`, `c`, `D`: create/commit/describe flows
- `B`, `S`, `X`, `a`: rebase/squash/split/abandon flows
- High-risk commands are confirmation-gated and previewed

### Sync and Recover

- `s`: working-copy status
- `F` / `P`: fetch/push prompt flows
- `o`, `u`, `U`: operation log and undo/redo loop

## Tutorial Gallery

These media files are generated locally into `target/vhs/`:

```bash
docs/vhs/render.sh
```

### Static Screens (What The App Looks Like)

Log home, where most work starts:

![log](target/vhs/static-log.png)

Status view, for working-copy triage:

![status](target/vhs/static-status.png)

Help/command registry, for discoverability:

![help](target/vhs/static-help.png)

Keymap view, for exact binding lookup:

![keys](target/vhs/static-keys.png)

### Dynamic Flows (How Work Moves)

Revision navigation -> show/diff -> return:

![navigation](target/vhs/tutorial-dynamic-navigation.gif)

Command mode + paging + history traversal:

![command/history](target/vhs/tutorial-dynamic-command-history.gif)

Prompt and confirm safety behavior:

![safety](target/vhs/tutorial-dynamic-safety.gif)

Remote sync + operation follow-through:

![remote/ops](target/vhs/tutorial-dynamic-remote-ops.gif)

Full 25-item tutorial catalog: `docs/tutorial-vhs.md`.
Scenario authoring rules: `docs/vhs/scenarios.md`.

## Safety Model

- Tier-C rewrite/mutation commands require explicit confirmation.
- `git push` preview is shown when available.
- Dangerous tutorial captures use cancel/reject paths by default.

## Current Limits

- View switching is command/key driven, not tab/window driven.
- This project is still evolving; some long-tail `jj` flows are intentionally passthrough.

## Learn More

- Workflow narratives: `docs/workflows.md`
- Screen behavior reference: `docs/screens.md`
- Navigation checklist: `docs/navigation-behavior-checklist.md`
- Terminology: `docs/glossary.md`
- Release readiness audit: `docs/release-readiness-audit-2026-02-08.md`

## For Contributors

- Tests and snapshots: `docs/contributing-tests.md`
- Architecture and internals: `docs/architecture.md`
- Security policy: `SECURITY.md`
- Changelog: `CHANGELOG.md`

## License

Dual-licensed under either:

- Apache License, Version 2.0 (`LICENSE-APACHE`)
- MIT license (`LICENSE-MIT`)
