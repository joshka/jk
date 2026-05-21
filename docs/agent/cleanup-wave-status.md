# Cleanup Wave Status

This document summarizes the current maintainability cleanup wave in plain product terms. It is a
snapshot for humans and future agents; detailed per-packet evidence stays in
[`source-maintainability-ledger.md`](source-maintainability-ledger.md), and process details stay in
[`../process-observations.md`](../process-observations.md).

## Recent Work

- Feature roots for files: file list and file show now live under `files`. This makes file-view
  behavior easier to find when changing file navigation, copy, search, refresh, or display behavior.
- Feature ownership for operation detail: operation show/diff detail moved under `operation_log`.
  This makes operation-log navigation, recovery actions, and operation detail views start from one
  feature root.
- View tests moved beside their features: tests moved out of production modules for file show, file
  list, operation detail, operation log, view-state target routing, action output, workspaces, and
  resolve. Shared interactive-process tests also moved beside their owner. This keeps production
  files shorter while preserving behavior tests next to the code they describe.
- App modal dispatch got its first reducer-shaped cleanup: copy, view, action, role, push-remote,
  and fetch-remote menu key handling now have named helpers while `handle_active_mode_key` remains
  the dispatch table.
- App text prompts now follow the same pattern: search, log revset, describe, commit, bookmark
  create/move/set, and bookmark rename prompt handling moved into named helpers.
- App abandon preview and confirmation handling now also have named modal helpers, keeping empty
  abandon rechecks, non-empty confirmation, cancellation, and output-closing behavior out of the
  main dispatch table.
- Central source ownership contracts were tightened in `main`, `app`, `app_screen`, `command`,
  `action_menu`, `tui`, `jj_actions`, and `jj_rows`. This makes future cleanup less dependent on
  chat context because the source now states which module owns process setup, dispatch, modal
  projection, shared chrome, command metadata, action plans, and rendered-row helpers.
- The first module-layout conversion moved small existing `foo.rs` plus `foo/` pairs to `foo/mod.rs`
  for `action_output`, `app_screen`, `command`, `diff`, `help`, `interactive_process`,
  `rendered_jj`, `show`, `status`, `sticky_file_view`, `view_state`, and `workspaces`.
- The second module-layout conversion moved narrow nested roots to `mod.rs` for
  `action_menu/revision_actions`, `app/action_lifecycle/entry`, `app/mode_input/reducers`,
  `bookmarks/actions`, `bookmarks/rows`, `jj/view_spec`, `jj_actions/files`,
  `jj_actions/working_copy`, and `operation_log/detail`.
- The third module-layout conversion moved coherent file/resolve roots to `mod.rs` for `files`,
  `files/list`, `files/show`, `resolve`, and `app/tests`.
- `jj_actions` now has a real table-of-contents root: describe/commit plans moved under
  `jj_actions/describe`, abandon plans moved under `jj_actions/abandon`, and the root keeps
  `CommandOutput` plus public action-plan re-exports.
- `bookmarks` now has a real feature root: the root is a table of contents, bookmark view behavior
  lives in `bookmarks/view.rs`, rows and action-target policy stay under the same feature, and
  callers still use `crate::bookmarks::BookmarksView`.
- Action plan ownership improved: file action plans, operation recovery plans, and bookmark action
  plans have moved toward their owning concepts. This reduces the role of root action modules as
  mixed-purpose buckets.
- Per-packet evidence is current: each recent change records why it happened, what stayed unchanged,
  and the validation that backs the behavior-preserving claim.

## Short Work Map

- File views moved under a file feature root. This makes day-to-day file actions easier to change
  because file list behavior, file show behavior, and their tests now start from one obvious place.
- Operation detail moved under the operation-log feature. This supports undo, redo, restore, revert,
  and operation inspection work by keeping recovery-related behavior near the operation log.
- View tests moved out of production modules. This shortens production files while keeping the tests
  beside the feature they prove, so future behavior changes have nearby evidence. Operation-log view
  behavior, view-state target routing, action-output modal state, and the interactive-process
  terminal boundary are now in that shape too.
- App modal key handling is being simplified. Copy/view/action menus, text prompts, and abandon
  confirmation now have named handlers, which makes the main keyboard dispatch read more like a map
  of user modes instead of a long implementation block.
- Action command plans are moving toward their owning feature. This keeps feature-specific decisions
  such as bookmark targets, file actions, and operation recovery away from mixed global buckets.
- Source-shape audit is now tracked. Largest files, remaining inline tests, visibility count, and
  app dispatch complexity are recorded so the next cleanup target is chosen from evidence rather
  than guesswork.
- Upcoming cleanup should focus on measured reader pain: app action lifecycle, remaining inline
  feature tests, status and operation-log view ownership, graph contracts, and shared UI chrome.
- Product work is still waiting behind the cleanup wave. The target product scope remains practical
  `jj` TUI workflows such as abandon, undo/redo, operation-log movement, multi-parent `jj new`,
  clearer push/fetch flows, status/file actions, bookmarks, rebase, absorb, squash, and user-facing
  README/tutorial material.

## Why These Tasks Came First

- They were low-risk and behavior-preserving, so they could be split into reviewable jj changes.
- They directly support the feature-root direction: a maintainer should start from `files`,
  `operation_log`, `workspaces`, or `resolve` and find the nearby view behavior and tests.
- They reduce cognitive load before deeper app-dispatch work. Smaller feature modules make it easier
  to see whether shared reducers or helpers are genuinely useful.
- They are easy to validate with focused view-level tests plus `cargo check`, formatting, and
  Markdown checks.

## Current State

- The current top of stack splits the bookmark feature root after the root `jj_actions` action-plan
  split.
- Recent behavior-preserving packets have focused on locality, feature ownership, and making the
  automatic session easier to audit from files rather than chat history.
- The broad goal is still active. The completed packets do not prove the whole cleanup queue is
  done.

## Likely Next Work

- Module layout cleanup: continue applying the epage Rust style rule to existing split modules.
  Larger roots such as `app`, `operation_log`, `graph`, `action_menu`, `jj`, and `tui` should move
  toward table-of-contents `mod.rs` files through topical splits, not blind path moves.
- Remaining `foo.rs` plus `foo/` pairs after the first conversion are mostly larger roots or nested
  roots with feature/action policy: `action_menu`, `app`, `app/action_lifecycle`, `app/mode_input`,
  `graph`, `jj`, `operation_log`, and `tui`.
- App modal dispatch readability: `src/app/mode_input.rs` now mostly reads as a dispatch table plus
  named modal handlers. The next app-dispatch work should be based on measured remaining complexity,
  not another automatic extraction.
- Action lifecycle readability: `src/app/action_lifecycle/*` should stay focused on dispatch,
  preview, completion, refresh, and reveal policy. Repeated completion/result handling should be
  audited before extracting helpers.
- Remaining inline tests: row/action helpers and shared process helpers still have inline tests.
  These should move only when the split improves reader locality, not just because a file is large.
- Mechanical reports: largest files, broad visibility, inline-test modules, and repeated list
  mechanics are tracked in [`source-cleanup-audit.md`](source-cleanup-audit.md) and should be
  treated as prompts for review rather than automatic refactor targets.
- Documentation sweep: central modules such as `app.rs`, `app_screen.rs`, `command.rs`, `tui.rs`,
  `jj_actions.rs`, and `jj_rows.rs` should keep concise ownership contracts explaining where future
  behavior belongs.

## Process Observations

- Bounded workers have worked well for mechanical moves with clear write sets, especially test
  splits. The main thread should keep owning jj changes, review, validation, and next-slice choice.
- The most common rework has been Markdown wrapping and occasional mechanical path replacement.
  Focused tests and `just md-check` have caught those quickly.
- The cleanup is currently prioritizing reader locality and feature ownership over abstract helper
  extraction. Shared helpers should wait until repeated behavior is well understood and clearly
  domain-neutral.
- Behavior-preserving packets should keep saying exactly what did not change: rendered `jj` output,
  command argv, status wording, selection behavior, key behavior, refresh/reveal behavior, and test
  assertions.
- Automatic work needs human-readable status summaries because packet names can be too close to code
  structure. Keep this file at the product/task level, and leave implementation details in the
  ledger and process log.
