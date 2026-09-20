# Shared Selector Foundation

## Purpose

Command workflows need the same answers before constructing a preview: which repository objects
were chosen, which command role they fill, whether their order is meaningful, and whether the user
cancelled or submitted unusable state. Those answers should not be inferred independently by each
rebase, squash, restore, refs, or workspace form.

The foundation in `jk-core` deliberately starts after interaction and before command construction.
Views still own navigation, search, rendering, scrolling, refresh preservation, and ordered marks.
Provider adapters still own loading jj data. Command modules still own jj arguments.

```text
view cursor + ordered choices
              |
              v
     SelectionDecision<T>
              |
              v
       resolve_selection       (pure, provider-neutral)
              |
              v
 resolved | ambiguous | invalid | cancelled
              |
              v
       workflow command spec
```

## Contracts

- `SelectorKind` identifies revisions, filesets, operations, and the planned bookmark, tag, remote,
  and workspace extension points.
- `SelectorRole` states whether values are sources, destinations, parents, or general targets.
- `SelectionRequest` makes one-value versus one-or-more cardinality explicit.
- `SelectionCandidates` carries the current cursor and any explicitly ordered choices. Ordered
  choices win over the cursor and retain insertion order.
- `SelectionResolution` distinguishes success, ambiguity, invalid state, and cancellation. An empty
  vector is never a successful selection.
- `ResolvedSelection` records whether ordering came from the cursor or explicit user ordering.

The model is generic over the selected value. A revision view can submit a stable change ID, an
operation list can submit a full operation ID, and a future bookmark screen can submit its own
provider-neutral identifier without teaching the resolver about jj output or terminal rows.

## Initial Consumers

The first migrations validate boundary behavior without replacing mature view state:

- `jj new` resolves ordered revision marks as parents, falling back to the cursor revision;
- the diff file-list popup resolves one fileset target and rejects a stale index;
- operation show and diff resolve one full operation ID before returning an action result.

The semantic log cursor, diff navigation state, operation refresh policy, menu rendering, and
command builders remain unchanged. They own additional behavior that a generic selector should not
absorb.

## Adopting The Foundation

Mutation workspaces should construct a `SelectionRequest` at the point where roles become known:

- rebase resolves revision sources separately from one or more destinations;
- squash resolves source revisions separately from the destination revision;
- restore resolves a revision source and an ordered fileset selection;
- bookmarks, tags, remotes, and workspaces use their matching `SelectorKind` with workflow-specific
  roles while keeping provider metadata in their loader or view row types.

Each workflow should match every `SelectionResolution` variant before constructing its existing
command query. Ambiguous, invalid, and cancelled outcomes must not invoke a runner or create a
command-history record. This preserves one command-construction owner while making the handoff from
interactive state auditable and testable.
