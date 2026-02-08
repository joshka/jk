# Help Grouping + Mode Badge Improvement Suggestions (Pre-Implementation)

## Context

Follow-up accepted scope for the current UX/docs pass.

## Status Key

- `considered`: captured and discussed, not yet accepted for current pass
- `accepted`: approved for implementation in next pass
- `rejected`: intentionally not doing
- `done`: implemented and validated

## Accepted Batch (From Current Thread)

1. `done` - Add sectioned help/keymap grouping (`Navigation`, `Views`, `Actions`,
   `Safety`) while keeping two-column output.
1. `done` - Replace remaining ASCII wrapper headings (`===`) with cleaner text-only
   section labels.
1. `done` - Add a subtle mode badge in the header using the same muted chrome palette.
1. `done` - Add a "first 5 minutes in jk" walkthrough block to README with one concrete
   end-to-end example.
1. `done` - Update tutorial docs to display GIFs inline with narrative for each scenario.

## Similar Ideas (New Suggestions Before Implementation)

1. `considered` - Add one-line intent labels in help sections (for example "use this when triaging
   working copy").
1. `considered` - Add a compact "Back/Forward history" subsection in help with `Left/Right` and
   `Ctrl+o/i` side by side.
1. `considered` - Split keymap output into `normal`, `command`, and `confirm` subsection anchors.
1. `considered` - Add a `:help workflow` query shortcut that prefilters to day-one flows.
1. `considered` - Add a tiny shell-vs-jk decision table in README near the walkthrough.

## Next Decision Gate

After implementation and validation, mark accepted items as `done` and decide whether to carry any
`considered` items into the next timestamped batch.
