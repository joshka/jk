# Sync Workflow

## Goal

Exchange state with remotes and keep bookmark-tracking state understandable.

## Likely Commands

- `git fetch`
- `git push`
- bookmark track/untrack and related set flows

## UI Bias

- attach sync actions to status and bookmark-related screens
- make `jj git fetch` direct when the remote/default command shape is clear
- prefer previews and explicit targeting over broad remote dashboards

## Common Direct Flow: `jj git fetch`

Fetch is a high-frequency OSS workflow. It can be a direct action when the remote/default command
shape is clear.

Expected behavior:

- launch from status, log, or command mode;
- show command output or errors clearly;
- refresh the current screen after completion;
- avoid confirmation unless the command shape becomes unusual.

Push is different. It should preview the destination and affected refs before running.

## Acceptance Criteria

- common sync actions are obvious
- the workflow stays bounded and does not become a host-specific control panel
