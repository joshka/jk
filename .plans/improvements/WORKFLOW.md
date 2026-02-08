# Improvement Suggestion Workflow

Use this workflow for UI/UX/docs improvement requests before implementation starts.

## File Naming

Create a timestamped file per suggestion batch:

- `.plans/improvements/YYYY-MM-DD-HHMMSS-<slug>.md`

## Required Sections

1. Context
1. Status key
1. Accepted batch
1. Similar ideas
1. Next decision gate

## Status Lifecycle

Use these status values only:

- `considered`
- `accepted`
- `rejected`
- `done`

Transition guidance:

1. Start all new ideas as `considered`.
1. Move to `accepted` after explicit user confirmation.
1. Move to `done` only after implementation + validation.
1. Move to `rejected` when intentionally dropped.

## Execution Discipline

1. Write suggestion file first.
1. Confirm accepted scope.
1. Implement.
1. Update statuses in the same file.
1. Link resulting changes/snapshots/docs.

## Compaction Rule

For context-constrained sessions, prefer:

1. one small accepted batch,
1. one deferred list of considered items,
1. one follow-up timestamped file when scope changes.
