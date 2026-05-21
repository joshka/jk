# Workspace Screen

## Purpose

The workspace screen is a focused read-only utility surface for current root and workspace context.
It helps users answer "where is this jj workspace?" and "which workspaces exist?" without leaving
`jk` or mutating the repository.

## Source Commands

- `jj root`
- `jj workspace list`
- `jj workspace list --template <workspace metadata template>`

The metadata template uses `name`, `target.change_id()`, and `target.commit_id()`. It intentionally
does not use `root` or per-workspace root commands.

## Current Behavior

- Startup entry: `jk workspaces`.
- Global entry: `X` opens workspaces from normal navigation.
- View menu entry: `workspaces`.
- The header shows the current root from `jj root`, or a readable root error if that command fails.
- The list preserves rendered `jj workspace list` rows as the presentation source.
- Exact workspace name, target change id, and target commit id come only from the metadata template.
- Search moves through rendered workspace rows.
- Copy offers the current root, exact selected workspace name when metadata is available, selected
  target change id and commit id when available, and selected row text.
- Refresh preserves the selected exact workspace name when metadata is still available, otherwise it
  clamps by prior index.

## Degradation

Failures are shown in place instead of blocking the entire screen:

- If `jj root` fails, the root header reports that the current root is unavailable.
- If `jj workspace list` fails, the list is empty and the list error is shown.
- If metadata command execution, JSON parsing, or row-count pairing fails, rendered rows remain
  visible but exact workspace name and target copy options are withheld.

## Non-Goals

- No `jj workspace add`.
- No `jj workspace rename`.
- No `jj workspace forget`.
- No `jj workspace update-stale`.
- No worktree manager dashboard.
- No replacement for shell directory navigation.
- No per-workspace root exactness. Copy the current root path separately.

## Selection Model

- Selection unit: rendered workspace row.
- Rendered labels are opaque and are not parsed for exact names.
- Exact workspace identity is optional metadata paired by row order.
- Future workspace actions must use exact metadata or a stronger upstream contract, not rendered row
  text.

## Integration Notes

This packet deliberately avoids `WorkspaceRef.root()` and `jj workspace root --name` because this
checkout has shown that they can render or fail with "Workspace has no recorded path: default".
`jj root` succeeds here and is the exact source for the current workspace root.

The row-order pairing between rendered rows and metadata rows is a soft agreement with
`jj workspace list`. If that pairing fails, `jk` keeps rendered rows visible and withholds exact
metadata.
