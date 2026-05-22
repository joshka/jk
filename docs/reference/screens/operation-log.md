# Operation Log Screen

## Purpose

The operation log screen is the recovery and audit surface.

## Source Commands

- `jj operation log`
- with drill-downs to `jj operation show` and `jj operation diff`

## View Model

- primary mode: list of operation items
- drill-down into separate operation detail screens or documents
- optional inline expansion for selected operation metadata when it improves recovery context

## Priority

Priority 1. Operation log is the recovery anchor and should arrive before risky mutation flows grow
too far.

## Core Information

- operation identifiers and timestamps
- enough summary text to identify what happened
- clear relationship to undo, redo, restore, and revert

## Primary Interactions

- move between operation items
- open operation show
- open operation diff
- launch undo
- launch redo
- launch restore
- launch revert
- copy operation ids
- refresh
- go back

## Selection Model

- selection unit: operation item
- operation identity must be exact before any recovery action
- inline detail may expand the selected operation with command args, tags, or affected refs
- recovery actions target one operation at a time

## Interaction Details

- Movement: move by operation item, not graph decoration.
- Drill-down: operation show and operation diff open dedicated detail screens or documents.
- Undo/redo: direct actions should preview or clearly state which operation will be affected.
- Restore/revert: high-risk actions need confirmation and should describe the destination state in
  jj terms.
- Refresh: preserve selected operation id when possible; if it disappears, keep the nearest item and
  report the mismatch.

## Shortcut Candidates

- `j`/`k`, arrows: move operation selection
- `Enter`, `s`, `l`, `Right`: open operation show
- `d`: open operation diff
- `u`: undo flow
- `Ctrl-r`: redo flow
- `r`: refresh
- `y`: copy operation id
- `h`, `Left`: back

## Safety Model

Recovery actions are high-value and potentially confusing. The screen should bias toward explicit
language, previews, and traceable outcomes.

## Integration Notes

The list can start from rendered `jj operation log` output with a narrow operation-id parser.
Restore, revert, and other recovery flows should be designed around `jj` command behavior and should
move toward a harder contract if they need exact operation graph semantics or transaction planning.

The preferred contract exposes operation ids, graph relationships, operation metadata, affected ref
or workspace context, and renderable styled rows together.

## Acceptance Criteria

- users can inspect the operation history without leaving `jk`
- undo/redo feel discoverable and confidence-building
- operation drill-downs preserve context cleanly
- recovery actions use exact operation ids and clear previews
