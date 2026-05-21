# Sync Workflow

## Goal

Exchange state with remotes and keep bookmark-tracking state understandable.

## Likely Commands

Shipped today:

- `git fetch`
- `git push`
- bookmark create/set/move/delete
- bookmark track/untrack

Passthrough commands:

- bookmark advance

## UI Bias

- attach sync actions to status and bookmark-related screens
- make `jj git fetch` a direct action from normal app views when the remote/default command shape is
  clear
- prefer previews and explicit targeting over broad remote dashboards

## Common Direct Flow: `jj git fetch`

Fetch is a high-frequency OSS workflow. It can be a direct action when the remote/default command
shape is clear.

Expected behavior:

- available as a global/direct action from normal app views;
- show command output or errors clearly;
- refresh the current screen after completion;
- avoid confirmation unless the command shape becomes unusual.

Push is different. It should preview the destination and affected refs before running.

## Bookmark Track And Untrack

Bookmark track/untrack are shipped as preview-first actions from the bookmarks view:

- `bt` tracks the exact selected remote bookmark or a proven local row with exactly one eligible
  untracked remote sibling.
- `bu` untracks the exact selected tracked remote bookmark or a proven local row with exactly one
  eligible tracked remote sibling.
- both commands are always remote-scoped with `--remote exact:"<remote>"` and an exact bookmark
  string pattern.

The flow intentionally stays a guided ref maintenance action, not a remote dashboard. It does not
handle host state, branch protection, credentials, force-push policy, or broad remote inference.

## Acceptance Criteria

- common sync actions are obvious
- the workflow stays bounded and does not become a host-specific control panel
