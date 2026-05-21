# Source Maintainability Ledger

This ledger turns the current source audit into bounded follow-up work. Keep it concise: add only
findings that name an owning concept, a concrete risk, and a proof path. Use it with
[`architecture.md`](architecture.md), [`rust-style.md`](rust-style.md), and the repo-local
development guidance before starting broad source-shape work.

The evidence in this file comes from the 2026-05-20 source audit, `just largest-rust-files`, and
direct reads of the cited source files. The quality bar is grounded in these reviewed `../practice`
sources:

- `src/content/guides/rust-maintainability.md`
- `src/content/guides/documentation-workflow.md`
- `src/content/guides/code-shape.md`
- `src/content/patterns/reader-locality.md`
- `src/content/patterns/strengthen-cohesion.md`

## Quality Bar

- Favor reader-first Rust: reduce the number of concepts, fields, jumps, and hidden invariants a
  maintainer must hold at once.
- Improve cohesion by moving data, rules, and transitions toward the named concept that changes for
  the same reason.
- Preserve reader locality. Extract helpers only when the new name and location reduce live context;
  keep weak helpers near the caller.
- Separate structure from behavior. Source-shape cleanup should be behavior-preserving unless the
  change explicitly owns a user-visible behavior.
- Treat docs as contracts. Module comments and Rustdoc should state current behavior, current
  ownership, side effects, and lifecycle constraints.
- Keep follow-up slices bounded. A large file is a signal to inspect ownership, not a reason to
  split by line count.

## Current Concept Map

- Command contracts: `src/command.rs` owns key binding metadata plus `Command`, `ViewCommand`,
  `CommandContext`, and `ViewEffect`.
- App mode and screens: `src/app_screen.rs` owns `InteractionMode` and overlay/status projection;
  `src/app/mode_input.rs` owns modal and prompt key reducers.
- App services: `src/app/services.rs` owns `AppServices`, the app side-effect boundary used by tests
  and app orchestration.
- Action lifecycle: `src/app/action_lifecycle/entry.rs`, `completion.rs`, and
  `rewrite_completion.rs` own guided action dispatch, result handling, refresh, reveal, and status
  construction.
- Action planning: `src/jj_actions.rs` owns preview-first action plans, command argv construction,
  preview summaries, and fallback result wording.
- Rendered rows: `src/jj_rows.rs` owns rendered-row loading, metadata pairing, and row grouping for
  graph-adjacent utility views.
- Command execution and view specs: `src/jj.rs` owns shared `jj` process helpers, `ViewSpec`, direct
  command construction, and navigation target provenance.
- Document mechanics: `src/sticky_file_view.rs` owns sticky document scrolling, file anchors, file
  jumping, and search for rendered file-oriented documents.
- Shared chrome: `src/tui.rs` owns shared layout, status/header rendering, overlays, and modal
  presentation.

## Audit Findings

### Large Surfaces

`just largest-rust-files` reported these largest source files:

```text
3601 src/jj_actions.rs
1836 src/jj_rows.rs
1477 src/bookmarks.rs
1458 src/jj.rs
1245 src/action_menu.rs
1235 src/command.rs
1217 src/graph.rs
1192 src/app/tests/bookmark_actions.rs
1134 src/tui.rs
```

The maintainability question is not "split the largest files." The question is whether a future
change must keep unrelated facts live at the same time. `src/jj_actions.rs`, `src/jj_rows.rs`, and
`src/jj.rs` are the first places to inspect when action planning, rendered-row metadata, or command
construction starts mixing concepts.

### Stale Or Narrow Docs

- `src/action_menu.rs` still describes "future graph mutation preparation", but the module now owns
  active status, file, and operation action surfaces.
- `src/command.rs` says help and status text live in `tui.rs`, while command metadata owns much of
  the help/status vocabulary.
- `src/interactive_process.rs` says it has no app call site, but app services now wire it into app
  behavior.
- `src/sticky_file_view.rs` says show/diff only, while status and operation detail surfaces also use
  the shared document mechanics.

These are documentation-contract bugs. Fix them in source comments or Rustdoc as narrow docs-only
patches when touching the owning module.

### Weak Or Missing Intent Docs

The audit found important contracts that are visible in code but not yet explained strongly enough
for non-linear readers:

- `Command`, `ViewCommand`, `CommandContext`, and `ViewEffect` in `src/command.rs`
- `InteractionMode` in `src/app_screen.rs`
- action plan and value types in `src/jj_actions.rs`
- `AppServices` in `src/app/services.rs`
- private invariants such as `StatusPathContract`, `BookmarkMetadataCoverage`, `PlainDocument`, and
  `PendingCommand`

These do not need broad public API documentation. They need short ownership and invariant comments
near the type that future edits are likely to land on first.

### Repeated Or High-Live-Context Surfaces

- `handle_mode_key_event_with_terminal` in `src/app/mode_input.rs` carries many modal key paths in
  one control-flow surface.
- `apply_action_menu_item` in `src/app/action_lifecycle/entry.rs` maps many action menu variants to
  preview, prompt, run, or status behavior.
- `completion.rs` and `rewrite_completion.rs` repeat confirm/run/refresh/reveal/status-result
  construction.
- Graph, status, file, resolve, bookmark, operation, and workspace screens repeat
  identity-preserving list-view mechanics.

Do not jump from these findings to a generic list abstraction or a broad `jj_actions.rs` split. Use
one bounded behavior-preserving slice at a time, and prove that the new owner reduces live context.

## Prioritized Corrective Slices

### 1. Action Completion Outcome Helper

- Owner: `src/app/action_lifecycle`, likely a small `shared.rs` used by `completion.rs` and
  `rewrite_completion.rs`.
- Purpose: collapse repeated result construction after confirmed actions: run command, refresh,
  reveal target when applicable, and build the status/result screen.
- Non-goals: no behavior changes, no action menu redesign, no `jj_actions.rs` split, and no generic
  list abstraction.
- Proof: focused app action-lifecycle tests for existing completion and rewrite-completion flows,
  plus `cargo check`, focused working-copy and bookmark action tests, and `just md-check` if docs
  change.

### 2. Repair Stale Source Comments

- Owner: the module that owns each stale comment: `src/action_menu.rs`, `src/command.rs`,
  `src/interactive_process.rs`, and `src/sticky_file_view.rs`.
- Purpose: make module comments describe current ownership and call sites so future agents do not
  route work through outdated assumptions.
- Non-goals: no code movement and no behavior changes.
- Proof: `rustup run nightly cargo fmt --check`; `cargo check`; focused tests only if comments are
  edited near doctests or examples.

### 3. Document Central App And Command Contracts

- Owner: `src/command.rs`, `src/app_screen.rs`, `src/app/services.rs`, and `src/app.rs` for
  `PendingCommand`.
- Purpose: add short Rustdoc or private comments for the mode, command, effect, and service
  contracts that future dispatch changes must preserve.
- Non-goals: no visibility changes, no API reshaping, and no new command behavior.
- Proof: `cargo check`; focused command/help tests if wording changes affect generated help or
  status labels.

### 4. Name Action-Plan Cohesion Before Splitting

- Owner: `src/jj_actions.rs`.
- Purpose: identify which plan/value clusters change together before extracting any submodule.
  Candidate clusters include preview-first mutation plans, file mutation plans, bookmark plans, and
  operation recovery plans.
- Non-goals: no mechanical line-count split and no public facade churn.
- Proof: a docs or source-comment inventory first; then a behavior-preserving extraction with
  command-construction tests for each moved cluster.

### 5. Inventory Identity-Preserving List Mechanics

- Owner: the concrete list views first: graph, status, file, resolve, bookmarks, operation log, and
  workspaces.
- Purpose: name the shared invariants around preserving selection by identity across refresh,
  search, filtering, and content shrink.
- Non-goals: no generic list abstraction until at least two follow-up edits prove the same contract
  needs a shared owner.
- Proof: view-level tests that show selection preservation and clamping on refresh/content shrink
  for each touched screen.
