# Workflow Matrix

This document groups work by user workflow rather than by `jj` namespace. Workflows are the right
level for deciding what deserves native UI support.

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

Planned follow-ups:

- file search/annotate
- tag list
- workspace root

Planning bias:

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

Planned follow-ups:

- `split`
- `duplicate`
- `operation restore`
- `operation revert`

Deferred:

- `diffedit`
- `arrange`

Passthrough commands:

- `metaedit`
- `parallelize`
- `simplify-parents`

Planning bias:

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

Planned follow-ups:

- bookmark track/untrack

Passthrough workflow:

- `bookmark advance`

Planning bias:

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

Planned follow-ups:

- `op restore`
- `op revert`

Deferred:

- `operation abandon`

`operation integrate` is classified as passthrough/specialized in command inventory.

Planning bias:

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
- `bookmark delete`
- file list/show
- resolve

Planned follow-ups:

- bookmark rename/forget/track/untrack
- tag list/set/delete
- file search/annotate/track/untrack/chmod
- workspace root/list/add/rename/forget/update-stale

Planning bias:

- useful utility screens
- actions launched from the relevant utility context
- minimal chrome and no dashboard framing

## Passthrough Workflow

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

Planning bias:

- supported through regular `jj` CLI invocation outside `jk` for now
- native promotion requires evidence of frequency and clear `jk` value-add

## Deferred Workflow

Commands deferred for now:

- `gerrit`
- `util`
