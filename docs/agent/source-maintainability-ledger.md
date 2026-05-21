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

- Recent completed packets: `Extract simple selection restore helper`,
  `Inventory list selection contracts`, `Clarify app mode input dispatch`,
  `Inventory action planning cohesion`, `Repair remaining contract drift`,
  `Add packet quality gate`, `Audit source maintainability surface`,
  `Factor action completion outcomes`, `Repair stale source ownership docs`,
  `Document app command contracts`, `Fail closed on row metadata drift`,
  `Extract jj syntax helpers`, and the `Retire Or Narrow` slice for `src/jj.rs` compatibility
  re-exports.
- Command and app contracts: `src/command.rs` owns key binding metadata plus `Command`,
  `ViewCommand`, `CommandContext`, and `ViewEffect`; `src/app_screen.rs` owns `InteractionMode`; and
  `src/app/mode_input.rs` owns modal and prompt key reducers.
- App services: `src/app/services.rs` owns the app side-effect boundary used by tests and app
  orchestration.
- Action lifecycle: `src/app/action_lifecycle/{entry,completion,rewrite_completion,shared}.rs` owns
  guided action dispatch, shared outcome helpers, refresh and reveal policy, and status/result
  construction.
- Action planning: `src/jj_actions.rs` owns preview-first action plans, command argv construction,
  preview summaries, and fallback result wording.
- View state: `src/view_state.rs` owns action-target projection policy and the view-level routing
  that chooses the next detailed screen.
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
3585 src/jj_actions.rs
2145 src/jj_rows.rs
1478 src/bookmarks.rs
1440 src/jj.rs
1255 src/command.rs
1246 src/action_menu.rs
1218 src/graph.rs
778 src/app/mode_input.rs
731 src/app/action_lifecycle/preview.rs
702 src/view_state.rs
```

The maintainability question is not "split the largest files." The question is whether a future
change must keep unrelated facts live at the same time. `src/jj_actions.rs`, `src/jj_rows.rs`,
`src/jj.rs`, `src/bookmarks.rs`, `src/command.rs`, `src/action_menu.rs`, `src/graph.rs`, and
`src/sticky_file_view.rs` are the first places to inspect when action planning, row pairing, command
construction, or selection-preserving view behavior starts mixing concepts.

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
`PendingCommand` are no longer active ledger gaps.

### Active Documentation Drift

`docs/agent/architecture.md` still keeps the rendering guidance anchored to "show/diff" wording.
Fold that wording cleanup into the next packet that already touches documentation.

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

- `src/app/action_lifecycle/preview.rs` still repeats pending, finished, and status-context
  patterns. The review called this the clearest repeated implementation pattern.
- `src/jj_actions.rs` still mixes preview-first plans, argv construction, preview summaries, and
  fallback wording even after `src/jj_syntax.rs` absorbed the pure syntax helpers. The git sync
  cluster is the cleanest bounded next extraction.
- `src/view_state.rs` still repeats action-target projection and navigation routing that want a
  single policy owner.
- `src/app/mode_input.rs` still carries many modal key paths in one control-flow surface, but the
  recent readability packet already reduced the densest dispatch path, so it is not the next packet.
- `src/app.rs` is no longer the main pressure point; the current measurements and review findings
  point at the surfaces above instead.

Do not jump from these findings to a generic list abstraction or a broad `jj_actions.rs` split. Use
one bounded, behavior-preserving slice at a time, and prove that the new owner reduces live context.

## Prioritized Corrective Slices

### 1. Action Preview Pane Construction Helper

- Owner: `src/app/action_lifecycle/preview.rs`.
- Purpose: pull the repeated action preview pane construction pattern into one helper so the
  status-context and pending/finished setup reads as a single unit.
- Non-goals: no preview behavior change, no keymap redesign, and no result-model reshaping.
- Proof: focused preview tests covering current rendering and scroll behavior, plus `cargo check`.

### 2. Git Sync Action-Plan Cluster

- Owner: `src/jj_actions.rs`.
- Purpose: extract the git sync action-plan cluster now that the shared syntax helpers are already
  out of the file.
- Non-goals: no broad `jj_actions.rs` split, no public facade churn, and no wording drift in argv
  labels or fallback result text.
- Proof: command-construction tests for the moved cluster, plus the current compile pass.

### 3. View Action-Target Projection Policy

- Owner: `src/view_state.rs`.
- Purpose: group the action-target projection rules that decide how view commands turn into the next
  detail screen.
- Non-goals: no graph/search redesign and no new navigation model.
- Proof: focused view-state tests that show the current action target mapping still produces the
  same destinations.

### 4. Documentation Drift Cleanup

- Owner: `docs/agent/architecture.md` and `docs/agent/source-maintainability-ledger.md`.
- Purpose: fold the small remaining docs drift cleanup into the next packet that already touches
  documentation, instead of spinning up a separate source-shape pass.
- Non-goals: no source behavior work and no new contract surface.
- Proof: `just md-check`.

### 5. Completed: Retired `src/jj.rs` Compatibility Re-exports

- Status: completed in the current packet.
- Result: source and test imports now refer to `jj_actions` and `jj_rows` directly; `src/jj.rs`
  keeps only the helpers it owns; and no compatibility re-export remains.
- Proof: focused compile pass plus the import audit recorded in `docs/process-observations.md`.

### 6. Completed: Simple Selection Restore Helper Inventory

- Status: completed in the current packet.
- Result: the helper stays narrow and the selection contracts remain documented for future readers.
  `selection.rs` keeps cursor mechanics only; `graph.rs` preserves selection by change id with
  multi-selection retention; `status.rs` preserves the selected row by path, then rendered text,
  then previous cursor row; `file_list.rs` preserves exact file-path selection; `resolve.rs`
  preserves exact conflict-path selection; `bookmarks.rs` still depends on row-local mutation
  metadata; `operation_log.rs` preserves exact operation-id selection; and `workspaces.rs` preserves
  exact workspace-name selection while mixing selectable rows with header metadata.
- Proof: the `Extract simple selection restore helper` packet and the inventory in
  `docs/process-observations.md`.

### 7. Quality Gate Refinements

- Owner: `docs/agent/source-maintainability-ledger.md` and the measurement commands that feed it.
- Purpose: keep the next audit mechanical by refreshing `just largest-rust-files`, the visibility
  scan, and the module-doc scan before each new packet.
- Non-goals: no source behavior work and no broad guidance rewrite.
- Proof: rerun the measurement commands, update the ledger, and record the results in
  `docs/process-observations.md`.
