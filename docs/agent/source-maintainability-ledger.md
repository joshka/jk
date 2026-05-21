# Source Maintainability Ledger

This ledger records the active maintainability objective for `jk` and the current evidence that can
guide bounded follow-up packets. It is not a standing "split the biggest files" queue. Measurements
nominate places to read first; they do not mandate refactors.

Before starting a source-shape packet, gather fresh measurements and read the owning module. Prefer
product work and local documentation improvements over movement that only changes line counts.

## Active Quality Bar

The current maintainability objective is documentation-first readability with vertical ownership.
Future packets should make the code easier for a maintainer to read locally without weakening the
rendered-`jj` presentation contract.

- Document durable ownership and caller-facing contracts on central modules, public types, and
  crate-visible boundaries.
- Keep behavior vertical: put rules, data, wording, tests, and docs near the concept that changes
  for the same reason.
- Put each rule where a maintainer would look when the user-visible concept changes. Ask which
  product concept owns the decision before asking what kind of code it is.
- Favor direct readable control flow over generic abstractions. Extract only when a named owner
  reduces live context, hides fewer side effects, or makes an invariant harder to violate.
- Preserve rendered `jj` output, argv shape, labels, refresh behavior, selection behavior, and
  app-level routing unless the packet explicitly owns that contract.
- Treat current kind-of-code buckets such as `jj_rows`, `jj_actions`, `action_menu`, `tui`, and
  `view_state` as temporary homes or shared infrastructure only when they hold genuinely shared
  contracts. They should not hide feature-specific product decisions.
- Treat line counts, visibility counts, repeated helper shapes, and regex hot spots as prompts for
  review, not proof that a split is correct.
- Each packet should say what changed, what intentionally stayed put, and which focused validation
  preserved the contract.

### Destination Shape

The long-term direction is feature roots plus shared infrastructure. The exact file names can
change, but a maintainer should be able to start from a feature such as `operation_log`,
`bookmarks`, `status`, `files`, or `log` and find the local row model, view behavior, action
availability, and tests without first understanding global buckets.

Feature modules should own product decisions that change together:

- view state and bindings;
- row models, row interpretation, and rendered-output assumptions;
- selection, search, copy, refresh, and reveal behavior;
- feature-specific action availability and action target resolution;
- feature tests and user-visible contracts.

Shared modules should own cross-cutting mechanics that two feature owners can use without
understanding each other's domain:

- `jj`: process execution, syntax quoting, command construction, and view specs;
- `actions`: command plans, argv/preview/run contracts, and command-output preservation after a view
  has already chosen an action;
- `ui`: shared chrome, modal rendering, menus, status hints, and theme primitives;
- `app`: event loop, mode dispatch, action lifecycle, refresh/reveal orchestration, and services;
- `selection`, `search`, `clipboard`, and similar helpers only when the rule is domain-neutral.

Use the feature-policy versus shared-mechanics test before moving code: if two feature owners would
not use a helper without learning each other's product rules, keep the rule with one feature for
now.

Examples for future packets:

- Operation-log behavior now starts from `operation_log`: `src/operation_log/rows.rs` owns rendered
  row grouping, operation-id template parsing and pairing, and fail-closed metadata drift tests;
  `src/operation_log.rs` owns movement/copy, undo/redo/restore/revert availability, operation detail
  navigation, and view tests.
- Bookmark behavior now starts from `bookmarks`: `src/bookmarks/rows.rs` owns rendered row loading,
  bookmark metadata template parsing and pairing, local/remote state classification, and fail-closed
  drift tests; `src/bookmarks/action_targets.rs` owns safe mutation targets.
- Cross-view action plans such as rebase, squash, absorb, new, edit, duplicate, split, restore,
  revert, track, untrack, chmod, fetch, push, describe, and abandon may live under an action-plan
  owner, but view-specific availability belongs with the feature that offers the action.
- Rendered document scrolling, sticky file headings, and rendered jj document parsing may become a
  document feature owner when that lowers reader burden more than today's separate helper modules.

### Recent Packet Evidence

2026-05-21 shared chrome rendering contract documentation:

- `src/tui.rs` now documents the split between feature-owned main content and shared chrome-owned
  title/status/overlay rendering.
- The packet records presentation-only contracts for optional status hints, borrowed overlays,
  action-output body/footer sizing, abandon-confirm input rendering, fallback-friendly overlay
  styling, and clipped modal geometry.
- Focused validation covered `cargo check`, `cargo test tui -- --test-threads=1`, and
  `rustup run nightly cargo fmt --check`. Full `just check` also passed at the top of the stack.

2026-05-21 action menu ownership contract documentation:

- `src/action_menu.rs` now documents the split between shared menu presentation contracts, stable
  action vocabulary, role prompts, follow-up payloads, feature-owned availability, app-owned
  lifecycle, and `jj_actions` command plans.
- Follow-up docs now explicitly constrain payloads to exact revision strings, operation ids,
  selected paths, role prompts, candidate lists, and chmod modes instead of UI selection state,
  preview text, refresh policy, or reveal targets.
- Focused validation covered `cargo check`, `cargo test action_menu -- --test-threads=1`, and
  `rustup run nightly cargo fmt --check`.

2026-05-21 app screen projection contract documentation:

- `src/app_screen.rs` now documents how transient `InteractionMode` state projects into prompt
  status lines and borrowed `tui::Overlay` values without owning dispatch, command execution, or
  side effects.
- `ViewMenuOption` and `view_menu_options` now document the split between user-visible menu labels,
  static menu data, selected-index clamping, and app-owned navigation/diff-format dispatch.
- Focused validation covered `cargo check`, `cargo test app_screen -- --test-threads=1`, and
  `rustup run nightly cargo fmt --check`.

2026-05-21 shared row helper contract documentation:

- `src/jj_rows.rs` now states that feature-specific row policy belongs in feature roots and this
  module owns only domain-neutral rendered-row mechanics: plain-text flattening, metadata drift
  handling, JSON field extraction, graph-line detection, and rendered line text extraction.
- Main-thread review rewrote several generic helper comments into contracts about fail-closed
  metadata parsing and intentional style loss.
- Focused validation covered `cargo check`, `cargo test jj_rows -- --test-threads=1`, and
  `rustup run nightly cargo fmt --check`. The test filter matched 0 tests, so the useful proof for
  the docs-only change is buildability and formatting.

2026-05-21 command dispatch contract documentation:

- `src/command.rs` now documents the boundary between app-level command vocabulary, view-local
  commands, binding metadata, key-pattern matching, command context input, and `ViewEffect` output.
- The packet preserved behavior and public API shape. Its value is reader locality: future key,
  prefix, help, and status-hint work can inspect command contracts before tracing `app.rs`.
- Focused validation covered `cargo check`, `cargo test command -- --test-threads=1`, and
  `rustup run nightly cargo fmt --check`.

2026-05-21 feature-root refactoring direction:

- `docs/agent/architecture.md` now states the destination shape as feature roots plus shared
  infrastructure and gives the rule for future moves: ask which product concept owns the decision
  before asking what kind of code it is.
- `AGENTS.md` now gives the same compact project-level rule, so future agents start with feature
  policy versus shared mechanics before selecting an owner module.
- The guidance names `operation_log`, `bookmarks`, `status`, `files`, `documents`, `app`, `jj`,
  `actions`, and `ui` as conceptual destinations without making exact filenames the review target.
- The packet intentionally changed docs only. The useful proof is Markdown formatting/linting and a
  review that the guidance reinforces the existing vertical row migrations rather than prescribing a
  broad rewrite.

2026-05-21 graph row ownership migration:

- `src/graph/rows.rs` now owns `LogItem`, `load_entries`, `load_compact_log_context`, rendered log
  row grouping, revision metadata template execution, fail-closed metadata pairing, and the row
  tests that previously lived under `src/jj_rows/revisions.rs`.
- `src/graph.rs` declares `mod rows;` and re-exports the graph row surface for crate-local app/view
  tests while continuing to own graph selection, multi-select, reveal, search, copy, and
  graph-to-detail navigation.
- `src/jj_rows.rs` no longer has submodules or graph/log row exports. It keeps only shared row
  helper mechanics such as plain-text flattening, metadata drift handling, JSON field helpers,
  graph-line helpers, and `line_text`.
- `src/show.rs`, `src/view_state.rs`, and focused app tests now construct or load graph rows through
  `crate::graph`, so tests point at the feature owner instead of the old kind-of-code bucket.

2026-05-21 log row ownership definition:

- `src/sticky_file_view.rs` now loads rendered document lines directly through `run_jj` with
  `ColorMode::Always`, so file-oriented document views no longer depend on graph/log row items just
  to preserve styled output.
- At the time of this packet, `src/jj_rows/revisions.rs` was graph/log-specific in practice:
  `src/graph.rs` consumed `LogItem` and `load_entries` for selectable graph rows, while
  `src/show.rs` consumed only `load_compact_log_context` for sticky commit context.
- This packet set the acceptance criteria for the follow-up graph row migration: graph selection,
  multi-select, reveal, search, copy, visible-working-copy detection, compact show context, and
  rendered document loading had to retain their current output and tests, while `sticky_file_view`
  stayed independent of `LogItem`.

2026-05-21 file-list row migration:

- `src/file_list/rows.rs` contains `FileListItem`, `load_file_list_entries`, the exact path parser,
  and the file-list row tests that previously lived under `jj_rows`.
- `src/file_list.rs` declares `mod rows;` and re-exports the file-list row item and loader for
  crate-local app/view tests while continuing to own selection, search, copy, refresh, drill-down,
  and file action behavior.
- `src/jj_rows.rs` no longer owns file-list row loading or exact path parsing. It keeps shared
  rendered-row helpers such as `document_plain_text`, `RowMetadata`, JSON field helpers, graph-line
  helpers, and `line_text`, plus revision/log row loading.
- `src/app/tests/support.rs`, focused app tests, and `src/view_state.rs` construct file-list rows
  through `crate::file_list::...`, so tests now point at the feature owner.

2026-05-21 row ownership reassessment:

- Before the file-list row migration, `src/jj_rows.rs` had already shrunk to revision/log rows,
  file-list rows, and shared rendered-row helpers such as `document_plain_text`, `RowMetadata`, JSON
  field helpers, graph-line helpers, and `line_text`.
- The reassessment nominated file-list rows as the cleanest remaining feature-root row migration
  because `src/file_list.rs` already owned the user-visible `jj file list` view.
- Before the log row ownership definition, revision/log rows were broader than a size cleanup:
  `src/graph.rs` consumed `LogItem` and `load_entries`, while `src/show.rs` used
  `load_compact_log_context` and `src/sticky_file_view.rs` used `load_entries` for file-detail
  behavior. The follow-up packet split document loading before moving graph rows.

2026-05-21 resolve row migration:

- `src/resolve/rows.rs` contains `ResolveEntry`, `load_resolve_entries`,
  `RESOLVE_CONFLICT_TEMPLATE`, resolve JSON template parsing, integer side-count parsing, and the
  resolve row parser tests that previously lived under `jj_rows`.
- `src/resolve.rs` declares `mod rows;` and re-exports the resolve row item and loader for
  crate-local app/view tests, plus the test-only conflict template for `src/jj.rs` command argv
  tests.
- `src/jj_rows.rs` no longer owns resolve row parsing or the resolve conflict template. It kept
  shared rendered-row helpers such as `line_text` and JSON string helpers for revision and
  feature-owned row loaders.
- `src/app/tests/support.rs`, focused detail-restore tests, and `src/jj.rs` tests now reference
  resolve row/template ownership through `crate::resolve::...`, so tests point at the feature owner.

2026-05-21 workspace row migration:

- `src/workspaces/rows.rs` contains `WorkspaceContext`, `WorkspaceItem`, `load_workspace_context`,
  `WORKSPACE_METADATA_TEMPLATE`, workspace metadata parsing, row pairing, root/list/metadata
  degradation, and the workspace row tests that previously lived under `jj_rows`.
- `src/workspaces.rs` declares `mod rows;` and re-exports the workspace row context, item, loader,
  and test-only metadata template for crate-local app/view/jj tests.
- `src/jj_rows.rs` no longer declares a workspace submodule or re-exports workspace row types. It
  keeps shared rendered-row helpers such as `line_text` and JSON field helpers because revision,
  file-list, and feature-owned row loaders still use them.
- `src/app/tests/support.rs` and `src/jj.rs` tests now reference workspace row/context/template
  ownership through `crate::workspaces::...`, so tests point at the feature owner.

2026-05-21 bookmark row migration:

- `src/bookmarks/rows.rs` contains `BookmarkItem`, `BookmarkRowState`, `LocalBookmarkRemoteState`,
  `RemoteBookmarkTrackingState`, `BookmarkLocalPeerState`, `load_bookmark_entries`,
  `BOOKMARK_METADATA_TEMPLATE`, metadata parsing, row pairing, and the bookmark row tests that
  previously lived under `jj_rows`.
- `src/bookmarks.rs` declares `mod rows;` and re-exports the row item, row-state enums, and loader
  for crate-local app/view tests.
- `src/jj_rows.rs` no longer declares a bookmark submodule or re-exports bookmark row types. It
  keeps shared rendered-row helpers such as `line_text` and JSON field helpers because revision,
  workspace, file-list, and feature-owned row loaders still use them.
- `src/app/tests/support.rs`, focused app tests, and `src/view_state.rs` construct bookmark rows
  through `crate::bookmarks::...`, so tests now point at the feature owner.

## Mechanical Audit Snapshot

Snapshot date: 2026-05-21. Rerun these commands before using the measurements for new work.

Commands used:

```sh
just largest-rust-files
rg -n '^\s*pub(\(|\s)' src
rg -n '^\s*pub\s' src
rg -n '^\s*pub\(crate\)' src
rg -n '^\s*pub\(super\)' src
rg -n '^//!|^///|^\s*pub(\([^)]*\))?\s+(struct|enum|fn|const|static|trait|type|mod|use)\b' \
  src/main.rs src/app.rs src/app_screen.rs src/command.rs src/action_menu.rs src/tui.rs \
  src/jj_actions.rs src/jj_rows.rs
rg -n '(ListState|selected|selection|restore|move_(up|down|to|selection|selected)|'\
'select_(next|previous)|next_(item|row)|previous_(item|row)|clamp)' \
  src/graph.rs src/status.rs src/file_list.rs src/resolve.rs src/bookmarks.rs \
  src/operation_log.rs src/workspaces.rs
rg -n '(complete|completion|result|outcome|finish|preview|execute|ActionOutput|'\
'ActionResult|MutationPlan|FollowUp|status)' src/app/action_lifecycle src/jj_actions.rs
rg -n '^\s*(match|if let|while let|for |loop\b)|\bmatch\b|else \{|\.and_then\(|\.map_or\(' \
  src/*.rs src/app/*.rs src/app/action_lifecycle/*.rs src/jj_actions/*.rs src/jj_rows/*.rs \
  src/action_menu/*.rs src/jj/*.rs src/tui/*.rs
```

### Largest Rust Files

The largest production files reported by `just largest-rust-files` were:

```text
1218 src/graph.rs
1191 src/jj_actions.rs
 994 src/tui.rs
 977 src/bookmarks.rs
 876 src/bookmarks/rows.rs
 833 src/jj_actions/bookmarks.rs
 820 src/status.rs
 797 src/jj/view_spec.rs
 773 src/jj.rs
 743 src/action_menu/revision_actions.rs
 705 src/sticky_file_view.rs
 642 src/help.rs
 639 src/jj_actions/working_copy.rs
 632 src/command.rs
 628 src/app_screen.rs
 613 src/diff.rs
 597 src/rendered_jj.rs
```

The same report also listed large test files. Those test sizes are evidence to read the surrounding
contracts, not evidence to split production code.

### Rustdoc Coverage

The named central files all have module docs. The mechanical immediate-doc scan still shows weak or
missing item-level Rustdoc on many public or crate-visible contracts:

- `src/main.rs`: module docs are enough for the binary entry point; `run` lives in `src/app.rs`.
- `src/app.rs`: `run` lacks direct Rustdoc, though the module docs explain app orchestration.
- `src/app_screen.rs`: `InteractionMode` is documented, but `ViewMenuOption`, `ViewMenuAction`, and
  view-menu helpers are weakly documented.
- `src/command.rs`: the central enums are documented, but many public constructors, binding helpers,
  and prefix-match helpers have no direct caller-facing docs.
- `src/action_menu.rs`: module docs exist, but the public action vocabulary, prompts, menu item
  model, and facade are mostly undocumented at the item level.
- `src/tui.rs`: module docs exist, but `Areas`, `Overlay`, and render facades have no direct docs.
- `src/jj_actions.rs`: module docs and local comments explain preview-first plans, but public plan
  types and argv/preview/run methods mostly lack Rustdoc.
- `src/jj_rows.rs`: shared rendered-line helpers and revision/log row facades are weakly documented.

This nominates a source documentation sweep before another broad source split. The sweep should add
short ownership and contract docs only where a maintainer would otherwise need to reconstruct
visibility, side effects, ids, argv shape, or output-shape assumptions.

### Visibility Shape

Broad visibility counts across `src`:

```text
778 pub
 96 pub(crate)
118 pub(super)
```

The highest per-file broad-visibility counts were in `src/app/services.rs`, test support,
`src/jj_actions.rs`, `src/jj_actions/working_copy.rs`, `src/sticky_file_view.rs`,
`src/jj_actions/bookmarks.rs`, `src/action_menu.rs`, `src/jj/view_spec.rs`, and `src/command.rs`.
Use these counts to audit boundary clarity and documentation. Do not narrow visibility just to move
the count.

### Repeated List Mechanics

Selection, restore, movement, `ListState`, and clamp evidence across list-style views:

```text
172 src/graph.rs
 93 src/bookmarks.rs
 78 src/status.rs
 49 src/operation_log.rs
 47 src/file_list.rs
 44 src/resolve.rs
 42 src/workspaces.rs
```

The evidence shows a real repeated shape: views keep a `Selection`, build a Ratatui `ListState`,
move next/previous/first/last, clamp after refresh, and often restore by key or previous index. This
does not automatically justify a generic list-view abstraction. The current `Selection` and
`restore_by_key_or_index` helpers already cover some shared mechanics, while each view still owns
different ids, action menus, status messages, navigation effects, and refresh contracts.

Use this evidence for focused improvements only when a product change makes one repeated mechanic
hard to audit in multiple places.

### Action Completion And Result Handling

Action lifecycle and mutation-plan evidence:

```text
 97 src/app/action_lifecycle/preview.rs
 86 src/app/action_lifecycle/completion.rs
 72 src/app/action_lifecycle/entry.rs
 48 src/app/action_lifecycle/rewrite_completion.rs
 18 src/app/action_lifecycle/shared.rs
 15 src/jj_actions.rs
```

The clear owner for app-side completion policy is `src/app/action_lifecycle`, especially
`completion.rs`, `preview.rs`, and `shared.rs`. `src/jj_actions.rs` owns plan-side argv, preview
text, and execution helpers. A future packet should improve documentation or grouping at that
boundary only if it can state which side owns result wording, refresh/reveal behavior, or command
output preservation. Do not merge app lifecycle policy into mutation plans.

### Control-Flow Hot Spots

Cheap nested/control-flow counts nominated these files for readability review:

```text
51 src/app/mode_input.rs
31 src/app/action_lifecycle/completion.rs
28 src/graph.rs
25 src/jj_actions.rs
25 src/command.rs
24 src/help.rs
23 src/app.rs
22 src/status.rs
21 src/app/action_lifecycle/entry.rs
```

The stricter indentation-oriented scan put `src/app/mode_input.rs` first again with 35 hits, ahead
of `src/jj_actions.rs`, `src/command.rs`, `src/action_menu/revision_actions.rs`, and
`src/view_state.rs`. That makes `src/app/mode_input.rs` the only clearly bounded app readability
candidate in this snapshot.

## Completed Source-Shape Context

Recent completed packets already moved several coherent owners out of broad modules:

- help projection policy into `src/help.rs`;
- path and revision action-menu policy into `src/action_menu/path_actions.rs` and
  `src/action_menu/revision_actions.rs`;
- operation, bookmark, workspace, and resolve row loading into feature-owned `rows.rs` modules;
- revision row loading into `src/jj_rows/revisions.rs` as a staging point;
- ViewSpec navigation provenance into `src/jj/view_spec.rs`;
- status hint projection into `src/tui/status_hints.rs`;
- pure modal key reducers and prompt-plan helpers into `src/app/mode_input/reducers.rs`.

Those packets improved local contracts, but several are still organized by kind of code rather than
by user-visible feature. Treat them as staging points, not the final product shape:

- `src/jj_rows/*` proved row contracts and metadata pairing, but feature-owned row modules are the
  better destination when a feature packet touches the related view behavior. After the
  operation-log, bookmark, workspace, resolve, and file-list row migrations, `src/jj_rows.rs` is
  mostly shared helpers plus revision/log staging.
- `src/jj_actions/*` proved command-plan boundaries, but action availability and target policy
  should move toward feature roots rather than stay in global action-menu planning.
- `src/action_menu/*` should shrink over time toward shared menu vocabulary and presentation; the
  feature deciding that an action is available should own that decision.

Do not move code only to match a destination tree. Move it when a packet can preserve behavior, name
the owning product concept, and make the reader path shorter.

## Next Packet Recommendations

Recommended bounded candidates:

1. Source documentation sweep for central public and crate-visible contracts, starting with
   `src/action_menu.rs`, `src/tui.rs`, `src/jj_actions.rs`, `src/command.rs`, `src/app_screen.rs`,
   and the loaders/helpers in `src/jj_rows.rs`. The docs should explicitly mark which contracts are
   shared mechanics and which are feature policy that should migrate to a feature root when touched.
1. Revision/log row migration only if the packet first defines the feature root (`graph` versus a
   future `log`) and acceptance criteria across `src/graph.rs`, compact show context in
   `src/show.rs`, and sticky file detail behavior in `src/sticky_file_view.rs`. Do not do this only
   to reduce `src/jj_rows.rs` or `src/graph.rs` line counts.
1. Action lifecycle documentation or grouping packet if the owner is clearly app-side completion,
   preview, or shared result wording. Keep `src/jj_actions.rs` focused on plan construction, preview
   text, argv, and execution.
1. Selection/list mechanics packet only when one repeated movement, restore, or clamp rule changes
   across multiple list views and cannot stay readable through the existing `Selection` helpers.

Pause broad source-shape splits where modules are cohesive:

- `src/graph.rs` remains a large but coherent graph view with selection, search, refresh, log-mode
  switching, copy, detail navigation, and graph action-menu opening.
- `src/tui.rs` remains the shared chrome and overlay renderer after status hints moved out. Split
  only a concrete overlay family with snapshot proof.
- `src/bookmarks.rs` remains mostly view behavior plus focused tests; target-selection policy
  already has `src/bookmarks/action_targets.rs`.
- `src/jj_rows.rs` is now mostly shared helpers plus revision/log staging. Leave revision/log rows
  until the cross-view owner and behavior proof are clear.
- Do not create a `slices/` or other umbrella bucket. Prefer feature roots plus shared
  infrastructure.
