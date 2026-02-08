# UX Screen Flow Overhaul (2026-02-08)

## Goal

Make `jk` navigation obvious, fast, and aligned with common `jj` workflows:

1. Read history and working-copy state quickly.
1. Jump between screens without context loss.
1. Execute high-frequency mutate/recover flows with clear, mode-aware hints.

## Inputs Used

1. Existing `README.md` day-one and CLI-flow guidance.
1. VHS command/tape docs on GitHub (`charmbracelet/vhs`).
1. Ratatui release/app presentation guidance and examples.

## Work Plan

1. Help/commands UX
   1. Fix spacing/readability issues in `:commands` output.
   1. Put day-one flows first; keep full command registry below.
   1. Add snapshot tests for help layout.
1. Navigation semantics
   1. Ensure log up/down always moves by revision item (not text line).
   1. Add explicit screen back/forward navigation and discoverability.
   1. Add tests for graph-row fallback and history traversal.
1. Documentation coverage
   1. Create one screen reference doc covering every screen and interaction mode.
   1. Add UX checklists and flow guidance for power users.
1. VHS assets
   1. Add tape files for key flows.
   1. Render GIFs to `target/vhs/` and embed in docs.
1. Codify guardrails
   1. Update `AGENTS.md` with UX standards, snapshot discipline, and VHS workflow.

## Deliverables

1. Improved in-app help layout and command flow guidance.
1. Screen history navigation (`back`/`forward`) with keybindings and hints.
1. Log item-navigation fixes with coverage tests.
1. `docs/screens.md` as the canonical screen reference.
1. `docs/vhs/*.tape` plus rendered `target/vhs/*.gif`.
1. Updated `AGENTS.md` with UX-focused future-session guidance.
