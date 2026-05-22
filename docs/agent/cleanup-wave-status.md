# Cleanup Wave Status

This document summarizes the current maintainability cleanup wave in product terms. Detailed
per-packet evidence stays in [`source-maintainability-ledger.md`](source-maintainability-ledger.md),
and packet-by-packet execution evidence stays in
[`../process-observations.md`](../process-observations.md).

This is a queue and completion-framing document, not the canonical ownership map. Use
[`architecture.md`](architecture.md) for current structure and [`workflow.md`](workflow.md) for the
implementation doctrine that future packets should follow.

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
- View dispatch and target delegation were narrowed across `view_state`, `view_state/targets`, and
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
- The final broader gate passed with `just check`, including Markdown checks, `cargo check`,
  `cargo test`, and `cargo clippy -- -D warnings`.
- That proves the current tree is build-clean and test-clean. It does not prove that every remaining
  large coherent owner should be split further.

## Remaining Reader Pain

- The largest files are now dominated by app and feature tests rather than mixed production owners.
- The biggest remaining production files are mostly coherent feature or boundary owners such as
  `bookmarks/actions/plan.rs`, `jj/view_spec/mod.rs`, `bookmarks/targets/resolver.rs`,
  `operation_log/detail.rs`, `workspaces/rows.rs`, and `terminal_process/mod.rs`.
- Future cleanup should continue to use measured reader pain instead of splitting because a file is
  merely large.

## What Completion Means Here

- The runtime-path cleanup wave is complete. The tracked traversal map is covered and the final repo
  gate passed on the resulting tree.
- That is not the same as “all maintainability work is complete forever.” The remaining question is
  no longer whether the call tree was reviewed; it is whether any dense coherent owner now creates
  enough reader pain to justify another bounded packet.

## Post-Traversal Queue

- Read `bookmarks/actions/plan.rs` before any broad bookmark action rework.
- Read `bookmarks/targets/resolver.rs` before changing bookmark target safety rules.
- Read `jj/view_spec/mod.rs` before expanding startup parsing or detail-navigation provenance.
- Read `operation_log/detail.rs` before reshaping operation detail document behavior.
- Read `workspaces/rows.rs` before changing workspace row metadata or pairing rules.
- Read `terminal_process/mod.rs` before changing interactive terminal handoff or restoration.
