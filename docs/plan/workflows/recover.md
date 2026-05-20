# Recover Workflow

## Goal

Understand what happened and undo or restore confidently when something goes wrong.

## Likely Commands

Shipped today:

- `operation log`
- `operation show`
- `operation diff`
- `operation restore`
- `operation revert`
- `undo`
- `redo`

Planned follow-ups:

- none currently scheduled in this workflow spec

Passthrough / CLI-first:

- `operation integrate` (specialized recovery-oriented command; executed via regular `jj` for now)

Deferred:

- `operation abandon`

## UI Bias

- operation log is the anchor
- actions should be explicit, contextual, and previewed where practical

## Acceptance Criteria

- recovery is legible
- the user can navigate from “what happened?” to “fix it” cleanly
