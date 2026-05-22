# Workflow Reference

This document groups `jk` behavior by user workflow rather than by `jj` namespace. Workflows are the
right level for deciding what deserves native UI support, what should stay CLI-first, and where a
new action belongs.

## Inspect

Goal: understand history, change content, file content, and current repo state.

Shipped today:

- log
- show
- diff
- status
- file list/show
- bookmark list
- resolve
- operation log

Still CLI-first:

- file search/annotate
- tag list

Workflow bias:

- highest priority
- mostly read surfaces
- should set the quality bar for navigation, search, copy, and refresh

## Rewrite

Goal: reshape change history intentionally.

Shipped today:

- `new`
- `edit`
- `next`
- `prev`
- `commit`
- `describe`
- `rebase`
- `squash`
- `abandon`
- `restore`
- `revert`
- `absorb`

Still CLI-first:

- `split`
- `duplicate`

Intentionally not promoted:

- `diffedit`
- `arrange`

Passthrough commands:

- `metaedit`
- `parallelize`
- `simplify-parents`

Workflow bias:

- guided flows, not broad command mirrors
- `jj new trunk` is common, low-risk, and easy to undo, so it can be more direct than the wider
  rewrite flows
- strong previews and confirmations for risky actions
- graph context should stay visible in the mental model even when the action launches from
  prompt/confirm flows

## Sync

Goal: exchange state with remotes and ref tracking surfaces.

Shipped today:

- `git fetch`
- `git push`
- `bookmark set`
- `bookmark create`
- `bookmark move`
- `bookmark delete`
- `bookmark track`
- `bookmark untrack`

Still CLI-first:

- host-specific or dashboard-style remote management

CLI-first commands:

- `bookmark advance`

Workflow bias:

- attach sync actions to status and bookmark-related surfaces
- `jj git fetch` is common and low-risk enough to be a direct action with clear output and refresh
- previews matter more than command breadth
- keep host-specific integration out until the generic sync loop feels solid

## Recover

Goal: undo mistakes and inspect repository operation history.

Shipped today:

- `op log`
- `op show`
- `op diff`
- `undo`
- `redo`
- `operation restore`
- `operation revert`

Intentionally not promoted:

- `operation abandon`

`operation integrate` remains CLI-first because it is specialized and does not justify native UI
yet.

Workflow bias:

- second only to inspect in importance
- deserves strong, explicit UI language
- should make dangerous repair actions feel visible and reversible

## Refs And Workspace Hygiene

Goal: manage bookmarks, tags, files, and workspaces as focused utility tasks rather than as the main
app model.

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

Still CLI-first:

- tag list/set/delete
- file search/annotate
- workspace add/rename/forget/update-stale

Workflow bias:

- useful utility screens
- actions launched from the relevant utility context
- minimal chrome and no dashboard framing

## CLI-First Workflow

Goal: preserve breadth without committing to native UI too early.

Likely commands:

- `interdiff`
- `metaedit`
- `parallelize`
- `simplify-parents`
- `fix`
- `config`
- `sparse`
- `sign`
- `unsign`

Workflow bias:

- supported through regular `jj` CLI invocation outside `jk`
- native promotion requires evidence of frequency and clear `jk` value-add

## Intentionally Deferred

Commands intentionally kept CLI-first:

- `gerrit`
- `util`
