# Source Maintainability Ledger

This ledger records the current bounded follow-up packets after the completed bookmark, rewrite,
help projection, and path action-menu refactoring slices. It is not a standing "split the biggest
files" queue. Reassess with fresh measurements before starting another source-shape packet.

Current evidence comes from the 2026-05-21 reassessment packet on the
`Reassess source maintainability queue` docs change: `just largest-rust-files`, `wc -l` over likely
hotspots, cheap visibility and control-flow scans, and direct reads of the cited source files
together with [`architecture.md`](architecture.md) and [`rust-style.md`](rust-style.md).

## Quality Bar

- Favor reader locality and low cognitive burden over generic file splitting.
- Keep ownership vertical: move rules, data, and wording toward the concept that changes for the
  same reason.
- Preserve rendered `jj` output, argv shape, labels, refresh behavior, selection behavior, and
  app-level routing unless the slice explicitly owns that contract.
- Use docs and tests as proof of ownership. Each packet should say what moved, what did not move,
  and what focused validation preserved the contract.

## Reassessment Snapshot

### Completed Slice History

- `Bookmark Action Target Resolver`
- `Bookmark Action Plan Submodule`
- `Bookmark Row Metadata Module`
- `Rewrite Action Plan Submodule`
- `Action Preview Pane Construction Helper`
- `Git Sync Action-Plan Cluster`
- `Working-Copy Action Plan Cluster`
- `Operation Recovery And Target Plan Cluster`
- `View Action-Target Projection Policy`
- `Simple Selection Restore Helper`
- `Retired src/jj.rs Compatibility Re-exports`
- `Documentation Drift Cleanup`
- `Help Projection Policy Packet`
- `File And Status Path Action-Menu Policy`

Those packets closed the previous bookmark-, rewrite-, help-, and path-heavy queue. The next work
should start from the current hotspots instead of replaying the old priority order.

### Current Size Snapshot

Current largest Rust files from `just largest-rust-files`:

```text
1440 src/jj.rs
1299 src/jj_rows.rs
1218 src/graph.rs
1192 src/app/tests/bookmark_actions.rs
1159 src/jj_actions.rs
1134 src/tui.rs
1028 src/action_menu.rs
 973 src/bookmarks.rs
 876 src/jj_rows/bookmarks.rs
 833 src/jj_actions/bookmarks.rs
 820 src/status.rs
```

Direct source checks after the help and path extractions:

```text
1440 src/jj.rs
1299 src/jj_rows.rs
1218 src/graph.rs
1134 src/tui.rs
1028 src/action_menu.rs
 973 src/bookmarks.rs
 642 src/help.rs
 632 src/command.rs
 246 src/action_menu/path_actions.rs
```

The completed extractions changed the old hotspots materially:

```text
before help extraction: 1255 src/command.rs
 after help extraction:  632 src/command.rs, 642 src/help.rs
before path extraction: 1246 src/action_menu.rs
 after path extraction: 1028 src/action_menu.rs, 246 src/action_menu/path_actions.rs
```

Large files alone do not justify a packet. They only nominate surfaces to inspect for mixed concepts
and high live context.

### Current Visibility Snapshot

Cheap `rg`-style counts found these visibility totals across `src/**/*.rs`:

- all `pub` lines: 1172
- restricted visibility lines (`pub(crate)`, `pub(super)`, `pub(in ...)`): 401

Top files by `pub` count:

- `src/app/services.rs`: 112
- `src/app/tests/support.rs`: 97
- `src/jj_actions.rs`: 84
- `src/jj_rows.rs`: 54
- `src/jj_actions/working_copy.rs`: 47
- `src/jj.rs`: 47
- `src/action_menu.rs`: 45
- `src/sticky_file_view.rs`: 44

These counts are measurement only. They are useful when they line up with a concept boundary, not as
a goal by themselves.

### Current Control-Flow Snapshot

Cheap token scans over production files found these current hotspots:

- `src/app/mode_input.rs`: 58
- `src/jj.rs`: 50
- `src/action_menu.rs`: 35
- `src/app/action_lifecycle/completion.rs`: 34
- `src/command.rs`: 32
- `src/jj_rows.rs`: 31
- `src/tui.rs`: 29
- `src/jj_actions.rs`: 29
- `src/bookmarks/action_targets.rs`: 28
- `src/app.rs`: 28

The current next slices come from where these counts overlap with mixed ownership during direct
reads. `src/graph.rs`, `src/tui.rs`, and `src/bookmarks.rs` are still large, but the next packet
should have an owner and a contract, not a line-count target.

## Completed Current Slices

### Help Projection Policy Packet

- Status: completed on 2026-05-21 in the Codex main thread.
- Owner: `src/help.rs`
- Outcome: `src/help.rs` owns `HelpContext`, help sections and rows, generated help projection,
  context visibility, and the `help_metadata` / `view_help_metadata` policy. `src/command.rs`
  remains focused on command vocabulary, key labels, binding matching, and help-mode prefix matching
  through the narrow `command_is_visible_in_help` helper.
- Non-goals: no key binding changes, no dispatch changes, no `ViewEffect` changes, and no TUI help
  layout redesign in `src/tui.rs`.
- Proof: focused command/help projection tests, especially context-specific visibility cases,
  `cargo check`, `just md-check`, and final `just check` with 533 passed / 2 ignored.

### File And Status Path Action-Menu Policy

- Status: completed on 2026-05-21 in the Codex main thread.
- Owner: `src/action_menu/path_actions.rs`
- Outcome: `src/action_menu/path_actions.rs` owns `FileActionContext`, its working-copy and exact
  revision scopes, status path menu construction, file path menu construction, chmod item
  construction, and the focused path-action policy tests. `src/action_menu.rs` keeps the shared
  action vocabulary and the broad `ExactActionContext` routing surface.
- Maintainability evidence: `src/action_menu.rs` dropped from 1246 lines in the reassessment
  snapshot to 1028 lines after extraction, and the new `src/action_menu/path_actions.rs` is 246
  lines including moved tests.
- Non-goals preserved: no graph multi-revision menu redesign, no role-prompt redesign, no mutation
  preview execution changes, and no new action vocabulary.
- Proof: focused action-menu, file-action, and detail-restore tests, plus `cargo check`,
  `cargo clippy -- -D warnings`, `rustup run nightly cargo fmt --check`, and `just md-check`.

### Operation Row Metadata Packet

- Status: completed on 2026-05-21 in the Codex main thread.
- Owner: `src/jj_rows/operations.rs`
- Outcome: `src/jj_rows/operations.rs` owns `OperationLogItem`, operation-log row loading, operation
  id metadata loading, row grouping, operation id parsing, and the focused operation row drift
  tests. `src/jj_rows.rs` keeps the stable facade for `OperationLogItem` and
  `load_operation_log_entries`, plus shared row helpers and the remaining row families.
- Maintainability evidence: `src/jj_rows.rs` dropped from 1299 lines in the reassessment snapshot to
  1075 lines after extraction, and the new `src/jj_rows/operations.rs` is 251 lines including moved
  tests.
- Non-goals preserved: no graph revision grouping changes, no bookmark/file-list/resolve/workspace
  row changes, no rendered ANSI conversion changes, no operation id shape changes, and no
  process-boundary move into `src/jj.rs`.
- Proof: focused `cargo test jj_rows -- --test-threads=1` passed during the extraction, with final
  gate commands recorded in `docs/process-observations.md`.

## Current Next Slices

### 1. Rendered Row Loader And Metadata Packets

- Owner: `src/jj_rows.rs`, with likely new siblings under `src/jj_rows/` beside
  `src/jj_rows/bookmarks.rs`.
- Purpose: continue splitting row loading and metadata pairing by rendered row family after the
  operation packet. The clearest remaining packet is workspace row loading (`WorkspaceContext`,
  `WorkspaceItem`, workspace metadata parsing and pairing), which already has a narrow template,
  row-count drift behavior, and focused tests in one file.
- Evidence: `src/jj_rows.rs` is 1075 lines after bookmark metadata and operation rows were moved
  out. `load_workspace_context`, `WorkspaceContext`, `WorkspaceItem`, workspace metadata parsing,
  and workspace drift tests form the next coherent row-family concept.
- Non-goals: do not change rendered ANSI conversion, graph revision grouping, bookmark rows, file
  list path preservation, or the process boundary in `src/jj.rs`. Do not broaden parsing beyond the
  current narrow metadata templates.
- Proof: focused workspace-row tests for the chosen packet, plus `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.

### 2. ViewSpec Navigation Provenance Packet

- Owner: `src/jj.rs`, likely a `ViewSpec`-owned submodule if the implementation needs one.
- Purpose: make the `ViewSpec` contract easier to audit by grouping constructor policy,
  app-label/display-arg policy, diff-format application, and navigation revset provenance close to
  the type that owns it.
- Evidence: `src/jj.rs` is still the largest production file at 1440 lines. Direct reads show one
  coherent process-boundary owner, but `ViewSpec` carries several distinct responsibilities: command
  constructors, target exactness, path provenance, `navigation_revset`, `show_context_revset`,
  display labels, and diff-format rewriting. Those are all one concept, but they do not need to
  share live context with `run_jj`, `run_direct_args`, template execution, or remote parsing.
- Non-goals: do not change argv shape, command words, labels, `--git` behavior, exact-change safety,
  direct startup revset fallbacks, or any process helper. Do not move `jj_actions.rs` preview-first
  mutation plans into this packet.
- Proof: focused `jj` tests around `ViewSpec`, navigation revsets, diff format, operation specs,
  file specs, and labels, plus `cargo check`, `rustup run nightly cargo fmt --check`, and
  `just md-check`.

### 3. Graph Revision Action-Menu Policy Packet

- Owner: `src/action_menu.rs`, with a likely `src/action_menu/revision_actions.rs` or
  `graph_actions.rs` split if the boundary remains sharp during implementation.
- Purpose: separate graph/detail revision action policy from shared action vocabulary. The parent
  should keep `ActionKind`, `FollowUp`, `ActionMenuItem`, `ActionMenu`, and prompt vocabulary while
  the revision policy owns `ExactActionContext`, graph versus detail surface handling,
  multi-revision role prompts, and revision mutation item ordering.
- Evidence: after the path extraction, `src/action_menu.rs` is 1028 lines. The remaining dense area
  is not path policy; it is the graph/detail routing in `build_action_menu`, `ExactActionContext`,
  `menu_item_for_new_parents`, `menu_item_for_split`, `menu_item_for_multirev_action`,
  `menu_item_for_absorb`, and `mutation_menu_items`.
- Non-goals: do not alter labels, shortcuts, safety tiers, role-prompt wording, follow-up payloads,
  path action policy in `path_actions.rs`, or action execution in `jj_actions.rs` and
  `app/action_lifecycle.rs`.
- Proof: focused `cargo test action_menu -- --test-threads=1`, graph action-menu tests, detail
  restore/action tests, plus `cargo check`, `rustup run nightly cargo fmt --check`, and
  `just md-check`.

### 4. Status Hint Projection Packet

- Owner: `src/tui.rs`, with a likely `src/tui/status_hints.rs` split if the module shape allows it
  without broad churn.
- Purpose: move status-hint vocabulary and width-fit projection behind one small owner, leaving
  `tui.rs` focused on shared chrome and overlay rendering.
- Evidence: `src/tui.rs` is 1134 lines. Direct reads show overlay rendering is still broadly
  cohesive, but `StatusHints`, the per-view hint tables, `status_hint_candidates`, and
  `status_hint_spans` are a bounded projection concept with clear tests and no app state.
- Non-goals: do not redesign title/status layout, help overlay rendering, action-output overlay
  rendering, menu rendering, or theme styles. Do not change which hints are shown for any view.
- Proof: focused `tui` status-line/status-hint tests or snapshots, plus `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.

## Not The Next Packet

- `src/graph.rs`: still large at 1218 lines, but the remaining direct read is one view contract:
  bindings, selection, search, refresh, mode switching, graph-to-detail navigation, and opening the
  action menu. The action-menu projection itself belongs in `src/action_menu.rs`, not another graph
  split.
- `src/bookmarks.rs`: still large at 973 lines, but much of that is focused tests around bookmark
  action-target safety. The production view now delegates target resolution to
  `src/bookmarks/action_targets.rs`; another split should wait for new bookmark view behavior, not
  happen only because tests are long.
- `src/tui.rs` overlay rendering as a whole: large but cohesive around shared chrome. The bounded
  status-hint packet above is reasonable; a broad overlay split is not yet justified without a
  concrete overlay owner and snapshot proof.
- `src/jj_actions.rs`: still visible in the largest-file list at 1159 lines, but the current shape
  is a facade over already extracted bookmark, rewrite, sync, working-copy, and operation action
  clusters. Do not start another `jj_actions` split until a remaining mutation-plan family presents
  a narrower owner.
- `src/app/services.rs`: has the highest `pub` count in the cheap scan, but that visibility is a
  test-service seam, not evidence of mixed production concepts. Do not optimize the count without a
  concrete service-boundary problem.
