# Source Maintainability Ledger

This ledger turns the current source audit into bounded follow-up work. Keep it concise: add only
findings that name an owning concept, a concrete risk, and a proof path. Use it with
[`architecture.md`](architecture.md), [`rust-style.md`](rust-style.md), and the repo-local
development guidance before starting broad source-shape work.

The evidence in this file comes from the 2026-05-20 source audit, `just largest-rust-files`, direct
reads of the cited source files, and the copied `../practice` guidance on reader-first Rust, code
shape, documentation workflow, and agent workflow.

## Quality Bar

- Favor reader locality and low cognitive burden. Reduce the number of concepts, fields, branches,
  and hidden invariants a maintainer must hold at once.
- Keep ownership vertical and cohesive. Move data, rules, and transitions toward the concept that
  changes for the same reason.
- Preserve rendered `jj` output, argv shape, status wording, selection behavior, key handling, and
  refresh semantics unless the slice explicitly owns that behavior.
- Separate structure from behavior. Source-shape cleanup should stay behavior-preserving unless the
  change explicitly owns a user-visible contract.
- Treat docs as contracts. Module comments and Rustdoc should state current ownership, side effects,
  lifecycle constraints, and selection or refresh rules.
- Measure before editing. Use current file-size, visibility, and hotspot scans to choose the next
  slice instead of splitting by line count alone.

## Current Concept Map

- Recent completed packets: `Action Preview Pane Construction Helper`,
  `Git Sync Action-Plan Cluster`, `View Action-Target Projection Policy`,
  `Bookmark Action Target Resolver`, `Extract simple selection restore helper`, and the
  `Retire Or Narrow` slice for `src/jj.rs` compatibility re-exports.
- Command and app contracts: `src/command.rs` owns key binding metadata plus `Command`,
  `ViewCommand`, `CommandContext`, and `ViewEffect`; `src/app_screen.rs` owns `InteractionMode`; and
  `src/app/mode_input.rs` owns modal and prompt key reducers.
- App services: `src/app/services.rs` owns the app side-effect boundary used by tests and app
  orchestration.
- Action lifecycle: `src/app/action_lifecycle/{entry,completion,rewrite_completion,shared}.rs` owns
  guided action dispatch, shared outcome helpers, refresh and reveal policy, and status/result
  construction; `src/app/action_lifecycle/preview.rs` owns the preview-pane construction helper.
- Bookmark cohesion: `src/bookmarks.rs` owns bookmark list state, rendering, refresh, and selection;
  `src/bookmarks/action_targets.rs` owns fail-closed bookmark action-target resolution;
  `src/jj_actions.rs` owns bookmark action plans and the remaining preview-first action plans;
  `src/jj_rows.rs` owns bookmark row metadata pairing; `src/view_action_targets.rs` owns bookmark
  action-target projection policy.
- Action planning: `src/jj_actions/git_sync.rs` owns the extracted git sync action-plan cluster;
  `src/jj_actions.rs` keeps the stable facade and the remaining action-plan clusters, including the
  rewrite action plan.
- View routing: `src/view_state.rs` keeps the view-level routing that chooses the next detailed
  screen.
- Selection mechanics: `src/selection.rs` owns the restore helper and shared selection cursor
  mechanics.
- Compatibility cleanup: `src/jj.rs` keeps only the helpers it owns, without the old compatibility
  re-export layer.
- Row metadata pairing: `src/jj_rows.rs` owns rendered-row loading, fail-closed metadata pairing,
  and row grouping for graph-adjacent utility views.
- Syntax helpers: `src/jj_syntax.rs` owns pure `jj` syntax helpers extracted from
  `src/jj_actions.rs`.
- Document mechanics: `src/sticky_file_view.rs` owns sticky document scrolling, file anchors, file
  jumping, and search for rendered file-oriented documents.
- Shared chrome: `src/tui.rs` owns shared layout, status/header rendering, overlays, and modal
  presentation.

## Audit Findings

### Large Surfaces

`just largest-rust-files` reported these largest source files:

```text
3289 src/jj_actions.rs
2145 src/jj_rows.rs
1478 src/bookmarks.rs
1440 src/jj.rs
1255 src/command.rs
1246 src/action_menu.rs
1218 src/graph.rs
1192 src/app/tests/bookmark_actions.rs
1134 src/tui.rs
820 src/status.rs
```

The maintainability question is not "split the largest files." The question is whether a future
change must keep unrelated facts live at the same time. `src/bookmarks.rs`, `src/jj_actions.rs`,
`src/jj_rows.rs`, `src/view_action_targets.rs`, `src/jj.rs`, `src/command.rs`, `src/action_menu.rs`,
`src/graph.rs`, and `src/sticky_file_view.rs` are the first places to inspect when bookmark vertical
cohesion, command construction, or selection-preserving view behavior starts mixing concepts.
`src/app/action_lifecycle/preview.rs` and `src/view_state.rs` dropped out of the top-file list after
the recent extractions, which is the expected result of the completed slices.

### Visibility Surface

Current broad `rg` scans found 283 public or restricted Rust items and 162 restricted-visibility
lines. The counts below are measurement-only, not a design judgment.

- `src/jj_actions.rs`: 34
- `src/app/services.rs`: 29
- `src/jj_rows.rs`: 19
- `src/command.rs`: 17
- `src/jj.rs`: 16
- `src/sticky_file_view.rs`: 15

### Match Hotspots

Current match and control-flow hotspot scans found these counts. The counts below are
measurement-only, not a design judgment.

- `src/jj_actions.rs`: 48
- `src/app/mode_input.rs`: 35
- `src/command.rs`: 28
- `src/app/action_lifecycle/completion.rs`: 27
- `src/app/action_lifecycle/preview.rs`: 26
- `src/view_state.rs`: 22
- `src/bookmarks.rs`: 19

### Closed Documentation Drift

The previous broad missing-module-doc finding is no longer active. A fresh scan only found
`src/main.rs` missing a `//!` module doc in the first eight lines. The central app and command
contract gaps are also closed by the `Document app command contracts` packet: `Command`,
`ViewCommand`, `CommandContext`, `ViewEffect`, `InteractionMode`, `AppServices`, and
`PendingCommand` are no longer active ledger gaps. The rendered-file guidance drift in
`docs/agent/architecture.md` is also closed. The sticky projection rules now refer to rendered
file-oriented documents directly instead of anchoring the contract to show/diff wording.

### Weak Or Missing Intent Docs

The audit still found important contracts that are visible in code but not explained strongly enough
for non-linear readers:

- action plan and value types in `src/jj_actions.rs`
- private invariants such as `StatusPathContract`, `BookmarkMetadataCoverage`, and `PlainDocument`
- `src/main.rs` missing a `//!` module doc in the first eight lines, if full module-doc coverage is
  worth keeping on the ledger

These do not need broad public API documentation. They need short ownership and invariant comments
near the type that future edits are likely to land on first.

### Repeated Or High-Live-Context Surfaces

- `src/jj_actions.rs`, `src/jj_rows.rs`, `src/bookmarks/action_targets.rs`, and
  `src/view_action_targets.rs` still share bookmark-related vertical concerns across action targets,
  action plans, and row metadata, but the selected-row target policy now has a focused owner.
- `src/jj_actions.rs` still mixes the remaining action-plan clusters, argv construction, preview
  summaries, and fallback wording. The git sync cluster is already out of the generic path, while
  the bookmark action-plan cluster remains in `src/jj_actions.rs`; the next bounded slice should
  extract that cluster without reopening the whole planner.
- `src/app/action_lifecycle/completion.rs` still concentrates the status/result construction
  branches for action outcomes. It is a smaller target than `src/jj_actions.rs`, but it remains a
  dense live-context surface.
- `src/bookmarks.rs` remains dense because bookmark list rendering, refresh, selection, search, and
  copy behavior still share the same file.
- `src/app/mode_input.rs` still carries many modal key paths in one control-flow surface, but the
  recent readability packet already reduced the densest dispatch path, so it is not the next packet.
- `src/app.rs` is no longer the main pressure point; the current measurements and review findings
  point at the surfaces above instead.

Do not jump from these findings to a generic list abstraction or a broad `jj_actions.rs` split. Use
one bounded, behavior-preserving slice at a time, and prove that the new owner reduces live context.

## Prioritized Corrective Slices

### 1. Completed: Bookmark Action Target Resolver

- Status: completed in the `Extract bookmark target resolver` packet.
- Result: `src/bookmarks/action_targets.rs` owns selected-row forget, track, and untrack resolution,
  including local/remote peer checks, exact-target checks, all-remotes requirements, and fail-closed
  metadata wording. `BookmarksView` keeps the existing public methods as thin delegates.
- Non-goals preserved: no bookmark behavior change, no action wording drift, no app call-site churn,
  and no row metadata or command-construction changes.
- Proof: focused bookmark resolver tests and app bookmark action tests preserve the current accepted
  and rejected states.

### 2. Bookmark Action Plan Submodule

- Owner: `src/jj_actions/bookmarks.rs`.
- Purpose: move bookmark action-plan construction into a stable submodule while keeping the public
  `jj_actions` facade intact.
- Non-goals: no argv wording drift, no behavior change, and no public call-site churn.
- Proof: `src/jj_actions.rs` still ranks among the largest and hottest files after the git-sync
  extraction.

### 3. Bookmark Row Metadata Module

- Owner: `src/jj_rows/bookmarks.rs`.
- Purpose: separate bookmark row metadata pairing from the broader row-loading module once the
  action and target ownership above have settled.
- Non-goals: no fail-closed behavior change, no row grouping redesign, and no new selection policy.
- Proof: `src/jj_rows.rs` remains a large row-pairing surface after the recent selection and
  action-target extractions.

### 4. Rewrite Action Plan Submodule

- Owner: `src/jj_actions.rs`, with a bounded rewrite child module if the cluster stays cohesive.
- Purpose: peel off the rewrite action-plan cluster as another bounded `jj_actions` slice after the
  bookmark ownership is clarified.
- Non-goals: no broad `jj_actions.rs` split, no public facade churn, and no wording drift.
- Proof: the current size and hotspot scans still put `src/jj_actions.rs` at the top of the source
  maintenance list.

### 5. Completed: Action Preview Pane Construction Helper

- Status: completed in a recent packet.
- Result: `src/app/action_lifecycle/preview.rs` owns the preview-pane construction helper that turns
  a successful preview/load result into `ActionOutput::pending` and records `StatusLine::error` on
  failure. Callers still own their exact `InteractionMode` variants, command labels, status
  contexts, and preview-text mappings.
- Non-goals preserved: no preview behavior change, no keymap redesign, and no result-model
  reshaping.
- Proof: focused preview tests covering current rendering and scroll behavior, plus `cargo check`.

### 6. Completed: Git Sync Action-Plan Cluster

- Status: completed in a recent packet.
- Result: `src/jj_actions/git_sync.rs` owns `JjGitFetch`, `JjGitPush`, `JjGitPushTarget`, and their
  command-construction tests; `src/jj_actions.rs` keeps the stable public facade through re-exports.
- Non-goals preserved: no broad `jj_actions.rs` split, no public facade churn, and no wording drift
  in argv labels or fallback result text.
- Proof: `cargo test jj_actions -- --test-threads=1` keeps the moved git sync tests discoverable
  through the `jj_actions` module path.

### 7. Completed: View Action-Target Projection Policy

- Status: completed in a recent packet.
- Result: `src/view_action_targets.rs` owns push targets, bookmark mutation targets, selected local
  bookmark names, bookmark forget targets, and exact restore/revert action contexts. `ViewState`
  keeps the stable app-facing methods as thin delegates, so view operation routing remains in
  `src/view_state.rs`.
- Non-goals preserved: no graph/search redesign, no new navigation model, no call-site churn, and no
  behavior or error wording drift.
- Proof: focused view-state and app action tests preserve the current action target mapping and
  destinations.

### 8. Completed: Simple Selection Restore Helper

- Status: completed in a recent packet.
- Result: the helper stays narrow and the selection contracts remain documented for future readers.
  `selection.rs` keeps cursor mechanics only; `graph.rs` preserves selection by change id with
  multi-selection retention; `status.rs` preserves the selected row by path, then rendered text,
  then previous cursor row; `file_list.rs` preserves exact file-path selection; `resolve.rs`
  preserves exact conflict-path selection; `bookmarks.rs` still depends on row-local mutation
  metadata; `operation_log.rs` preserves exact operation-id selection; and `workspaces.rs` preserves
  exact workspace-name selection while mixing selectable rows with header metadata.
- Proof: the `Extract simple selection restore helper` packet and the inventory in
  `docs/process-observations.md`.

### 9. Completed: Retired `src/jj.rs` Compatibility Re-exports

- Status: completed in a recent packet.
- Result: source and test imports now refer to `jj_actions` and `jj_rows` directly; `src/jj.rs`
  keeps only the helpers it owns; and no compatibility re-export remains.
- Proof: focused compile pass plus the import audit recorded in `docs/process-observations.md`.

### 10. Completed: Documentation Drift Cleanup

- Status: completed in the prior packet.
- Result: `docs/agent/architecture.md` now describes sticky rendering rules for rendered
  file-oriented documents directly, so the old show/diff wording drift is closed.
- Non-goals preserved: no source behavior work and no new contract surface.
- Proof: `just md-check`.
