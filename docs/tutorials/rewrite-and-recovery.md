# Rewrite And Recovery

## Prerequisites

- A `jj` repository with a small stack of changes.
- For the recovery examples, run `just demo-setup` and `cd target/demo-repos/operation-recovery`.
- For absorb, use a repo where the selected exact source and destination rows make the preview
  appear; the flow is intentionally narrow.

## Describe And Commit

- `D` opens a description prompt from graph rows and from status.
- Graph `D` uses the selected exact change id.
- Status `D` targets `@`.
- `C` opens a commit prompt.
- Commit always targets `@`; it does not take the selected graph row as an argument.
- Both flows preview the exact command before they run.

## Edit And Navigate Working Copy

- `e` edits the selected exact graph revision.
- `]` moves to the next editable change from `@`.
- `[` moves to the previous editable change from `@`.
- The next/previous flows are based on the current working copy, not the highlighted graph row.

## Abandon

- `a` opens the action menu on an exact graph row.
- Choose `abandon` from the menu when exact graph context allows it.
- Empty changes use a lighter confirmation path.
- Non-empty changes use a stronger confirmation path.
- If the selected row is not exact, `jk` keeps the action unavailable instead of guessing.

## Squash, Rebase, And Absorb

- Use the action menu on an exact graph row for rewrite flows.
- `squash` and `rebase` are preview-first and keep the command shape visible.
- `absorb` is exact-target only and stays deliberately narrow.
- These flows are for intentional rewrite, not for general patch editing.

## Restore And Revert

- `restore` and `revert` appear only where a view already has exact graph-derived revision
  provenance.
- `restore` is also available from file-list/file-show only when the selected path is exact.
- `revert` is available for exact revisions only, not from status or resolve.
- Review the preview carefully before confirming; the flow is explicit about its target.

## Operation Recovery

- Press `O` to open the operation log.
- Use `s`, `l`, `Right`, or `Enter` for operation show.
- Use `d` for operation diff.
- Use `u` for undo.
- Use `Ctrl-r` for redo.
- Operation recovery is the place to inspect `jj` history after a mutation.
