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

Do not create a `slices/` folder or another implementation-phase bucket. The destination should look
like feature roots with optional submodules for local concerns, plus shared infrastructure for
boring cross-cutting mechanics:

- `app`: input, modes, navigation, services, and action lifecycle orchestration;
- `log`, `operation_log`, `bookmarks`, `status`, and `files`: feature-owned views, rows, action
  availability, target resolution, and tests;
- `documents`: rendered document mechanics such as sticky headings, rendered line structure, and
  document search when those mechanics are not owned by one file-oriented feature;
- `actions`: cross-view command plans and execution contracts such as rewrite, working-copy, file,
  sync, describe, and abandon plans;
- `jj`: command construction, process execution, syntax quoting, and view specs;
- `ui`: shared chrome, overlays, menus, status hints, and theme primitives.

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

- When deciding ownership, prefer product-language questions over code-shape questions. "What does
  the operation log show, select, copy, or recover from?" points to `operation_log`; "how do
  bookmark rows map to safe bookmark mutations?" points to `bookmarks`; "how do we quote a jj exact
  string pattern?" points to `jj`; "how is an action preview executed once a plan exists?" points to
  app action lifecycle; and "how does a modal list render?" points to shared UI.
- Operation-log behavior now starts from `operation_log`: `src/operation_log/rows.rs` owns rendered
  row grouping, operation-id template parsing and pairing, and fail-closed metadata drift tests;
  `src/operation_log.rs` owns movement/copy, undo/redo/restore/revert availability, operation detail
  navigation, and view tests; `src/operation_log/actions.rs` owns undo/redo and exact operation
  restore/revert argv, preview, and run contracts. Future operation-detail rendering should move
  toward `operation_log/detail.rs` when that shortens the reader path.
- Bookmark behavior now starts from `bookmarks`: `src/bookmarks/rows.rs` owns rendered row loading,
  bookmark metadata template parsing and pairing, local/remote state classification, and fail-closed
  drift tests; `src/bookmarks/action_targets.rs` owns safe mutation targets. Future bookmark
  create/set/move/rename/delete/forget/track/untrack availability belongs under the bookmark feature
  before a shared action plan exists.
- Cross-view action plans such as rebase, squash, absorb, new, edit, duplicate, split, restore,
  revert, track, untrack, chmod, fetch, push, describe, and abandon may live under an action-plan
  owner, but view-specific availability belongs with the feature that offers the action.
- `actions/*` should own argv, preview, and run behavior after a feature has already selected a
  command target. It should not own whether the log, status, bookmark, file, or operation-log view
  offers that action to the user.
- Rendered document scrolling, sticky file headings, and rendered jj document parsing may become a
  document feature owner when that lowers reader burden more than today's separate helper modules.

### Recent Packet Evidence

2026-05-21 help projection test split:

- `src/help.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/help/tests.rs` with `use super::*;` for private access to help projection helpers.
- This improves local readability by keeping generated help projection policy, grouping, context
  filtering, labels, and key text visible without scrolling through 264 lines of inline tests. The
  production file measured 641 lines before the split and 377 lines after it.
- The packet intentionally preserved all ten help projection test names, assertions, helper-free
  setup, imports, help grouping, labels, context filtering, key text, public API, and runtime
  behavior.
- Focused validation covered `cargo test help -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Main-thread review validation reran the same focused checks successfully with 23 passed and 544
  filtered out for `cargo test help -- --test-threads=1`.
- Rework was limited to applying Panache wrapping for the new process note; no help projection code,
  help UI behavior, keymap behavior, command metadata, visibility, or public API changed.

2026-05-21 jj file/content action-plan module split:

- Restore, revert, and file mutation command plans moved from `src/jj_actions.rs` to
  `src/jj_actions/files.rs`, with `src/jj_actions.rs` declaring `mod files;` and continuing to
  re-export `JjRestorePlan`, `JjRevertPlan`, `JjFileMutationPlan`, `JjFileMutationKind`,
  `JjFileMutationTarget`, and `JjFileChmodMode` for app-facing callers.
- The related restore/revert/track/untrack/chmod tests moved from `src/jj_actions/tests.rs` to
  `src/jj_actions/files/tests.rs` with their existing names and assertions preserved. Describe,
  commit, and abandon tests stayed in `src/jj_actions/tests.rs`.
- The packet intentionally preserved command argv, preview wording, exact revset/fileset quoting,
  run behavior, public re-export names, and app behavior. It did not change status/file-list/view
  action availability.
- Focused validation covered `cargo test jj_actions::files -- --test-threads=1` with 8 passed and
  559 filtered out, `cargo test jj_actions -- --test-threads=1` with 42 passed and 525 filtered out,
  `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check` after applying Panache
  wrapping.
- Main-thread review validation reran the same focused checks successfully. After the extraction,
  `src/jj_actions.rs` measures 397 lines, `src/jj_actions/files.rs` measures 491 lines,
  `src/jj_actions/tests.rs` measures 159 lines, and `src/jj_actions/files/tests.rs` measures 206
  lines.

2026-05-21 operation-log action-plan owner move:

- Operation recovery command plans moved from `src/jj_actions/operation.rs` to
  `src/operation_log/actions.rs`, keeping the existing inline tests with the moved module.
- `src/operation_log.rs` now declares `pub(crate) mod actions;`, while `src/jj_actions.rs` keeps
  re-exporting `JjOperationRecovery`, `JjOperationRecoveryKind`, and `JjOperationTarget` for app
  callers. This mirrors the bookmark action-plan owner move and keeps app imports stable while
  making undo/redo/restore/revert behavior discoverable from the operation-log feature root.
- The packet intentionally preserved command argv, command labels, preview wording, status action
  wording, run behavior, app-facing re-export names, key behavior, and operation-log view behavior.
- Focused validation covered `cargo test operation -- --test-threads=1` with 46 passed and 521
  filtered out, `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check` after
  applying Panache wrapping to the new process note.

2026-05-21 bookmark action-plan owner move:

- Bookmark mutation command plans moved from `src/jj_actions/bookmarks.rs` to
  `src/bookmarks/actions.rs`, with their existing sibling tests moved from
  `src/jj_actions/bookmarks/tests.rs` to `src/bookmarks/actions/tests.rs`.
- `src/bookmarks.rs` now declares `pub(crate) mod actions;`, while `src/jj_actions.rs` keeps
  re-exporting the app-facing bookmark action-plan boundary. This reduces caller churn while making
  bookmark create/set/move/rename/delete/forget/track/untrack behavior discoverable from the
  bookmark feature root.
- `src/bookmarks/action_targets.rs` now imports bookmark action target types from the sibling
  `actions` module instead of from the global `jj_actions` bucket.
- The packet intentionally preserved command argv, preview wording, exact-name quoting, rename
  validation, run behavior, test names, and public app-facing re-exports.
- Focused validation covered `cargo test bookmarks::actions -- --test-threads=1` with 8 passed and
  559 filtered out, `cargo test bookmarks -- --test-threads=1` with 40 passed and 527 filtered out,
  `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check`.

2026-05-21 feature-root refactoring guidance:

- `docs/agent/architecture.md` now records the target feature-root plus shared-infrastructure shape
  as an explicit direction rather than a `slices/` or kind-of-code folder migration.
- The guidance keeps feature policy with product owners: `operation_log`, `bookmarks`, `status`,
  `files`, and `log` should own view state, row interpretation, action availability, target
  resolution, and feature tests when those rules change with the user-visible surface.
- The guidance keeps shared infrastructure intentionally boring: `app` owns orchestration and
  lifecycle, `jj` owns process/command/syntax/view-spec boundaries, `actions` owns command plans
  after a feature has selected targets, and `ui` owns shared chrome and overlay rendering.
- The packet intentionally did not move Rust code. It documents the next refactoring direction so
  future source-shape packets have a concrete ownership test before moving behavior.
- Focused validation covered `just md-check` after applying Panache formatting.

2026-05-21 sticky file document test split:

- `src/sticky_file_view.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/sticky_file_view/tests.rs` with `use super::*;` for private access to rendered document
  helpers and `StickyFileDocument` internals.
- This improves local readability by keeping shared document rendering, sticky heading projection,
  no-wrap viewport handling, search, file navigation, and scroll mechanics visible without scrolling
  through 132 lines of inline tests. The production file measured 702 lines before the split and 568
  lines after it.
- The packet intentionally preserved all five sticky file view test names, assertions, helper
  functions, snapshots, rendering semantics, sticky heading behavior, no-wrap behavior, scroll
  behavior, visibility, and public API.
- Focused validation covered `cargo test sticky_file_view -- --test-threads=1` with 5 passed and 562
  filtered out, `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check` after
  this documentation update.
- Rework was limited to applying rustfmt's wrapping for one moved test assertion; no document
  rendering, sticky heading projection, no-wrap viewport handling, scroll behavior, visibility, or
  public API changed.

2026-05-21 command vocabulary test split:

- `src/command.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/command/tests.rs` with `use super::*;` for private access to command matching helpers.
- This improves local readability by keeping command vocabulary, key-label, binding matching, prefix
  matching, and help-filtering code visible without scrolling through 159 lines of inline tests. The
  production file measured 714 lines before the split and 553 lines after it.
- The packet intentionally preserved all eight command vocabulary test names, assertions, helper
  functions, key labels, binding matching, prefix matching, help filtering, visibility, and public
  API.
- Focused validation covered `cargo test command -- --test-threads=1` with 86 passed and 481
  filtered out, `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check` after
  this documentation update.
- Rework was limited to applying rustfmt's wrapping for one assertion in the moved test module and
  Panache wrapping for the new notes; no command behavior, key labels, binding matching, prefix
  matching, help filtering, visibility, or public API changed.

2026-05-21 jj command-boundary test split:

- `src/jj.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in `src/jj/tests.rs`
  with `use super::*;` for private access to command construction, color, output summarization,
  remote parsing, and exact-change-id helpers.
- This improves local readability by keeping the `jj` process boundary, argv construction, rendered
  output loading, color handling, and output summarization code visible without scrolling through
  296 lines of inline tests. The production file measured 773 lines before the split and 476 lines
  after it.
- The packet intentionally preserved all `jj` test names, assertions, helper functions, argv
  expectations, parsing behavior, output summarization behavior, color handling, visibility, and
  public API.
- Focused validation covered `cargo test jj::tests -- --test-threads=1` with 29 passed and 538
  filtered out, `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check` after
  this documentation update.
- Rework was limited to moving the extracted test block into a Rust child module and applying
  Panache wrapping to the new notes; no `jj` command behavior, argv construction, parsing, output
  summarization, color handling, visibility, or public API changed.

2026-05-21 ViewSpec test split:

- `src/jj/view_spec.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/jj/view_spec/tests.rs` with `use super::*;` for private access to ViewSpec parsing helpers
  and state.
- This improves local readability by keeping ViewSpec construction, label, diff-format, path, and
  navigation-target code visible without scrolling through 336 lines of inline tests. The production
  file now measures 460 lines.
- The packet intentionally preserved all ViewSpec test names, assertions, helper functions, labels,
  argv expectations, diff-format handling, navigation target semantics, visibility, and public API.
- Focused validation covered `cargo test jj::view_spec -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check` after this documentation update.
- Rework was limited to moving the extracted test block into a Rust child module and applying
  Panache wrapping to the new notes; no ViewSpec behavior, labels, argv, diff-format handling,
  navigation target semantics, visibility, or public API changed.

2026-05-21 working-copy action-plan test split:

- `src/jj_actions/working_copy.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live
  in `src/jj_actions/working_copy/tests.rs` with `use super::*;` for private access to command-plan
  helpers, split target inspection, and interactive command construction.
- This improves local readability by keeping working-copy creation, duplication, split, and
  navigation argv and preview-summary code visible without scrolling through 197 lines of inline
  tests. The production file measured 639 lines before the split and 441 lines after it.
- The packet intentionally preserved all ten working-copy action-plan test names, assertions, helper
  access, argv expectations, preview wording checks, labels, visibility, and public API.
- Focused validation covered `cargo test jj_actions::working_copy -- --test-threads=1` with 10
  passed and 557 filtered out, `cargo check`, `rustup run nightly cargo fmt --check`, and
  `just md-check` after this documentation update.
- Rework was limited to moving the extracted test block into a Rust child module; no working-copy
  command behavior, argv, preview summaries, labels, visibility, or public API changed.

2026-05-21 bookmark action-plan test split:

- `src/jj_actions/bookmarks.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/jj_actions/bookmarks/tests.rs` with `use super::*;` for private access to command-plan
  helpers and validation.
- This improves local readability by keeping bookmark mutation argv, preview-summary, exact-pattern,
  target, and rename-validation code visible without scrolling through 243 lines of inline tests.
  The production file measured 833 lines before the split and 588 lines after it.
- The packet intentionally preserved all eight bookmark action-plan test names, assertions, helper
  access, argv expectations, preview wording checks, exact pattern quoting, labels, visibility, and
  public API.
- Focused validation covered `cargo test jj_actions::bookmarks -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check` after this documentation update.
- Rework was limited to moving the extracted test block into a Rust child module and applying
  Panache wrapping to the new notes; no bookmark command behavior, argv, preview summaries, labels,
  or visibility changed.

2026-05-21 bookmark row test split:

- `src/bookmarks/rows.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests live in
  `src/bookmarks/rows/tests.rs` with `use super::*;` for private access to the owning module.
- This improves local readability by keeping bookmark row production policy visible without
  scrolling through 476 lines of inline tests. The production file measured 876 lines before the
  split and 398 lines after it.
- The packet intentionally preserved all ten bookmark row test names, assertions, helper functions,
  rendered-output pairing expectations, metadata parsing expectations, local/remote state
  expectations, visibility, labels, and public API.
- Focused validation covered `cargo test bookmarks -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check` after this documentation update.
- Rework was limited to formatting cleanup after the mechanical split left inline-module indentation
  and one leading blank line in the out-of-line sibling test module.

2026-05-21 revision action-menu test split:

- `src/action_menu/revision_actions.rs` now declares `#[cfg(test)] mod tests;`, and the moved tests
  live in `src/action_menu/revision_actions/tests.rs` with `use super::*;` for private access to the
  owning module.
- This improves local readability by keeping revision action-menu production policy visible without
  scrolling through 291 lines of inline tests. The production file measured 743 lines before the
  split and 447 lines after it.
- The packet intentionally preserved all seven revision action-menu test names, assertions, action
  ordering, labels, follow-ups, safety tiers, private helpers, and production visibility.
- Focused validation covered `cargo test action_menu -- --test-threads=1` with 40 passed and 527
  filtered out, `cargo check`, and `rustup run nightly cargo fmt --check` with existing rustfmt
  unstable-option warnings, plus `just md-check` after this documentation update.

2026-05-21 new trunk refresh flow:

- `src/app/action_lifecycle/preview.rs` now uses private `finish_new_trunk_success` for the
  successful trunk shortcut's refresh, recent-mode reveal, clamp, and fixed status-message path.
- `run_new_trunk` still owns trunk preflight, command execution through the app service, and
  resolving `@` before refresh. The helper intentionally ignores raw service output and does not use
  `finish_successful_action_revealing_change`, preserving the fixed status strings and avoiding
  operation-show output.
- Existing `working_copy_actions::graph_new_trunk_uses_test_service_and_reveals_working_copy` covers
  the recent-mode success status. Focused validation covered
  `cargo test working_copy_actions -- --test-threads=1`,
  `cargo test action_lifecycle -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.

2026-05-21 rewrite source context helper:

- `src/app/action_lifecycle/preview.rs` now uses private `with_rewrite_source_context` for the
  shared rebase/squash preview status-context source suffix.
- Rebase and squash call sites still own their base status context strings, command labels, preview
  calls, and mode variants. The helper owns only the shared short-source label suffix that app
  lifecycle preview already owned.
- Existing rewrite action tests assert the exact rebase and squash status context strings, including
  `| source(s): ...`. Focused validation covered `cargo test rewrite_actions -- --test-threads=1`,
  `cargo test mode_input -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.

2026-05-21 text prompt side-effect helper:

- `src/app/mode_input.rs` now uses a private `apply_text_prompt_accept_decision` helper for the
  repeated text-prompt accept shape: reset the app mode to normal, open the feature-specific preview
  for `PromptAcceptDecision::Preview`, or assign the status line for
  `PromptAcceptDecision::StatusMessage`.
- The four affected call sites still name their prompt-specific reducer and preview method:
  describe, commit, bookmark-name, and bookmark-rename prompts. Search and log-revset prompts stayed
  separate because their accept paths have distinct side effects.
- This packet intentionally preserved prompt reducers, status strings, cancellation wording,
  validation behavior, key handling, visibility, and preview methods. Focused validation covered
  `cargo test mode_input -- --test-threads=1`,
  `cargo test describe_commit_actions -- --test-threads=1`,
  `cargo test bookmark_actions -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.

2026-05-21 source audit measurement refresh:

- The mechanical audit snapshot now records current largest-file, visibility, repeated-list,
  action-lifecycle/result, control-flow, and immediate-doc scan counts gathered for this packet.
- The refreshed recommendation names `src/app/mode_input.rs` as the clearest bounded readability
  candidate from current measurements and avoids treating `src/app.rs` as an automatic documentation
  target because it already has run-level docs.
- The packet intentionally changed documentation only. Validation covered `just md-check`.

2026-05-21 command binding contract docs:

- `src/command.rs` now documents command recovery conversion, key sequence labels and prefix
  behavior, shifted printable-character matching, prefix fallback ownership, first-exact match
  preservation, next-key label deduplication, and app-owned interpretation of view effects.
- The docs preserve the current boundary: feature views own local binding availability and execution
  policy; `help.rs` owns help projection; `app.rs` owns prefix timing and command application;
  `command.rs` owns shared identity and matching mechanics.
- This packet intentionally changed comments only. Focused validation covered `cargo check`,
  `rustup run nightly cargo fmt --check`, `just md-check`, and
  `cargo test command -- --test-threads=1`.

2026-05-21 action lifecycle entry ownership docs:

- `src/app/action_lifecycle/entry.rs` now documents that action lifecycle entry owns routing from
  accepted menu/key actions to prompts, previews, or status messages.
- `AGENTS.md` and `docs/agent/architecture.md` now make the feature-policy versus shared-mechanics
  split explicit, including the rule that feature roots own what a surface shows, selects, copies,
  recovers from, and offers while shared modules own cross-cutting mechanics after feature policy is
  already chosen.
- The module docs preserve the current boundaries: feature views and action menus choose
  availability and exact targets; preview owns preview pane/status construction; completion/shared
  own confirmed result handling; `jj_actions` owns command-plan argv, preview, and run contracts.
- Short comments now mark why app lifecycle entry keeps modal reset, preview opening, and sync
  remote-list side effects beside otherwise pure prompt decisions.
- This packet intentionally changed comments and documentation only. Focused validation covered
  `cargo check`, `rustup run nightly cargo fmt --check`, `just md-check`, and
  `cargo test action_lifecycle -- --test-threads=1`. Full `just check` also passed at the top of the
  stack.

2026-05-21 sync remote prompt decision reducers:

- `src/app/action_lifecycle/entry.rs` now separates loaded remote-list branching from app side
  effects with private push/fetch prompt decision reducers.
- `entry.rs` still owns `load_git_remotes()`, status updates, mode changes, and preview opening. The
  reducers only classify empty, single, multiple, and error remote-list results.
- New `src/app/action_lifecycle/entry/tests.rs` tests the pure decision matrix and existing
  `sync_actions` tests continue to prove app-level prompt behavior, status text, and output panes.
- After the extraction, `src/app/action_lifecycle/entry.rs` measured 559 lines and
  `src/app/action_lifecycle/entry/tests.rs` measured 84 lines.
- Focused validation covered `cargo test sync_actions -- --test-threads=1`,
  `cargo test action_lifecycle -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`. Full `just check` also passed at the
  top of the stack.

2026-05-21 sync refresh completion helper:

- `src/app/action_lifecycle/shared.rs` now owns `finish_successful_sync_action`, the shared app-side
  refresh, clamp, status, and refresh-failure result contract for successful sync commands.
- `src/app/action_lifecycle/completion.rs` still owns push/fetch command execution, command labels,
  command-failure handling through `finish_failed_action`, and final preview-mode assignment.
- The helper keeps the feature-specific status wording outside the shared mechanic by accepting a
  status-message builder: push uses raw output and fetch uses `fetch_status_message`.
- After the extraction, `src/app/action_lifecycle/completion.rs` measured 437 lines and
  `src/app/action_lifecycle/shared.rs` measured 192 lines.
- Focused validation covered `cargo test sync_actions -- --test-threads=1`,
  `cargo test mode_input -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`. Full `just check` also passed at the
  top of the stack.

2026-05-21 mode input reducer test split:

- `src/app/mode_input/reducers.rs` now declares its tests out-of-line in
  `src/app/mode_input/reducers/tests.rs`, keeping production reducer contracts visible without
  scrolling through the growing behavior matrix.
- The split is behavior-neutral: the same 12 reducer tests still cover role prompt, describe,
  commit, bookmark-name, and bookmark-rename prompt decisions.
- After the move, `src/app/mode_input/reducers.rs` measured 290 lines and
  `src/app/mode_input/reducers/tests.rs` measured 162 lines.
- Focused validation covered `cargo test mode_input -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`. Full `just check` also passed at the
  top of the stack.

2026-05-21 role prompt acceptance reducer extraction:

- `src/app/mode_input/reducers.rs` now owns the pure role-prompt acceptance decision:
  `RolePromptDecision` classifies rebase previews, squash previews, normal status messages, and
  error status messages without mutating app state.
- `src/app/mode_input.rs` still owns the side effects after prompt acceptance: reset to normal mode,
  open rewrite previews, and update `StatusLine`.
- The extraction follows the app shared-infrastructure rule: reducers own modal interpretation,
  while action lifecycle and status routing stay with app orchestration.
- Focused validation covered `cargo test command_navigation -- --test-threads=1`,
  `cargo test rewrite_actions -- --test-threads=1`, `cargo test mode_input -- --test-threads=1`,
  `cargo check`, `rustup run nightly cargo fmt --check`, and `just md-check`. Full `just check` also
  passed at the top of the stack.

2026-05-21 text prompt acceptance reducer extraction:

- `src/app/mode_input/reducers.rs` now owns pure accept decisions for describe, commit, bookmark
  name, and bookmark rename prompts. The prompt-specific reducers return either a preview plan or
  the exact cancellation status wording.
- `src/app/mode_input.rs` still owns the app side effects after text-prompt acceptance: reset to
  normal mode, open the relevant preview, or assign `StatusLine`.
- Main-thread review collapsed four identical decision enum shapes into one narrow
  `PromptAcceptDecision<T>` so prompt-specific function names carry the concept while the shared
  result shape stays boring.
- Focused validation covered `cargo test mode_input -- --test-threads=1`,
  `cargo test describe_commit_actions -- --test-threads=1`,
  `cargo test bookmark_actions -- --test-threads=1`, `cargo check`,
  `rustup run nightly cargo fmt --check`, and `just md-check`. Full `just check` also passed at the
  top of the stack.

2026-05-21 action plan root contract documentation:

- `src/jj_actions.rs` now documents the root action-plan ownership boundary: root plans own argv
  labels, argv construction, preview summaries, and direct execution envelopes; family submodules
  own narrower action families; feature views/action menus own availability and target selection;
  app lifecycle owns prompt flow, confirmation strength, refresh/reveal policy, and result-screen
  transitions.
- `CommandOutput` now documents that it carries presentation-ready preview or execution output so
  callers preserve `jj` wording instead of reparsing output or inferring state transitions.
- Main-thread review trimmed repetitive comments that restated similarly shaped method names,
  keeping the contracts around exact revsets/filesets, forward-diff preview honesty, and no
  simulation of jj results.
- Focused validation covered `cargo check`, `cargo test jj_actions -- --test-threads=1`,
  `rustup run nightly cargo fmt --check`, and `cargo doc --no-deps`.

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

Snapshot date: 2026-05-21. The current measurements below were gathered by the main thread for the
`Refresh source audit measurements` packet. Rerun these commands before using the measurements for
new work.

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

The largest files reported by `just largest-rust-files` were:

```text
1196 src/app/tests/bookmark_actions.rs
 876 src/bookmarks/rows.rs
 872 src/jj_actions.rs
 833 src/jj_actions/bookmarks.rs
 797 src/jj/view_spec.rs
 778 src/app/tests/working_copy_actions.rs
 773 src/jj.rs
 743 src/action_menu/revision_actions.rs
 714 src/command.rs
 702 src/sticky_file_view.rs
 659 src/app_screen.rs
 643 src/bookmarks/tests.rs
 642 src/help.rs
 639 src/jj_actions/working_copy.rs
 613 src/graph/tests.rs
 613 src/diff.rs
 605 src/graph.rs
 597 src/rendered_jj.rs
 596 src/app/tests/command_navigation.rs
 592 src/tui.rs
```

The same report also listed large test files. Those test sizes are evidence to read the surrounding
contracts, not evidence to split production code.

### Rustdoc Coverage

The named central files all have module docs. The mechanical immediate-doc scan below is a snapshot,
not the current recommended work queue. Later packets in this ledger added contract docs for several
of the originally nominated files, so use the bullets as historical evidence of why those packets
happened and rerun the scan before opening a new documentation packet.

- `src/main.rs`: module docs are enough for the binary entry point; `run` lives in `src/app.rs`.
- `src/app.rs`: current run-level docs already explain the app orchestration boundary. It is not
  automatically a documentation target just because it still appears in some mechanical scans.
- `src/app_screen.rs`: stale nomination. Later packet evidence documents `InteractionMode`,
  `ViewMenuOption`, and `view_menu_options` ownership around prompt status projection, borrowed
  overlays, static view-menu data, clamping, and app-owned dispatch.
- `src/command.rs`: stale as a generic sweep nomination. Later packet evidence documents command
  vocabulary, binding metadata, key-pattern matching, prefix fallback ownership, label behavior, and
  app-owned interpretation of view effects. Rerun a focused scan only when changing binding
  constructors, prefix matching, help projection, or command-context contracts.
- `src/action_menu.rs`: stale as a generic sweep nomination. Later packet evidence documents shared
  menu presentation, stable action vocabulary, prompts, follow-up payload boundaries, feature-owned
  availability, app-owned lifecycle, and `jj_actions` command plans.
- `src/tui.rs`: stale as a generic sweep nomination. Later packet evidence documents the shared
  chrome boundary, optional status hints, borrowed overlays, action-output sizing, confirmation
  input rendering, fallback-friendly overlay styling, and clipped modal geometry.
- `src/jj_actions.rs`: stale as a generic sweep nomination. Later packet evidence documents root
  action-plan ownership, `CommandOutput`, exact revsets/filesets, forward-diff preview honesty, and
  the boundary between feature availability, app lifecycle, and argv/preview/run contracts.
- `src/jj_rows.rs`: stale as a generic sweep nomination. Later packet evidence documents the shared
  rendered-row helper boundary after feature-owned row migrations, including plain-text flattening,
  metadata drift handling, JSON field extraction, graph-line detection, rendered line extraction,
  fail-closed metadata parsing, and intentional style loss.

This snapshot still supports small documentation work where current code requires a maintainer to
reconstruct visibility, side effects, ids, argv shape, or output-shape assumptions. It no longer
supports a broad weak-doc sweep over `app_screen.rs`, `command.rs`, `action_menu.rs`, `tui.rs`,
`jj_actions.rs`, or `jj_rows.rs` without fresh evidence.

Current immediate-doc scan counts for the named central files were:

```text
 86 src/action_menu.rs
 16 src/app.rs
 29 src/app_screen.rs
 82 src/command.rs
144 src/jj_actions.rs
 31 src/jj_rows.rs
  3 src/main.rs
 31 src/tui.rs
```

### Visibility Shape

Broad visibility counts across `src`:

```text
1220 pub total
 118 pub(crate)
 140 pub(super)
```

The highest per-file broad-visibility counts were:

```text
112 src/app/services.rs
102 src/app/tests/support.rs
 84 src/jj_actions.rs
 47 src/jj_actions/working_copy.rs
 44 src/sticky_file_view.rs
 35 src/jj_actions/bookmarks.rs
 32 src/action_menu.rs
 30 src/jj/view_spec.rs
 29 src/command.rs
 28 src/show.rs
 28 src/graph.rs
 28 src/file_show.rs
 28 src/diff.rs
 27 src/jj_actions/rewrite.rs
 25 src/status.rs
```

Use these counts to audit boundary clarity and documentation. Do not narrow visibility just to move
the count.

### Repeated List Mechanics

Selection, restore, movement, `ListState`, and clamp evidence across list-style views:

```text
87 src/graph.rs
60 src/status.rs
49 src/operation_log.rs
47 src/file_list.rs
44 src/resolve.rs
42 src/workspaces.rs
41 src/bookmarks.rs
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
144 src/app/action_lifecycle/preview.rs
102 src/app/action_lifecycle/entry.rs
 87 src/app/action_lifecycle/completion.rs
 61 src/jj_actions.rs
 55 src/app/action_lifecycle/rewrite_completion.rs
 25 src/app/action_lifecycle/shared.rs
  4 src/app/action_lifecycle/entry/tests.rs
```

The clear owner for app-side completion and preview policy is `src/app/action_lifecycle`, especially
`preview.rs`, `entry.rs`, `completion.rs`, and `shared.rs`. `src/jj_actions.rs` owns plan-side argv,
preview text, and execution helpers. A future packet should improve documentation or grouping at
that boundary only if it can name the side-effect boundary it is protecting, such as which side owns
result wording, refresh/reveal behavior, or command-output preservation. Do not merge app lifecycle
policy into mutation plans.

### Control-Flow Hot Spots

Cheap nested/control-flow counts nominated these files for readability review:

```text
41 src/app/mode_input.rs
27 src/command.rs
27 src/app/action_lifecycle/completion.rs
25 src/jj_actions.rs
24 src/help.rs
23 src/graph.rs
23 src/app/action_lifecycle/entry.rs
23 src/app.rs
22 src/status.rs
17 src/workspaces.rs
17 src/tui.rs
17 src/jj/view_spec.rs
17 src/app/action_lifecycle/preview.rs
16 src/view_state.rs
16 src/action_menu/revision_actions.rs
15 src/sticky_file_view.rs
15 src/jj.rs
14 src/app/navigation.rs
12 src/app/action_lifecycle/shared.rs
11 src/jj_actions/bookmarks.rs
10 src/resolve.rs
10 src/jj_actions/working_copy.rs
10 src/app/action_lifecycle/rewrite_completion.rs
 9 src/view_action_targets.rs
 9 src/operation_log.rs
```

The current scan puts `src/app/mode_input.rs` first again, ahead of `src/command.rs`,
`src/app/action_lifecycle/completion.rs`, and `src/jj_actions.rs`. Combined with the completed-doc
context, that keeps `src/app/mode_input.rs` as the clearest bounded readability candidate in this
snapshot. `src/app.rs` already has run-level docs and is not automatically a documentation target.

Later packet evidence partly supersedes this snapshot: `src/app/mode_input/reducers.rs` now owns
pure prompt acceptance decisions and out-of-line reducer tests, while `src/app/mode_input.rs` still
owns prompt side effects. Treat any follow-up as a fresh read of the current app mode-input
boundary, not as an automatic continuation of the original control-flow count.

## Completed Source-Shape Context

Recent completed packets already moved several coherent owners out of broad modules:

- help projection policy into `src/help.rs`;
- path and revision action-menu policy into `src/action_menu/path_actions.rs` and
  `src/action_menu/revision_actions.rs`;
- operation, bookmark, workspace, resolve, file-list, and graph/log row loading into feature-owned
  `rows.rs` modules;
- ViewSpec navigation provenance into `src/jj/view_spec.rs`;
- status hint projection into `src/tui/status_hints.rs`;
- pure modal key reducers and prompt-plan helpers into `src/app/mode_input/reducers.rs`.
- central ownership and contract docs for `src/app_screen.rs`, `src/command.rs`,
  `src/action_menu.rs`, `src/tui.rs`, `src/jj_actions.rs`, and `src/jj_rows.rs`.

Those packets improved local contracts, but several are still organized by kind of code rather than
by user-visible feature. Treat them as staging points, not the final product shape:

- `src/jj_rows.rs` now mostly owns shared rendered-row helpers. Re-nominate it only with fresh
  evidence that a helper boundary is unclear or that remaining shared mechanics are masking feature
  policy.
- `src/jj_actions/*` proved command-plan boundaries, but action availability and target policy
  should move toward feature roots rather than stay in global action-menu planning.
- `src/action_menu/*` should shrink over time toward shared menu vocabulary and presentation; the
  feature deciding that an action is available should own that decision.

Do not move code only to match a destination tree. Move it when a packet can preserve behavior, name
the owning product concept, and make the reader path shorter.

## Next Packet Recommendations

Recommended bounded candidates:

1. Mode-input boundary review is the strongest current measured candidate. Reread the current
   reducer split first, then name one remaining prompt-side-effect or status-wording rule that
   belongs in app orchestration versus reducers.
1. Fresh measurement pass before choosing any different packet. Rerun the audit commands and compare
   them with the completed documentation and row-migration evidence above so future work starts from
   current code instead of this snapshot.
1. App entry-point contract docs only with fresh evidence that the current `src/app.rs` surface
   still forces readers to reconstruct terminal lifecycle, event-loop ownership, refresh/reveal
   orchestration, or cross-view dispatch from call sites. Current run-level docs mean `src/app.rs`
   is not automatically a documentation target.
1. Action lifecycle preview/entry review only with a named side-effect boundary, such as
   preview-pane construction versus mode/status mutation, command-output preservation versus result
   wording, or prompt routing versus remote-list loading.
1. Feature-owned action availability packets when a product feature changes, especially around
   status, files, bookmarks, operation log, or graph/log actions. Feature roots should own whether
   an action is offered and which target it selects; `action_menu` should keep shared vocabulary and
   presentation, while `jj_actions` should keep argv, preview, and execution contracts.
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
