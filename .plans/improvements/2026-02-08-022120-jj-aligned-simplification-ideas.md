# JJ-Aligned Simplification Ideas (2026-02-08 02:21:20)

## Context

Recent UX feedback: `native / guided / tier` wording is not intuitive. The goal is to keep `jk`
closer to how `jj` users already think and speak, with simpler language and defaults.

## Status Key

- `considered`: captured and ready for selection.
- `accepted`: approved for implementation.
- `rejected`: intentionally not doing.
- `done`: implemented and validated.

## Accepted Batch

1. `done` - Replace user-facing `mode/tier` labels in `:commands` with plain behavior labels:
   `runs now`, `opens prompt`, `runs as jj`, and `asks confirmation`.
2. `done` - Simplify glossary wording to behavior-first terms rather than internal categories.
3. `done` - Remove remaining user-facing "tier" wording in README/screens docs.

## Similar Ideas (10 Other Simplifications)

1. `considered` - Show the exact `jj ...` command next to each key action in `:commands`.
2. `considered` - Split command help into `Daily` and `Advanced` sections, collapsed by default.
3. `considered` - Rename `:help inspect|rewrite|sync|recover` to `:help review|edit|sync|undo`.
4. `considered` - Add "default target" notes (for example: selected revision) for each action.
5. `considered` - In confirm mode, show only `command`, `impact`, and `y/N` (remove extra text).
6. `considered` - Make `:commands` query match `jj` aliases first (`st`, `ci`, `desc`, `op`).
7. `considered` - Add a one-line `jj` mental model at top of help:
   "pick revision -> run action -> verify in status/log".
8. `considered` - Mirror `jj status` section order in status screen summaries.
9. `considered` - Replace generic action verbs with `jj` verbs (`describe`, `rebase`, `squash`).
10. `considered` - Add context-sensitive next-step hints that include exact `jj` command names.

## Next Decision Gate

Choose up to 3 items from "Similar Ideas" to move to `accepted` for the next implementation
batch.
