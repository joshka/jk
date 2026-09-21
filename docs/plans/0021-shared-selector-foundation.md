# Shared Selector Foundation

Status: the resolver and initial consumers are integrated under review. See
[workspace coherence](../workspace-coherence.md) for validation and
[command coverage](../command-coverage.md) for native workflow scope. The adoption sections below
identify remaining migrations.

## Purpose

Command workflows need the same answers before constructing a preview: which repository objects
were chosen, which command role they fill, whether their order is meaningful, and whether the user
cancelled or submitted unusable state. Those answers should not be inferred independently by each
rebase, squash, restore, refs, or workspace form.

The `jk-core` resolver runs after interaction and before command construction.
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

The model is generic over the selected value. A revision workflow can submit a full commit ID, an
operation list can submit a full operation ID, and a bookmark consumer can submit its own identifier.
The resolver does not parse jj output or terminal rows.

## Initial Consumers

Three consumers use the resolver:

- `jj new` resolves ordered revision marks as parents, falling back to the cursor revision;
- the diff file-list popup resolves one fileset target and rejects an out-of-range index;
- operation show and diff resolve one full operation ID before returning an action result.

The semantic log cursor, diff navigation state, operation refresh policy, menu rendering, and
command builders remain unchanged. They own additional behavior that a generic selector should not
absorb.

Parent operands use full commit IDs. Stable change IDs continue to own marks and refresh selection,
but every marked change must resolve to one visible commit before submission. Missing or divergent
marks reject the request without falling back to the cursor. Display prefixes never determine
identity or uniqueness.

## Adopting The Foundation

Further workflow migrations should construct a `SelectionRequest` when command roles become known:

- rebase resolves revision sources separately from one or more destinations;
- squash resolves source revisions separately from the destination revision;
- restore resolves a revision source and an ordered fileset selection;
- bookmarks, tags, remotes, and workspaces use their matching `SelectorKind` with workflow-specific
  roles while keeping provider metadata in their loader or view row types.

Each workflow should match every `SelectionResolution` variant before constructing its existing
command query. Ambiguous, invalid, and cancelled outcomes must not invoke a runner or create a
command-history record. Command modules retain ownership of their queries and argv.

## Concrete Integration Boundaries

The rebase workflow's `PendingRebase::from_log` already freezes exact commit IDs. Its source
resolution can submit those IDs as `Revision` / `Source`, then resolve the chosen destination as
`Revision` / `Destination` when preparing the preview. Keep its source-role validation and stale or
divergent mark checks in the workflow: the shared resolver does not know jj semantics.

The refs workflow's `RemotePicker` owns a frozen list of remote names and a selected row. On Enter,
submit the selected name as `Remote` / `Target` before constructing fetch or push previews. The
bookmark row still owns local/remote identity and mutation eligibility.

The workspace lifecycle form captures its workspace name before input begins. Resolve a selected
workspace as `Workspace` / `Target` at that handoff; keep destination-path validation, add/forget
semantics, and active-workspace protections in the lifecycle form.

These workflows still own their selection checks; they have not migrated to the shared resolver.
Provider checks remain in each workflow, and the resolver supplies no generic picker widget.

## Remaining Interaction Work

This boundary cannot preserve hidden marks by itself. The current log refresh retains only marks
present in the replacement snapshot, so a narrower revset can clear marks silently. Separate
filtering from object removal and show hidden-mark counts before adopting filtered multi-selection.

The diff-file popup stores a row index while open. A terminal resize now captures the selected path
before reloading the diff and restores that path afterward; a missing path leaves selection invalid.
Any future asynchronous refresh or filtering must preserve the same identity rule rather than
reuse a valid index for a different file.

Overlay sizing uses terminal-cell width for Unicode text. The integrated overlays use shared styles
under the [TUI design contract](../tui-design.md), replacing the initial fixed dark/cyan palette.
Long names still need inspection and selection behavior that keeps their identity readable on
constrained terminals.
