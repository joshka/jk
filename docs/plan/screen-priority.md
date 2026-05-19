# Screen Priority Roadmap

This roadmap orders screens by the value they add to the core `jk` loop and by how much other work
depends on them. The order is not a release promise; it is the default planning sequence when no
stronger evidence says otherwise.

## Priority 0: Core Loop

Goal: make log -> inspect -> return good enough that `jk` is useful even before broad command
coverage.

1. **Log.** Home screen, revision selection, search, copy, refresh, show/diff drill-down, and row
   contract for future action selection.
1. **Show.** Detailed change inspection with sticky context, file navigation, search, copy, refresh,
   and switch-to-diff.
1. **Diff.** Patch inspection with sticky file context, file navigation, search, copy, refresh, and
   switch-to-show.
1. **Help/keymap.** Small discoverability surface that reflects active bindings and keeps the app
   learnable.

Implementation focus:

- stable semantic identities for selected log rows;
- default and broader revset/view modes for narrow current work versus wider repository context;
- renderable styled pieces that preserve user-configured `jj` output;
- back/forward history between log, show, and diff;
- search highlighting without rewriting the underlying rendered content;
- copy actions for ids and file labels.

## Priority 1: Daily Triage

Goal: keep common working-copy and recovery checks inside `jk`.

1. **Status.** Working-copy triage, changed groups, conflict signals, next-action entry points.
1. **Operation log.** Recovery and audit surface for operation ids, operation details, undo, redo,
   restore, and revert.
1. **Bookmarks.** Ref-state inspection and bookmark-centered actions, especially where sync and
   publication flows need context.

Implementation focus:

- section or item selection where actions naturally attach;
- exact semantic state before mutation-capable actions;
- refresh that preserves selected file, operation, or bookmark when possible;
- low-friction `jj git fetch` from status or global command context;
- previews and confirmations for dangerous recovery or ref changes.

## Priority 2: File And Conflict Utility

Goal: make file-specific inspection and conflict triage reachable without turning the app into a
dashboard.

1. **File list.** Revision or working-copy file list with path selection, copy, and drill-down.
1. **File show.** Dedicated file-content view for one path at one revision.
1. **Resolve.** Conflict list and resolution entry points.
1. **File annotate/search.** Later refinements once file navigation proves valuable.

Implementation focus:

- exact paths and fileset semantics for actions;
- no mutation based on loosely parsed file labels;
- dedicated detail screens over permanent preview panes;
- conflict semantics from structured or code APIs before guided resolution grows.

## Priority 3: Low-Frequency Utility

Goal: cover useful but less central jj concepts with focused utility screens.

1. **Tags.** Tag inspection and basic tag actions.
1. **Workspace root/list.** Workspace state, root inspection, and focused workspace flows.
1. **Evolog.** Change evolution inspection if usage proves it is common enough.
1. **Interdiff.** Derived detail view if comparing revisions becomes a frequent inspection task.

Implementation focus:

- utility screens should be simple lists or documents;
- no permanent sidebars or global dashboards;
- native promotion should be based on frequency and clear `jk` value.

## Priority 4: Guided Mutation Attachments

Goal: attach high-value write flows to existing screens rather than creating mutation-first screens.

Primary graph-attached flows:

- `new`, especially low-friction `jj new trunk`
- `edit`
- `next`
- `prev`
- `describe`
- `commit`
- `rebase`
- `squash`
- `split`
- `abandon`

Primary recovery-attached flows:

- `undo`
- `redo`
- `operation restore`
- `operation revert`

Primary ref/sync-attached flows:

- `git fetch`
- `git push`
- bookmark set/create/move/rename/delete/track/untrack

Implementation focus:

- selected rows must carry exact revision identity;
- multi-row selection must preserve order and graph relationships when they matter;
- common, easy-to-undo actions may be direct when the target is exact;
- previews should show what `jj` would do, not a separate invented model;
- risky actions need confirmation and a clear post-action refresh path.

## Priority Rules

- Prefer read surfaces before mutation flows that depend on them.
- Prefer a screen when repeated navigation, selection, refresh, or search matters.
- Prefer a guided flow when the user is doing one focused operation.
- Prefer passthrough when native UI would only mirror the CLI.
- Promote long-tail commands only when `jk` can add context, safety, or navigation.
- If a screen needs semantic data, choose the least duplicative non-lossy integration before
  implementation.
- Use [`interaction-model.md`](interaction-model.md) to keep shortcut meanings and mutation safety
  consistent across screens.
