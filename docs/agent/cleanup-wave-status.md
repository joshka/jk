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
  list, operation detail, workspaces, and resolve. This keeps production files shorter while
  preserving behavior tests next to the code they describe.
- App modal dispatch got its first reducer-shaped cleanup: copy, view, action, role, push-remote,
  and fetch-remote menu key handling now have named helpers while `handle_active_mode_key` remains
  the dispatch table.
- Action plan ownership improved: file action plans, operation recovery plans, and bookmark action
  plans have moved toward their owning concepts. This reduces the role of root action modules as
  mixed-purpose buckets.
- Per-packet evidence is current: each recent change records why it happened, what stayed unchanged,
  and the validation that backs the behavior-preserving claim.

## Why These Tasks Came First

- They were low-risk and behavior-preserving, so they could be split into reviewable jj changes.
- They directly support the feature-root direction: a maintainer should start from `files`,
  `operation_log`, `workspaces`, or `resolve` and find the nearby view behavior and tests.
- They reduce cognitive load before deeper app-dispatch work. Smaller feature modules make it easier
  to see whether shared reducers or helpers are genuinely useful.
- They are easy to validate with focused view-level tests plus `cargo check`, formatting, and
  Markdown checks.

## Current State

- The current top of stack refreshes this status map after the app modal key handler extraction.
- Recent behavior-preserving packets have focused on locality, feature ownership, and making the
  automatic session easier to audit from files rather than chat history.
- The broad goal is still active. The completed packets do not prove the whole cleanup queue is
  done.

## Likely Next Work

- App modal dispatch readability: `src/app/mode_input.rs` still has prompt and confirmation arms
  inline. The likely next valuable step is extracting named handlers for text prompts or abandon
  confirmation, but only where the names match existing concepts and preserve key behavior.
- Action lifecycle readability: `src/app/action_lifecycle/*` should stay focused on dispatch,
  preview, completion, refresh, and reveal policy. Repeated completion/result handling should be
  audited before extracting helpers.
- Remaining inline view tests: modules such as `status`, `operation_log`, `view_state`,
  `action_output`, and row/action helpers still have inline tests. These should move only when the
  split improves reader locality, not just because a file is large.
- Mechanical reports: largest files, broad visibility, inline-test modules, and repeated list
  mechanics should be measured periodically and treated as prompts for review rather than automatic
  refactor targets.
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
