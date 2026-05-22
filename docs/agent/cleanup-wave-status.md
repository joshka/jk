# Cleanup Wave Status

This document summarizes the current maintainability cleanup wave in product terms. Detailed
per-packet evidence stays in [`source-maintainability-ledger.md`](source-maintainability-ledger.md),
and packet-by-packet execution evidence stays in
[`../process-observations.md`](../process-observations.md).

## Current State

- The tracked call-tree cleanup map is now fully covered at `98/98` nodes.
- The runtime path from `main()` through `app`, modal dispatch, action lifecycle, view routing,
  feature roots, document mechanics, and shared TUI chrome has been converted from the earlier broad
  single-file owners into purpose-led roots or explicit "leave in place" decisions.
- Feature roots are now stronger for `log`, `status`, `operation_log`, `bookmarks`, `files`,
  `workspaces`, `resolve`, `show`, and `diff`.
- Shared action families are now shaped as real roots for `describe`, `abandon`, `rewrite`,
  `git_sync`, `working_copy`, and `files`.
- Shared document, menu, help, search, rendered-row, mode, and command surfaces now read as explicit
  boundaries instead of mixed implementation buckets.

## What Changed

- App runtime and input ownership were split across `app/mod.rs`, `app/navigation`, `app/dispatch`,
  `app/effects`, `app/input`, `app/reducers`, and `app/services`.
- Action lifecycle ownership was narrowed across `app/actions/entry`, `app/actions/preview`,
  `app/actions/completion`, `app/actions/input`, and `app/actions/pane`.
- View dispatch and target delegation were narrowed across `view_state`, `view_action_targets`, and
  the feature-owned target resolvers.
- Shared document mechanics were narrowed across `documents/rendered` and `documents/sticky`.
- `jj` command, process, and `ViewSpec` boundaries were narrowed into subtree roots.
- Root helpers were re-audited: `status_line` moved under `app`, `theme` moved under `tui`,
  `CopyOption` moved under `menus/model`, and `selection` plus `clipboard` were intentionally left
  as small honest helpers.

## What Stayed Intact

- Rendered `jj` output remains the default presentation source.
- Command argv shape, preview honesty, status wording, refresh behavior, reveal behavior, search
  boundaries, sticky file behavior, and copy behavior were preserved packet by packet.
- Shared roots that still exist do so because they currently read as coherent boundaries, not
  because they were skipped. The latest audits explicitly left `actions/mod.rs` and
  `app/actions/shared.rs` in place for that reason.

## Validation Position

- Packet-level `cargo check` runs were used after each structural move.
- Focused feature and action-family tests were run alongside each packet.
- The traversal map and process log are both current through the latest action-family splits.
- The remaining proof gap is repo-wide gate cleanliness, not uncertainty about the local ownership
  moves themselves.

## Remaining Reader Pain

- The largest files are now dominated by app and feature tests rather than mixed production owners.
- The biggest remaining production files are mostly coherent feature or boundary owners such as
  `bookmarks/actions/plan.rs`, `jj/view_spec/mod.rs`, `bookmarks/targets/resolver.rs`,
  `operation_log/detail.rs`, `workspaces/rows.rs`, and `terminal_process/mod.rs`.
- Future cleanup should continue to use measured reader pain instead of splitting because a file is
  merely large.

## Next Audit Questions

- Does the repo-wide gate pass with the refreshed cleanup docs and current stacked changes?
- Are there any stale status or audit docs that still describe the pre-split tree?
- After the gate passes, is there any remaining production owner on the call path that still reads
  as a mixed bucket rather than a coherent boundary?
