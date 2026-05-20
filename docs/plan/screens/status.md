# Status Screen

## Purpose

The status screen is the working-copy triage surface.

## Source Command

- `jj status`

## View Model

- likely single active read surface
- possible inline section expansion for details
- no permanent fixed sibling pane by default

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
- search, if useful on final output shape
- copy relevant file or revision identifiers
- jump to related file-oriented views
- launch `jj git fetch`
- launch push flow
- return to log

## Selection Model

Start with section-aware scrolling. Add row or file selection only when actions need to attach to
specific changed paths, conflicts, bookmarks, or next-action prompts.

Likely selectable units:

- changed file item
- conflict item
- bookmark or remote-sync signal
- suggested next action

## Interaction Details

- Scan: the first screenful should communicate clean/dirty state, conflicts, and high-signal next
  actions.
- Section navigation: jumping between sections is more useful than raw line paging once status has
  structured groups.
- File actions: file track, untrack, restore, chmod, and file drill-down should require exact paths
  or fileset data, not prose parsing.
- Sync entry: fetch and push flows can launch here when status shows remote or bookmark context, but
  they should use guided previews rather than raw command execution.
- Fetch: `jj git fetch` is common and low-risk enough to be a direct action when the remote/default
  command shape is clear. It should show failure/output clearly and refresh the current screen after
  completion.
- Push: push is higher risk than fetch. It should use a preview or confirmation that states the
  destination and selected bookmark/revision context.
- Refresh: refresh should preserve the active section or selected path when possible.

## Shortcut Candidates

- `j`/`k`, arrows: move or scroll
- `[`/`]`: previous/next section
- `/`, `n`, `N`: search
- `l`, `Right`: open file list
- `f`: fetch
- `p`: push flow
- `y`: copy selected path or identifier
- `r`: refresh
- `h`, `Left`: back

## Integration Notes

Use rendered `jj status` output first. A selection model over files or sections should be added only
after the screen proves it needs actions attached to precise working-copy state. If file actions
become mutation-capable, prefer structured state or `jj_lib` over broad parsing of status prose.

The preferred contract exposes working-copy cleanliness, changed path groups, conflict signals,
bookmark/sync hints, and renderable status sections together.

## Entry Points

- dedicated status shortcut
- direct startup via `jk status`
- command mode is deferred until that surface exists
- return point after mutation flows

## Open Design Choice

The status screen may remain mostly scroll-oriented rather than having a strong row selection model.
If actions attach naturally to groups or files, then selection may become worthwhile. That should be
earned by the final interaction shape, not assumed upfront.

## Acceptance Criteria

- the user can answer “what changed locally?” quickly
- the screen points naturally to the next action
- it does not become a cluttered task dashboard
- mutation-capable actions never depend on loosely parsed status prose
