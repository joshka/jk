# Status Screen

## Purpose

The status screen is the working-copy triage surface.

## Source Command

- `jj status`

## View Model

- single active read surface
- no permanent fixed sibling pane
- exact-path actions only on rows whose local row model proves one tracked repo-relative path

## Priority

Priority 1. Status is the daily triage screen after the core log/show/diff loop. It should answer
what changed locally and route the user to the next focused action.

## Core Information

- working-copy cleanliness or dirtiness
- changed file groups
- conflict or resolution signals
- high-signal next actions

## Primary Interactions

- refresh
- search across rendered status rows
- copy relevant file or revision identifiers
- jump to related file-oriented views
- launch `jj git fetch`
- launch push flow
- return to log

## Selection Model

- selection unit: rendered status row
- exact-path file actions activate only on rows whose local row model proves one tracked
  repo-relative path
- non-file rows remain visible, searchable, copyable, and selectable even when they do not expose a
  mutation-capable target

## Interaction Details

- Scan: the first screenful should communicate clean/dirty state, conflicts, and high-signal next
  actions.
- Section awareness: rendered status groups still matter for scanning, but movement stays row-based
  because shipped actions attach to exact rows.
- File actions: file track, untrack, restore, chmod, and file drill-down should require exact paths
  or fileset data, not prose parsing.
- Shipped file hygiene: status `?` rows offer guided track; tracked `M`/`A`/`D`/`!` rows offer
  guided untrack where an exact clean path is parsed; chmod is offered only for tracked non-deleted
  rows where status distinguishes a path that should exist.
- Sync entry: fetch and push flows can launch here when status shows remote or bookmark context, but
  they should use guided previews rather than raw command execution.
- Fetch: `jj git fetch` is common and low-risk enough to be a direct action when the remote/default
  command shape is clear. It should show failure/output clearly and refresh the current screen after
  completion.
- Push: push is higher risk than fetch. It uses a preview or confirmation that states the
  destination and selected bookmark/revision context.
- Refresh: refresh should preserve the active section or selected path when possible.

## Shipped Bindings

- `j`/`k`, arrows, `PageUp`, `PageDown`: move by rendered status row
- `g`/`G`, `Home`, `End`: first or last row
- `l`, `Right`: open the file list view
- `a`: open the action menu when the selected row has an exact path action target
- search, copy, refresh, back, fetch, push, and help come from shared app bindings

## Integration Notes

Use rendered `jj status` output first. A selection model over files or sections should be added only
after the screen proves it needs actions attached to precise working-copy state. If file actions
become mutation-capable, prefer structured state or `jj_lib` over broad parsing of status prose.

The preferred contract exposes working-copy cleanliness, changed path groups, conflict signals,
bookmark/sync hints, and renderable status sections together.

## Entry Points

- dedicated status shortcut
- direct startup via `jk status`
- return point after mutation flows

## Non-Goals

- no task-dashboard framing
- no mutation-capable actions built from loosely parsed status prose
- no broad row model beyond the exact targets the shipped actions need

## Acceptance Criteria

- the user can answer “what changed locally?” quickly
- the screen points naturally to the next action
- it does not become a cluttered task dashboard
- mutation-capable actions never depend on loosely parsed status prose
