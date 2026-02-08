# Directional UX + Docs Summary

## Context

This note captures the current direction for `jk` based on recent session prompts.

## Recent Prompt Themes

1. Make navigation terminal-native and obvious (`j/k`, arrows, paging, back/forward).
1. Keep log movement item-based (revision-to-revision), not rendered-line based.
1. Prioritize common workflows first in help and keymap output.
1. Reduce visual noise: muted chrome, less repetition, cleaner hierarchy.
1. Replace ASCII-heavy wrappers with cleaner text presentation.
1. Improve docs narrative: explain why `jk` helps, not only what exists.
1. Keep tutorial artifacts visual and practical (GIFs for dynamic, screenshots for static).
1. Encode planning/process discipline in `.plans` with status lifecycle.
1. Keep work split into focused `jj` changes when concerns differ.
1. Lock UX behavior with targeted tests and `insta` snapshots.

## Direction Statement

Target an opinionated, low-friction command cockpit for daily `jj` usage: inspect fast,
act safely, verify quickly, and recover confidently.

## Candidate Improvements

1. `done` Add a compact quick-actions strip in log view for selected revision intents.
1. `done` Show back-stack context in footer when screen history exists.
1. `done` Add workflow-scoped help presets (`:help inspect/rewrite/sync/recover`).
1. `done` Add session-local recent-intent shortcuts in help/command mode.
1. `done` Keep one primary next action visible per screen (single-line hint).
1. `done` Rank command-palette suggestions by frequency and recency.
1. `done` Provide dry-run-first paths for high-risk flows where possible.
1. `done` Add first-run onboarding that auto-hides after completing one full loop.
1. `done` Add workflow docs that pair each flow with one screenshot, one GIF, and one sentence.
1. `done` Add workflow-grouped UX regression snapshots (`inspect`, `rewrite`, `sync`, `recover`).

## Normal Developer Flow Narrative

Start in `log`, move by revision item with `j/k`, and open details with `Enter` and `d`.
Use `Left` to return and continue scanning with context preserved.
When ready, run mutation flows (`D`, `B/S/X`, `a`) with confirmation safeguards.
Check `status`, run `F`/`P` for sync, and verify results.
If needed, open `operation log` and use undo/redo recovery.

Core loop: `inspect -> act -> verify -> recover`.
