# Refs And Workspaces Workflow

## Goal

Manage bookmarks, tags, files, and workspaces as focused utility tasks rather than as the main app
model.

## Likely Commands

Shipped today:

- bookmark list
- `bookmark set`
- `bookmark create`
- `bookmark move`
- `bookmark rename`
- `bookmark delete`
- `bookmark forget`
- `bookmark track`
- `bookmark untrack`
- file list/show
- resolve

Planned follow-ups:

- tag list/set/delete
- file search/annotate/track/untrack/chmod
- workspace root/list/add/rename/forget/update-stale

## UI Bias

- focused utility screens
- actions launched from the relevant utility context
- minimal chrome and no dashboard framing
- remote-tracking mutations are exact bookmark-row actions, not parsed `name@remote` labels

## Acceptance Criteria

- common maintenance tasks have coherent homes
- utility breadth does not displace the core log/show/diff/status/op-log loop
