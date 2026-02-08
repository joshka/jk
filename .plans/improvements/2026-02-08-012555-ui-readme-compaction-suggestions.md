# UI + README Improvement Suggestions (Pre-Implementation)

## Context

This file captures improvement suggestions before implementation to reduce drift and keep decisions
explicit while compaction work proceeds.

## Status Key

- `considered`: captured and discussed, not yet accepted for current pass
- `accepted`: approved for implementation in next pass
- `rejected`: intentionally not doing
- `done`: implemented and validated

## Accepted Batch (From Current Thread)

1. `done` - Use muted unified chrome colors (header/footer), likely dark gray + white.
1. `done` - Condense help and keymap views, prefer two-column layout and less repetition.
1. `done` - Remove title/footer duplication where screen/context repeats noisily.
1. `done` - Add narrative captions above media in README so readers know what each item shows.
1. `done` - Refactor README into workflow-focused sections, and move internal details to docs.

## Similar Ideas (New Suggestions Before Implementation)

1. `considered` - Group `:commands` and `:keys` into sections (`Navigation`, `Views`, `Actions`,
   `Safety`) while retaining compact two-column rendering.
1. `considered` - Use one stable emphasis pattern for selected rows (marker + text emphasis),
   and avoid full-row background fills.
1. `considered` - Keep footer mode hints short by default and expand only for `prompt/confirm`.
1. `considered` - Add an optional “minimal hints” mode toggle for power users.
1. `considered` - Replace remaining ASCII underline-style wrapper headers with cleaner title/tip
   format for visual consistency.
1. `considered` - Add one “first 5 minutes” narrative walkthrough in README for devs new to `jj`.
1. `considered` - Add a quick “when to use shell vs jk” table to set usage expectations.
1. `considered` - Add direct links from README workflows to screen docs (`docs/screens.md`) and
   tutorial artifacts (`docs/tutorial-vhs.md`).

## Next Decision Gate

Before implementation starts, decide whether to include any `considered` items in the same batch
as the accepted five or keep them as a follow-up compaction pass.
