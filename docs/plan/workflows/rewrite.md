# Rewrite Workflow

## Goal

Reshape history safely while keeping the graph and surrounding context visible in the user’s mental
model.

## Likely Commands

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

Deferred:

- `diffedit`
- `arrange`

Passthrough commands:

- `metaedit`
- `parallelize`
- `simplify-parents`

## UI Bias

- guided flows over generic command launching
- direct action for common, easy-to-undo flows such as `jj new trunk`
- preview and confirmation for risky actions
- actions should originate from meaningful context such as log, show, diff, status, or op-log

## Common Direct Flow: `jj new trunk`

Starting new work from trunk is a frequent OSS workflow. It should be available as a low-friction
action when the trunk target is exact.

Expected behavior:

- run from the log or another graph-aware context;
- use the configured trunk/main target, not a parsed display string;
- refresh the log after success;
- make the new working-copy change visible;
- treat `jj undo` as the recovery path rather than requiring a heavy confirmation.

## Acceptance Criteria

- common rewrites feel safer and more understandable than ad hoc shell usage
- mutation flows do not take over the whole product model
