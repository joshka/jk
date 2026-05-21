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

- Recent completed packets: `Add packet quality gate`, `Audit source maintainability surface`,
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
3557 src/jj_actions.rs
2145 src/jj_rows.rs
1477 src/bookmarks.rs
1458 src/jj.rs
1254 src/command.rs
1246 src/action_menu.rs
1217 src/graph.rs
1192 src/app/tests/bookmark_actions.rs
1134 src/tui.rs
819 src/status.rs
778 src/app/tests/working_copy_actions.rs
734 src/app/mode_input.rs
730 src/app/action_lifecycle/preview.rs
711 src/view_state.rs
704 src/sticky_file_view.rs
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
- `src/action_menu.rs`: 10
- `src/interactive_process.rs`: 8
- `src/theme.rs`: 7
- `src/tui.rs`: 6

### Match Hotspots

Current match and control-flow hotspot scans found these counts. The counts below are
measurement-only, not a design judgment.

- `src/jj_actions.rs`: 48
- `src/app/mode_input.rs`: 31
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

`docs/agent/architecture.md` still describes `src/sticky_file_view.rs` as show/diff-only, but the
current source usage includes status, file-show, and operation-detail surfaces. That doc should be
narrowed before the next ownership-oriented packet.

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

- `src/app/mode_input.rs` still carries many modal key paths in one control-flow surface.
- `src/app/action_lifecycle/{completion,rewrite_completion,preview,shared}.rs` now share outcome
  helpers, but `preview.rs` still repeats pending, finished, and status-context patterns.
- `src/jj_actions.rs` still mixes preview-first plans, argv construction, preview summaries, and
  fallback wording even after `src/jj_syntax.rs` absorbed the pure syntax helpers.
- Graph, bookmarks, file list, resolve, operation log, status, and workspaces still repeat
  identity-preserving list mechanics around `Selection`, `clamp`, selected identity, and
  refresh-preserves tests.

Do not jump from these findings to a generic list abstraction or a broad `jj_actions.rs` split. Use
one bounded, behavior-preserving slice at a time, and prove that the new owner reduces live context.

## Prioritized Corrective Slices

### 1. Remaining Contract Drift Sweep

- Owner: `docs/agent/architecture.md`, `src/jj_actions.rs`, `src/jj_rows.rs`,
  `src/sticky_file_view.rs`, and `src/main.rs` only if full module-doc coverage is still a tracked
  policy.
- Purpose: repair the sticky-file-view architecture drift and document the remaining action-plan,
  row-metadata, and private invariant contracts without re-opening already closed app/command
  contract work.
- Non-goals: no visibility changes, no API reshaping, and no behavior changes.
- Proof: `cargo check`; focused tests only if wording changes affect generated help, status labels,
  doctests, or the module-doc policy.

### 2. App Mode Input Readability

- Owner: `src/app/mode_input.rs`.
- Purpose: make the modal and prompt key reducers easier to read without changing mode behavior,
  especially around the dense key-event dispatch path.
- Non-goals: no new input modes, no binding redesign, and no command coverage changes.
- Proof: focused mode-input tests for current key paths, plus `cargo check`.

### 3. Action Planning Cohesion

- Owner: `src/jj_actions.rs`.
- Status: inventory completed in the current packet; extraction remains future work.
- Purpose: split coherent plan and value clusters only after their source contracts are named.
  Current clusters are operation recovery/targeting, git sync, working-copy creation/copy/split,
  describe/commit, working-copy navigation, content and file mutations, bookmark mutations, graph
  rewrite plans, and abandon safety.
- Non-goals: no mechanical line-count split and no public facade churn.
- Proof: source-comment inventory first; then behavior-preserving extraction with
  command-construction tests for each moved cluster when an extraction packet starts.

### 4. Vertical Ownership For Rows, Views, And Actions

- Owner: `src/jj_rows.rs`, `src/bookmarks.rs`, `src/graph.rs`, `src/status.rs`, `src/file_list.rs`,
  `src/resolve.rs`, `src/operation_log.rs`, and `src/workspaces.rs`.
- Purpose: move repeated selection, identity, and refresh contracts toward the view that owns the
  behavior instead of letting those rules spread horizontally.
- Non-goals: no generic list abstraction until at least two follow-up edits prove the same contract
  needs a shared owner.
- Proof: view-level tests that show selection preservation and clamping on refresh or shrink for
  each touched screen.

### 5. Identity-Preserving List Mechanics

- Owner: the concrete list views first, then any shared helper only if repeated proof shows it is
  worth centralizing.
- Purpose: name the shared invariants around preserving selection by identity across refresh,
  search, filtering, and content shrink.
- Non-goals: no broad abstraction before the contract is named and tested in the owning views.
- Proof: snapshots or focused tests showing the selected identity is preserved, clamped, or
  intentionally cleared on each screen.

### 6. Completed: Retired `src/jj.rs` Compatibility Re-exports

- Status: completed in the current packet.
- Result: source and test imports now refer to `jj_actions` and `jj_rows` directly; `src/jj.rs`
  keeps only the helpers it owns; and no compatibility re-export remains.
- Proof: focused compile pass plus the import audit recorded in `docs/process-observations.md`.

### 7. Quality Gate Refinements

- Owner: `docs/agent/source-maintainability-ledger.md` and the measurement commands that feed it.
- Purpose: keep the next audit mechanical by refreshing `just largest-rust-files`, the visibility
  scan, and the module-doc scan before each new packet.
- Non-goals: no source behavior work and no broad guidance rewrite.
- Proof: rerun the measurement commands, update the ledger, and record the results in
  `docs/process-observations.md`.
