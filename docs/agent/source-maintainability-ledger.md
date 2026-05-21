# Source Maintainability Ledger

This ledger records the current bounded follow-up packets after the completed bookmark, rewrite,
help projection, and path action-menu refactoring slices. It is not a standing "split the biggest
files" queue. Reassess with fresh measurements before starting another source-shape packet.

Current evidence comes from the 2026-05-21 row-extraction reassessment packet:
`just largest-rust-files`, `wc -l src/*.rs src/jj_rows/*.rs`, direct reads of the row loaders and
current large modules, and the ownership rules in [`architecture.md`](architecture.md) and
[`rust-style.md`](rust-style.md).

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
- `Graph Revision Action-Menu Policy Packet`
- `ViewSpec Navigation Provenance Packet`
- `Status Hint Projection Packet`

Those packets closed the previous bookmark-, rewrite-, help-, and path-heavy queue. The next work
should start from the current hotspots instead of replaying the old priority order.

### Current Source-Shape Snapshot

Current largest Rust files from `just largest-rust-files` on 2026-05-21:

```text
1218 src/graph.rs
1192 src/app/tests/bookmark_actions.rs
1159 src/jj_actions.rs
 976 src/tui.rs
 973 src/bookmarks.rs
 876 src/jj_rows/bookmarks.rs
 833 src/jj_actions/bookmarks.rs
 820 src/status.rs
 797 src/jj/view_spec.rs
 778 src/app/tests/working_copy_actions.rs
 778 src/app/mode_input.rs
 773 src/jj.rs
 760 src/jj_rows.rs
 743 src/action_menu/revision_actions.rs
 705 src/sticky_file_view.rs
 642 src/help.rs
 639 src/jj_actions/working_copy.rs
 632 src/command.rs
 628 src/app_screen.rs
 613 src/diff.rs
```

Current row-family line counts from `wc -l src/jj_rows.rs src/jj_rows/*.rs` after the 2026-05-21
graph revision row extraction:

```text
876 src/jj_rows/bookmarks.rs
498 src/jj_rows/revisions.rs
338 src/jj_rows/workspaces.rs
282 src/jj_rows.rs
251 src/jj_rows/operations.rs
```

The old 1440-line `src/jj.rs` and 1299-line `src/jj_rows.rs` measurements are stale. ViewSpec,
operation rows, workspace rows, revision action-menu policy, and status hint projection changed the
largest-file picture materially. The remaining source-shape pressure is now mostly view/action
cohesion, not a single obvious "split the biggest file" queue.

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
  construction, and the focused path-action policy tests. `src/action_menu.rs` kept the shared
  action vocabulary and the broad `ExactActionContext` routing surface until the revision packet
  below moved that routing into its own owner.
- Maintainability evidence: `src/action_menu.rs` dropped from 1246 lines in the reassessment
  snapshot to 1028 lines after extraction, and the new `src/action_menu/path_actions.rs` is 246
  lines including moved tests.
- Non-goals preserved: no graph multi-revision menu redesign, no role-prompt redesign, no mutation
  preview execution changes, and no new action vocabulary.
- Proof: focused action-menu, file-action, and detail-restore tests, plus `cargo check`,
  `cargo clippy -- -D warnings`, `rustup run nightly cargo fmt --check`, and `just md-check`.

### Graph Revision Action-Menu Policy Packet

- Status: completed on 2026-05-21 in a Codex worker/subagent.
- Owner: `src/action_menu/revision_actions.rs`
- Outcome: `src/action_menu/revision_actions.rs` owns `ExactActionContext`, graph/detail/status/file
  surface routing, multi-revision role-prompt item construction, single-revision action ordering,
  detail selected-path insertion policy, and revision mutation item construction.
  `src/action_menu.rs` keeps shared action vocabulary, prompt vocabulary, `FollowUp`,
  `ActionMenuItem`, `ActionMenu`, the public `ExactActionContext` re-export, the public
  `build_action_menu` facade, and the shared `short_id` helper used by path and revision policy.
- Maintainability evidence: `src/action_menu.rs` dropped from 1028 lines after the path extraction
  to 302 lines after this packet, while the new `src/action_menu/revision_actions.rs` is 743 lines
  including moved tests. `src/action_menu/path_actions.rs` remains 246 lines and unchanged in
  ownership.
- Non-goals preserved: no label, shortcut, safety-tier, role-prompt wording, follow-up payload, path
  action policy, action execution, or app lifecycle behavior changes.
- Proof: focused `cargo test action_menu -- --test-threads=1` passed with graph, path, app, and TUI
  action-menu tests; `cargo test detail_restore_actions -- --test-threads=1` passed; final gate
  commands are recorded in `docs/process-observations.md`.

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

### Workspace Row Loading Packet

- Status: completed on 2026-05-21 in the Codex main thread.
- Owner: `src/jj_rows/workspaces.rs`
- Outcome: `src/jj_rows/workspaces.rs` owns `WorkspaceContext`, `WorkspaceItem`,
  `load_workspace_context`, the workspace metadata template, metadata loading, rendered-row pairing,
  workspace metadata parsing, row-count drift behavior, and focused workspace row tests.
  `src/jj_rows.rs` keeps the stable workspace facade plus shared row helpers and the remaining row
  families.
- Maintainability evidence: `src/jj_rows.rs` dropped from 1075 lines after the operation-row
  extraction to 760 lines after this packet, and the new `src/jj_rows/workspaces.rs` is 338 lines
  including moved tests.
- Non-goals preserved: no graph revision grouping changes, no operation/bookmark/file-list/resolve
  row changes, no rendered ANSI conversion changes, no workspace metadata JSON shape changes, no
  row-count drift behavior changes, and no workspaces view behavior changes.
- Proof: focused `cargo test jj_rows -- --test-threads=1` and
  `cargo test workspaces -- --test-threads=1` passed during the extraction, with final gate commands
  recorded in `docs/process-observations.md`.

### ViewSpec Navigation Provenance Packet

- Status: completed on 2026-05-21 in the Codex main thread.
- Owner: `src/jj/view_spec.rs`
- Outcome: `src/jj/view_spec.rs` owns `DiffFormat`, `ViewSpec`, constructor provenance, app and jj
  labels, exact-change target policy, path provenance, diff-format arg rewriting,
  `navigation_revset`, `show_context_revset`, and the direct startup revset parsers used by those
  policies. `src/jj.rs` re-exports `DiffFormat` and `ViewSpec` while keeping `JjCommand`,
  `LogViewMode`, command-word/prefix behavior, direct commands, process helpers, templates, and
  output summarization.
- Maintainability evidence: `src/jj.rs` dropped from 1440 lines in the reassessment snapshot to 773
  lines after extraction, and the new `src/jj/view_spec.rs` is 797 lines including focused
  provenance tests.
- Non-goals preserved: no argv shape changes, no command word changes, no label wording changes, no
  `--git` behavior changes, no exact-change safety changes, no direct startup revset fallback
  changes, no process helper movement, and no `jj_actions.rs` mutation-plan movement.
- Proof: focused `cargo test jj -- --test-threads=1` passed during the extraction, with final gate
  commands recorded in `docs/process-observations.md`.

### Status Hint Projection Packet

- Status: completed on 2026-05-21 in a Codex worker/subagent.
- Owner: `src/tui/status_hints.rs`
- Outcome: `src/tui/status_hints.rs` owns `StatusHints`, per-view status hint tables,
  `status_hint_candidates`, `status_hint_spans`, status-hint key styling, and the width-fit helper
  used only for status-bar hint projection. `src/tui.rs` re-exports `StatusHints`, calls the narrow
  `status_hint_spans` facade from `status_line_text`, and keeps shared title/status chrome,
  overlays, action-output layout, menu rendering, and overlay footer helpers.
- Maintainability evidence: `src/tui.rs` dropped from 1134 lines before the packet to 976 lines
  after extraction, and the new `src/tui/status_hints.rs` is 202 lines including focused projection
  tests.
- Non-goals preserved: no title/status layout redesign, no help overlay rendering changes, no
  action-output overlay rendering changes, no menu rendering changes, no theme style changes, no
  overlay footer movement, and no status hint vocabulary or truncation behavior changes.
- Proof: focused `cargo test tui -- --test-threads=1` passed with the moved module tests and the
  existing status chrome snapshots; final gate commands are recorded in
  `docs/process-observations.md`.

### Graph Revision Row Loading Packet

- Status: completed on 2026-05-21 in a Codex worker/subagent.
- Owner: `src/jj_rows/revisions.rs`
- Outcome: `src/jj_rows/revisions.rs` owns `LogItem`, graph revision row loading, compact
  log-context loading, revision metadata template execution, revision metadata parsing, rendered
  graph row grouping, revision id pairing, and the focused revision grouping/parser tests.
  `src/jj_rows.rs` keeps the stable revision row facade plus resolve rows, file-list rows,
  `document_plain_text`, `line_text`, JSON field helpers, `RowMetadata`, `first_content_char`, and
  `is_standalone_graph_line`.
- Maintainability evidence: `src/jj_rows.rs` dropped from 760 lines after the workspace extraction
  to 282 lines after this packet, while the new `src/jj_rows/revisions.rs` is 498 lines including
  moved tests.
- Non-goals preserved: no rendered ANSI conversion changes, no graph row grouping behavior changes,
  no metadata drift fail-closed behavior changes, no compact log-context behavior changes, no
  resolve row or file-list row changes, and no process-boundary movement into `src/jj.rs`.
- Proof: focused `cargo test jj_rows -- --test-threads=1` and `cargo test graph -- --test-threads=1`
  passed during the extraction, with final gate commands recorded in `docs/process-observations.md`.

## Current Row Extraction Reassessment

The graph revision row extraction has now completed. Broad `src/jj_rows` source-shape work should
pause unless product work expands resolve rows, file-list rows, or metadata pairing enough to expose
a sharper owner.

The remaining candidates from the prior ledger are not good standalone packets right now:

- Resolve-entry parsing is tiny and has no sibling behavior beyond its template, parser, and three
  tests. Extracting it would create a small file but not reduce meaningful live context.
- File-list path preservation is even smaller: one item type, one loader, exact path preservation,
  and two focused tests. Wait until file-list behavior grows beyond "rendered row plus exact path."
- Shared JSON helpers are reused by bookmark, workspace, and resolve metadata parsing, but their
  current home is acceptable. A helper module would make readers jump for simple field accessors.
- Facade re-exports are intentional. They keep callers stable after row-family splits and should not
  become a separate source-shape packet.

## Other Large Files

- `src/graph.rs` is the largest production file at 1218 lines, but the direct read still shows one
  cohesive view: bindings, selection, search, refresh, log-mode switching, exact revision selection,
  copy options, graph-to-detail navigation, and graph action-menu opening. The long test tail mostly
  proves those view contracts. Do not split it only for size; revisit if explicit multi-revision
  selection or graph action-menu behavior grows enough to deserve a `graph/selection.rs`-style
  owner.
- `src/jj_actions.rs` is 1159 lines and already a facade over extracted bookmark, git-sync,
  operation, rewrite, and working-copy action clusters. The remaining parent owns
  description/commit, restore, file mutation, revert, and abandon plans. A future file-mutation or
  abandon-confirmation packet may be valid if product work changes those flows, but there is no
  docs-only reason to split it now.
- `src/tui.rs` is 976 lines after status hints moved out. It still owns shared chrome and overlay
  rendering. The only plausible future split is a concrete overlay family, such as action-output and
  abandon-confirmation rendering, with snapshot proof. Do not start a broad overlay split.
- `src/bookmarks.rs` is 973 lines, but production code is compact and the large tail is focused
  bookmark safety tests. The existing `src/bookmarks/action_targets.rs` boundary handles the
  nontrivial target-selection policy. Leave this alone until bookmark view behavior changes.

## Next Worker Guidance

The next source-shape worker should pause `src/jj_rows` refactoring and take product work unless a
new product change exposes a sharper row boundary. It should not extract resolve rows, file-list
rows, JSON helpers, facade re-exports, or large view modules merely to reduce line counts. It should
preserve rendered `jj` output, ANSI conversion, metadata drift behavior, graph-row grouping, and all
caller-facing row facades.
