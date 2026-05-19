# Workflow Matrix

This document groups work by user workflow rather than by `jj` namespace. Workflows are the right
level for deciding what deserves native UI support.

## Inspect

Goal: understand history, change content, file content, and current repo state.

Likely surfaces:

- log
- show
- diff
- status
- file list/show/search/annotate
- bookmark list
- tag list
- workspace root

Planning bias:

- highest priority
- mostly read surfaces
- should set the quality bar for navigation, search, copy, and refresh

## Rewrite

Goal: reshape change history intentionally.

Likely commands:

- `new`
- `edit`
- `next`
- `prev`
- `commit`
- `describe`
- `rebase`
- `squash`
- `split`
- `abandon`
- `duplicate`
- `restore`
- `revert`

Planning bias:

- guided flows, not broad command mirrors
- `jj new trunk` is common, low-risk, and easy to undo, so it can be more direct than risky rewrite
  flows
- strong previews and confirmations for risky actions
- graph context should stay visible in the mental model even when the action launches from
  prompt/confirm flows

## Sync

Goal: exchange state with remotes and ref tracking surfaces.

Likely commands:

- `git fetch`
- `git push`
- bookmark track/untrack/set-related actions

Planning bias:

- attach to status and bookmark-related surfaces
- `jj git fetch` is common and low-risk enough to be a direct action with clear output and refresh
- previews matter more than command breadth
- keep host-specific integration out until the generic sync loop feels solid

## Recover

Goal: undo mistakes and inspect repository operation history.

Likely commands:

- `op log`
- `op show`
- `op diff`
- `undo`
- `redo`
- `op restore`
- `op revert`

Planning bias:

- second only to inspect in importance
- deserves strong, explicit UI language
- should make dangerous repair actions feel visible and reversible

## Refs And Workspace Hygiene

Goal: manage bookmarks, tags, files, workspaces, and local repo hygiene tasks.

Likely commands:

- bookmark list/set/create/move/rename/delete/forget/track/untrack
- tag list/set/delete
- file track/untrack/chmod
- workspace root/list/add/rename/forget/update-stale
- resolve list plus related conflict resolution entry points

Planning bias:

- useful utility surfaces
- can arrive after the core read/rewrite/recover loop
- should remain low-chrome and scoped, not become dashboards

## Passthrough Workflow

Goal: preserve breadth without committing to native UI too early.

Likely commands:

- `interdiff`
- `metaedit`
- `parallelize`
- `simplify-parents`
- `absorb`
- `fix`
- `config`
- `sparse`
- `sign`
- `unsign`
- `gerrit`
- `util`

Planning bias:

- supported through command mode only for now
- native promotion requires evidence of frequency and clear `jk` value-add
