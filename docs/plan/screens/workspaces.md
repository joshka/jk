# Workspace Screens

## Purpose

Workspace screens expose workspace-related state without making multi-workspace management the
center of the app.

## Source Commands

- `jj root`
- likely `jj workspace list`
- related guided flows: add, rename, forget, update-stale

## View Model

- root may be a simple informational screen
- workspace list, if added, should be a small utility list screen

## Priority

Priority 3. Workspace screens should remain low-frequency utility surfaces unless multi-workspace
use proves central for `jk`.

## Primary Interactions

- inspect current workspace root
- inspect available workspaces if supported
- launch add/rename/forget/update-stale flows
- return to log or previous context

## Selection Model

- root screen: no selection needed
- workspace list: selection unit is workspace item
- workspace actions require exact workspace name, path, and stale/current state

## Interaction Details

- Root: show workspace root and related repo path information as a simple document or info view.
- List: show available workspaces as a list when native support is worthwhile.
- Actions: add, rename, forget, and update-stale are guided flows with confirmation where they can
  detach or change a workspace.
- Refresh: preserve selected workspace name when possible.
- Return: workspace screens should return to the previous context, usually log or status.

## Shortcut Candidates

- `j`/`k`, arrows: move workspace selection where applicable
- `Enter`: inspect selected workspace
- `a`: add flow
- `R`: rename flow
- `d`: forget flow
- `u`: update-stale flow
- `y`: copy workspace path or name
- `r`: refresh
- `h`, `Left`: back

## Integration Notes

Use rendered root/list output for inspection. Workspace actions should use exact workspace names and
paths from structured data or `jj_lib` before becoming mutation-capable.

## Acceptance Criteria

- workspace state is easy to inspect when needed
- workspace management remains a utility concern, not a global app frame
- workspace actions make their target path/name explicit before running
