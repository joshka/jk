# Slice Progress

## Packet Clippy Baseline Cleanup

- Slice / task: remove known clippy baseline blockers (`dead_code` and `collapsible_if`) and keep
  behavior unchanged.
- Files changed: `src/app/navigation.rs`, `src/bookmarks.rs`, `src/graph.rs`,
  `src/operation_log.rs`, `src/jj_rows.rs`
- Validation: `cargo check`; `cargo clippy -- -D warnings`; focused
  `cargo test bookmarks -- --test-threads=1`; focused `cargo test file_list -- --test-threads=1`;
  focused `cargo test graph -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt --check`; `cargo test operation_log -- --test-threads=1`;
  `just check`; `just md-check`.
- Residual risk: no behavior change; remaining risk only if parser contracts shift (not touched).

## Slice 0: Source Integration Spike

- Files changed: `docs/plan/integration-feasibility.md`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`
- Verification: temporary scratch crate compiled and ran against adjacent `../jj/cli` and
  `../jj/lib`; compared `jj log` default output, ASCII graph style, and a custom log template;
  `just md-check`
- Remaining risk: `jj_cli` rendering pieces are public, but end-to-end workspace and command setup
  still requires awkward external wiring or copied orchestration
- Next slice: Slice 1: Log Row Contract, using the narrowed subprocess-plus-metadata path

## Slice 1: Log Row Contract

- Files changed: `src/jj.rs`, `src/graph.rs`, `docs/plan/progress.md`
- Verification: focused `cargo test restore_selection`,
  `cargo test converts_ansi_output_to_selectable_items`, full `cargo test`, and
  `rustup run nightly cargo fmt`
- Remaining risk: refresh preservation is keyed only by change id, so rows without a parsed change
  id still fall back to index clamping by design
- Next slice: Slice 2: View Mode Infrastructure

## Slice 2: View Mode Infrastructure

- Files changed: `src/app.rs`, `src/command.rs`, `src/diff.rs`, `src/graph.rs`, `src/jj.rs`,
  `src/show.rs`, `src/tui.rs`, `src/view_state.rs`, `docs/plan/progress.md`
- Verification: full `cargo test` before and after `rustup run nightly cargo fmt`, then
  `just md-check`
- Remaining risk: custom revset entry now exists through a lightweight graph-only prompt (`W`), but
  it does not yet offer history, editing helpers, or generated help text
- Next slice: Slice 3: Generated Help and Keymap

## Slice 3: Generated Help And Keymap

- Files changed: `src/app.rs`, `src/command.rs`, `src/tui.rs`, `src/view_state.rs`,
  `docs/plan/progress.md`
- Verification: full `cargo test` before and after `rustup run nightly cargo fmt`, including new
  help-projection and snapshot-style overlay tests, then `just md-check`
- Remaining risk: the status bar still uses concise handwritten hint text, while the help overlay is
  now the generated source of truth for exact bindings
- Next slice: Slice 4: Direct `jj git fetch`

## Slice 4: Direct `jj git fetch`

- Files changed: `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/tui.rs`, `docs/plan/progress.md`
- Verification: full `cargo test` before and after `rustup run nightly cargo fmt`; disposable-repo
  manual `jj --no-pager git fetch` run with signing disabled in the temporary Git repo;
  `just   md-check`
- Remaining risk: fetch output is summarized into the one-line status area rather than preserved in
  a dedicated output view, so unusually verbose fetch output may be harder to inspect
- Next slice: Slice 5: Direct `jj new trunk`

## Slice 5: Direct `jj new trunk`

- Files changed: `src/app.rs`, `src/command.rs`, `src/diff.rs`, `src/graph.rs`, `src/jj.rs`,
  `src/show.rs`, `src/tui.rs`, `src/view_state.rs`, `docs/plan/progress.md`
- Verification: full `cargo test`, then review-driven follow-up tests for the graph visibility
  fallback, another full `cargo test`, `rustup run nightly cargo fmt`, disposable-repo manual
  `jj --no-pager new 'trunk()'` run after cloning a temporary remote with a configured `main`
  branch, and `just md-check`
- Remaining risk: the exact-target validation and mode fallback are covered by helper tests and
  manual proof, but not yet by an app-level mocked command-runner test around the whole direct
  action path
- Next slice: Slice 6: Status Screen First Pass

## Slice 6: Status Screen First Pass

- Files changed: `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/main.rs`, `src/status.rs`,
  `src/tui.rs`, `src/view_state.rs`, `docs/plan/implementation-slices.md`,
  `docs/plan/screens/status.md`, `docs/plan/progress.md`
- Verification: focused `cargo test status::`, `cargo test parses_status_startup_view`, and
  `cargo test help_overlay_text_renders_generated_sections`; full `cargo test`;
  `rustup run nightly cargo fmt`; `markdownlint-cli2 docs/plan/progress.md`
- Remaining risk: the native status screen now has a dedicated shortcut and direct `jk status`
  startup path, but command-mode entry remains deferred because that app surface is not yet present,
  and recommended planning keeps command mode secondary
- Next slice: Slice 7: Operation Log First Pass

## Slice 7: Operation Log First Pass

- Files changed: `src/main.rs`, `src/jj.rs`, `src/command.rs`, `src/view_state.rs`, `src/app.rs`,
  `src/tui.rs`, `src/operation_log.rs`, `docs/plan/implementation-slices.md`,
  `docs/plan/screens.md`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`
- Verification: focused `cargo test operation_log`, focused
  `cargo test jj::tests::groups_operation_log_rows_and_preserves_styles`, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and
  `panache format --check README.md docs`
- Remaining risk: command-mode entry remains deferred because that app surface is still absent,
  operation show/diff remain explicit placeholders until a dedicated detail or preview design lands,
  and exact operation ids currently depend on pairing rendered `operation log` rows with a separate
  `self.id()` template stream in the same row order under `--at-op=@`
- Next slice: Slice 8: Bookmark List First Pass

## Slice 8: Bookmark List First Pass

- Files changed: `src/main.rs`, `src/jj.rs`, `src/command.rs`, `src/view_state.rs`, `src/app.rs`,
  `src/tui.rs`, `src/bookmarks.rs`, `docs/plan/implementation-slices.md`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`
- Verification: focused `cargo test bookmarks`, focused
  `cargo test bookmark_list_command_uses_bookmark_words_and_labels`,
  `cargo test parses_bookmark_metadata_lines`, `cargo test pairs_bookmark_rows_in_render_order`,
  `cargo test bookmark_rows_allow_missing_and_extra_metadata`
- Remaining risk: command-mode entry remains deferred because that app surface is still absent, and
  exact bookmark names and target ids currently depend on pairing rendered local bookmark rows with
  a separate metadata template stream by row order while remote/tracking semantics stay deliberately
  non-semantic in this first pass
- Next slice: Slice 9: File List And File Show

## Slice 9: File List And File Show

- Files changed: `src/app.rs`, `src/command.rs`, `src/diff.rs`, `src/file_list.rs`,
  `src/file_show.rs`, `src/jj.rs`, `src/main.rs`, `src/show.rs`, `src/status.rs`, `src/tui.rs`,
  `src/view_state.rs`, `src/graph.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/implementation-slices.md`, `docs/plan/progress.md`, `docs/plan/screens.md`,
  `docs/plan/screens/files.md`
- Verification: `panache format --check docs/plan/progress.md` and `markdownlint-cli2` run on
  `docs/plan/fragility-register.md`, `docs/plan/implementation-slices.md`, `docs/plan/progress.md`,
  `docs/plan/screens.md`, `docs/plan/screens/files.md`
- Remaining risk: file-list identity still comes from rendered path text, so any `jj file list`
  formatting changes can affect exact-path extraction and selection semantics until structured or
  templated output is introduced
- Next slice: Slice 10: Action Menu And Multi-Select

## Slice 10: Action Menu And Multi-Select

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/command.rs`, `src/graph.rs`,
  `src/main.rs`, `src/tui.rs`, plus view files updated for exhaustive-match arms and compile-time
  completeness checks in the new action-flow types
- Verification: `cargo check` and `cargo test`
- Remaining risk: action execution is currently preview-only and intentionally excludes mutation
  commands, so preview/review and explicit-role confirmation flows can be bypassed only by later
  slices; multi-select state is scoped to graph-based exact-change-id targeting and may still need
  additional pruning logic if downstream views carry stricter per-mode selection semantics
- Next slice: Slice 11: Push Preview Flow

## Slice 11: Push Preview Flow

- Files changed: `src/app.rs`, `src/jj.rs`, `src/tui.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo test push_preview`, `cargo test git_push`, manual
  `jj --no-pager git remote list`, and `just md-check`
- Remaining risk: status-driven push targets still rely on jj's default push resolution for the
  chosen remote, and the preview/result text still comes from `jj` CLI output, so future
  output-shape drift could require a parser or contract change
- Next slice: Slice 12: Rebase Preview Flow

## Slice 12: Rebase Preview Flow

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo check`; focused `cargo test rebase -- --nocapture`; full `cargo test`;
  disposable-repo manual `jj --no-pager rebase -r <source> -o <dest>` run followed by
  `jj --no-pager undo`; `rustup run nightly cargo fmt --check`; `just md-check`
- Remaining risk: the preview is honest about explicit roles, command shape, and undo path, but it
  still summarizes graph effect textually instead of rendering a simulated before/after graph, and
  long preview/result output remains unscrollable in a small terminal
- Next slice: Packet 13: Scrollable Action Output Overlay

## Packet 13: Scrollable Action Output Overlay

- Files changed: `src/action_output.rs`, `src/app.rs`, `src/main.rs`, `src/tui.rs`,
  `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo check`; focused `cargo test action_output`, `cargo test push_preview`, and
  `cargo test rebase_preview`; full `cargo test`; `rustup run nightly cargo fmt`; `just md-check`
- Remaining risk: action output is now scrollable for the active push/rebase preview or result, but
  there is still no persistent output history after the overlay is closed, and direct fetch output
  remains status-only until a later packet chooses to route direct actions through the same surface
- Next slice: Packet 14: Declutter Status Bar

## Packet 14: Declutter Status Bar

- Files changed: `src/app.rs`, `src/tui.rs`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo test`, `cargo test tui -- --nocapture`, `cargo check`,
  `rustup run nightly cargo fmt`, `just md-check`
- Remaining risk: the status bar now favors message visibility and a small curated hint set, but the
  compact hint mix is intentionally conservative and may still need per-view tuning if later
  terminal work wants a different balance
- Next slice: Packet 15: General Abandon From Exact Change Targets

## Packet 15: General Abandon From Exact Change Targets

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo check`; focused `cargo test abandon -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; disposable-repo manual
  `jj --no-pager abandon <change-id>` for one empty change and one non-empty change under
  `/tmp/jk-packet15-proof.7gHoJv`, each followed by `jj --no-pager undo`; `just md-check`
- Validation note: `just check` was attempted, but the local wrapper stopped at `cargo +nightly fmt`
  with `no such command: +nightly`; the equivalent `rustup run nightly cargo fmt --check` passed
- Remaining risk: the flow is exact for graph single-row targets and blocks
  selected-source/multi-target abandon, but empty-versus-non-empty detection still depends on
  `jj diff -r <revision> --summary` stdout and the preview title depends on a narrow
  `description.first_line()` template
- Next slice: Packet 15: 5.5 Review Repair

## Packet 15: 5.5 Review Repair

- Files changed: `src/app.rs`, `src/jj.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo test abandon -- --test-threads=1`; focused app and `jj` abandon command shape
  tests; `cargo check`; full `cargo test`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `jj --no-pager help -k revsets`;
  `jj --no-pager help abandon`; `jj --no-pager help log`; disposable-repo exact revset syntax probe
  under `/tmp/jk-exact-change.*`; `just md-check`
- Remaining risk: empty-preview abandon now rechecks immediately before execution and falls back to
  typed exact-revision confirmation if the target becomes non-empty, but the final recheck and
  `jj abandon` remain separate `jj` invocations rather than one atomic transaction
- Next slice: Packet 16: Operation Show/Diff Detail

## Packet 16: Operation Show/Diff Detail

- Files changed: `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/main.rs`,
  `src/operation_detail.rs`, `src/operation_log.rs`, `src/tui.rs`, `src/view_state.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `cargo check`; focused `cargo test operation_log`, `cargo test operation_detail`,
  `cargo test operation_show_command_uses_positional_operation_id`,
  `cargo test operation_diff_command_uses_operation_option`,
  `cargo test back_from_operation_detail_returns_to_operation_log`, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and `just md-check`
- Validation note: an early combined command-construction test invocation used multiple cargo test
  filters and failed with `unexpected argument`; the listed one-filter command-construction tests
  were run separately and passed.
- 5.5 review agent `019e4435-f6ce-7a42-94bb-ec62704e8940` (gpt-5) reported no code findings.
- Validation note: `just check` was attempted after Packet 16 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused operation tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Remaining risk: detail views intentionally do not parse operation transaction semantics, so copy
  and search operate on rendered text only; command exactness depends on the documented
  `jj operation show <operation-id>` and `jj operation diff --operation <operation-id>` shapes. A
  final app-level stack assertion for
  `operation log -> show -> diff -> back -> show -> back -> operation log` is still not covered;
  behavior currently mirrors pushed-detail transition semantics and is covered by a view-level
  show/diff switch test plus app-level back-from-detail coverage.
- Next slice: Packet 17: Undo/Redo From Operation Log

## Packet 17: Undo/Redo From Operation Log

- Files changed: `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/operation_log.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Behavior: operation-log `u` opens a scrollable preview for global `jj undo`, and `C-r` opens the
  same flow for global `jj redo`. The preview, generated help, result output, and tests all state
  that these actions operate on the current repo's undo/redo cursor and do not use the selected
  operation-log row as an argument.
- Final 5.5 review follow-up: fixed remaining Packet 17 issues by adding concise `u`/`C-r` recovery
  hints to the operation-log status bar and updating stale operation-log docs so recovery is global
  and repo-cursor based.
- Verification: `cargo check`; focused `cargo test operation_log`,
  `cargo test operation_undo_command_has_no_operation_id_argument`,
  `cargo test operation_redo_command_has_no_operation_id_argument`, `cargo test operation_recovery`,
  `cargo test operation_redo_failure_keeps_command_output_readable`,
  `cargo test operation_help_exposes_global_undo_and_redo_recovery_actions`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-check`
- Manual proof: disposable repo `/tmp/jk-packet17-proof.cPqScq` was initialized with
  `jj --no-pager git init`. From that repo's cwd, a `describe` mutation set the working-copy
  description to `packet 17 proof mutation`, `jj --no-pager undo` restored the previous
  no-description state, and `jj --no-pager redo` restored `packet 17 proof mutation`. The command
  shapes used for recovery were exactly `jj --no-pager undo` and `jj --no-pager redo`.
- Help proof: `jj --no-pager help undo` shows `Usage: jj undo [OPTIONS]` and describes undo as
  restoring older operations when repeated; `jj --no-pager help redo` shows
  `Usage: jj redo [OPTIONS]` and describes redo as the counterpart after one or more undos.
- Validation note: the first formatter check was started in parallel with the formatter run, so it
  reported the diff that the formatter was applying. A sequential
  `rustup run nightly cargo fmt --check` passed afterward.
- Validation note: `just check` was attempted after Packet 17 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused operation recovery tests, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and `just md-check`.
- Remaining risk: the flow intentionally delegates all transaction selection to `jj undo` and
  `jj redo`, so it does not preview which concrete operation will be undone or redone beyond showing
  the raw jj result afterward. Redo availability is not precomputed; unavailable redo is attempted
  and shown as readable jj error output.
- Next slice: Packet 18: `jj new` From Selected Parents

## Packet 18: `jj new` From Selected Parents

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: graph action menus now offer a preview-first `new` action. With no explicit
  multi-selection, it previews and runs `jj new <current-change-id>`. With explicit graph
  selections, it previews and runs `jj new <parent-1> <parent-2> ...` using current graph row order.
  Preview/result output uses the scrollable ActionOutput overlay, lists every exact parent id, and
  keeps `jj undo` visible after success.
- Verification: `cargo check`; focused `cargo test new_plan`, `cargo test open_action_menu`,
  `cargo test new_`, and `cargo test action_menu`; full `cargo test`; `jj --no-pager help new`;
  disposable-repo single-parent and merge-parent `jj new` proof under
  `/tmp/jk-packet18-proof.gGQtDR`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `just md-check`
- Validation note: `just check` was attempted after Packet 18 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused new/action-menu tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repo `/tmp/jk-packet18-proof.gGQtDR` was initialized with
  `jj --no-pager git init`. From that repo's cwd, the single-parent proof created working copy
  `squuswtskrqpwnpurzsxrzmkxkvnwmmo` with parent `zuupqvnuymlryzzwxxxmvkuwymopmsyy`, then
  `jj --no-pager undo` restored the base working copy. From the same repo's cwd, the merge-parent
  proof created working copy `wtwnpzzqkwnwultqoupwrkotxrkywmxn` with exact parents
  `vnswyskrxrwtskxyzrptylwntzklqrmr` and `qzzyspyxnskmwxpprqzvposmxrypnqtm`, then
  `jj --no-pager undo` restored the prior right-parent working copy.
- Remaining risk: parent identity is exact only because graph rows carry template-derived full
  change ids; the flow intentionally does not wrap these positional `jj new` parent arguments in a
  stronger revset because Packet 18 requested the exact `jj new <change-id>` shape. Explicit
  multi-select ordering is now graph-row order rather than toggle order, which is tested but still
  depends on the rendered graph and metadata streams staying paired.
- Next slice: Packet 19: Push Flow Simplification

## Packet 19: Push Flow Simplification

- Files changed: `src/app.rs`, `src/jj.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: push now skips remote selection when `jj git remote list` reports exactly one remote and
  opens the existing scrollable push preview directly for that remote. Multiple remotes still open
  the push-specific remote picker, and no-remotes or unsupported-view paths remain disabled with
  readable status errors. Preview/result context now names the target semantics explicitly: status
  pushes use jj default target resolution for the selected remote, bookmark pushes target the exact
  selected bookmark name, and graph pushes target the exact selected revision.
- Verification: `cargo check`; focused `cargo test push`; full `cargo test`; disposable remote-less
  jj proof under `/tmp/jk-packet19-proof.NfYfy6`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `just md-check`
- Manual proof: disposable repo `/tmp/jk-packet19-proof.NfYfy6` was initialized with
  `jj --no-pager git init`. From that repo's cwd, `jj --no-pager git remote list` returned no
  remotes, and `jj --no-pager git push --dry-run` reported
  `Warning: No bookmarks/tags found in the default push revset: remote_bookmarks(remote=origin)..@`
  followed by `Nothing changed.`
- Validation note: `just check` was attempted after Packet 19 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused push tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Remaining risk: status-context push still delegates target choice to jj's default push resolution
  for the selected remote; `jk` makes that delegation visible but does not precompute the exact
  bookmark or revision jj will select. Preview and result bodies preserve successful raw jj CLI
  output and may be followed by a local `refresh failed: ...` line if the post-push refresh step
  fails.
- Next slice: Packet 20 README/User Docs Refresh

## Packet 20: README/User Docs Refresh

- Files changed: `README.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `just md-check`; manual read-through against the current command inventory, shipped
  packet history, and README claims
- Remaining risk: the README now summarizes the current shipped surface instead of enumerating every
  binding, so it will need another refresh when shipped keys or startup views expand, and Packet 21
  will need to add capture specs before the media policy section can point at concrete artifacts
- Next slice: Packet 21: VHS Specs Without Committed GIFs

## Packet 21: VHS Specs Without Committed GIFs

- Files changed: `README.md`, `Justfile`, `docs/demo/README.md`,
  `docs/demo/operation-recovery.tape`, `docs/demo/setup-demo-repo.sh`, `docs/demo/static-log.tape`,
  `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `just demo-setup`; `vhs validate docs/demo/*.tape`; `just demo-static-log`;
  `just demo-operation-recovery`; `just md-check`
- Remaining risk: the captures now have tracked specs and deterministic repo setup, but the rendered
  media still depends on current jj output shape, terminal rendering, and VHS/ffmpeg behavior when
  the tapes are rerun, so the output still needs external publication review before it becomes a
  user-facing artifact
- Next slice: Packet 22: Squash Preview Flow

## Packet 22: Squash Preview Flow

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: graph action menus now expose source/destination wording for multi-revision rewrite
  actions, and the existing role prompt can open a scrollable `jj squash` preview. The preview lists
  every exact source revision, the exact destination, the exact command, graph effect,
  noninteractive destination-message behavior, confirmation instruction, and `jj undo` recovery.
  Confirmation runs one multi-source `jj squash` command, refreshes the current view, and prefers
  revealing the destination afterward.
- Command shape: one `jj squash` invocation with repeated `--from` arguments, an explicit `--into`
  destination, and `--use-destination-message`. The destination-message flag is required so source
  descriptions are discarded instead of opening an editor or prompt for a combined description.
- Verification: `cargo check`; focused `cargo test squash`; focused `cargo test action_menu`; full
  `cargo test`; `jj --no-pager squash --help`; disposable-repo proof under
  `/tmp/jk-squash-proof.oAjsZe` for multi-source squash and `jj --no-pager undo`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-check`
- Validation note: `just check` was attempted after Packet 22 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused squash/action-menu tests, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and `just md-check`.
- Validation note: two early focused-test invocations accidentally passed multiple Cargo test-name
  filters and failed with `unexpected argument`; the affected filters were then run separately or
  covered by `cargo test squash`, `cargo test action_menu`, and full `cargo test`.
- Remaining risk: the flow intentionally relies on jj CLI squash semantics for multi-source `--from`
  handling, emptied-source abandonment, descendant rebasing, and destination-message behavior. It
  does not simulate a before/after graph or detect whether the destination remains visible until
  after the command refreshes.
- Next slice: Packet 23: Describe And Commit Flows

## Packet 23: Describe And Commit Flows

- Files changed: `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: `D` opens a description prompt from graph rows with exact change ids and from status as
  `@`. Non-empty input opens a scrollable preview that shows the target, message, noninteractive
  command shape, and `jj undo`; empty input and cancel return to normal mode without running jj.
  Graph targets execute through an exact `change_id()` revset, while status uses the visible `@`
  target.
- Final 5.5 review outcome: no blocking findings; the status describe path was tightened with an
  app-level test for a status `D` prompt that targets `@` and opens the expected
  `jj describe @ --message <message>` preview.
- Behavior: `C` opens a commit prompt from graph or status, but the preview and generated help state
  that `jj commit` always targets `@` and ignores graph selection. Confirmation runs
  `jj commit --message <message>`, refreshes afterward, keeps the new-working-copy-on-top effect
  visible, and preserves success or failure output in `ActionOutput`.
- Command shapes: describe uses `jj describe <target> --message <message>`, with graph targets
  represented as `exactly(change_id("<id>"), 1)` and status targets represented as `@`. Commit uses
  `jj commit --message <message>` with no revision argument.
- Verification: `cargo check`; focused `cargo test describe --no-fail-fast`;
  `cargo test commit --no-fail-fast`; full `cargo test`; `jj --no-pager describe --help`;
  `jj --no-pager commit --help`; disposable-repo describe/commit/undo proof under
  `/tmp/jk-packet23-proof.UW66K1`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `just md-check`
- Validation note: an early focused-test invocation accidentally passed multiple Cargo test-name
  filters and failed with `unexpected argument`; the affected describe and commit filters were then
  run separately and passed.
- Validation note: `just md-check` initially found Panache formatting diffs in the touched docs;
  `just md-fmt` reformatted them and the rerun passed.
- Validation note: `just check` was attempted after Packet 23 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused describe/commit tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Remaining risk: graph describe targets are exact because they come from template-derived graph row
  change ids, but status describe and commit intentionally delegate to jj's dynamic `@` at execution
  time. Commit from graph is deliberately selection-independent and may still surprise users who do
  not read the preview; help and preview text call out that selected graph rows are not arguments.
- Next slice: Packet 24: Bookmark Mutation Flows

## Packet 24: Bookmark Mutation Flows

- Files changed: `src/app.rs`, `src/bookmarks.rs`, `src/command.rs`, `src/file_list.rs`,
  `src/jj.rs`, `src/tui.rs`, `src/view_state.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: graph and status views now expose local bookmark create (`b`), set (`=`), and move (`m`)
  flows. Each flow prompts for one bookmark name, targets the selected exact graph change id or
  visible status `@`, opens a scrollable `ActionOutput` preview, and requires Enter confirmation
  before running the `jj bookmark` command.
- Behavior: the bookmarks view now exposes local bookmark delete (`x`) for the selected exact local
  bookmark row. The preview uses an exact bookmark string pattern, says this is delete rather than
  forget, keeps `jj undo` visible, and requires Enter confirmation through `ActionOutput`.
- Review repair: reviewer `019e44b3-9a26-7402-a577-5247e84ecda2` found that remote rows exposed by
  args such as `--all-remotes` could drift against the local metadata stream and be treated as
  deletable, and that file-list hints advertised `x delete` even though global dispatch routed `x`
  to bookmark delete first. The repair pairs one metadata row to each rendered bookmark row, uses
  the machine `remote` template field to prove local rows, treats missing metadata as nonlocal,
  scopes `x` to the bookmarks view, and removes the file-list delete hint.
- Final repaired 5.5 review `019e44be-0503-7671-93cb-108959581966` (`gpt-5.5`, high) reported no
  findings and accepted Packet 24 repairs.
- Command shapes: create and set use `jj bookmark create|set --revision <target> <name>`, move uses
  `jj bookmark move --to <target> exact:<quoted-name>`, and delete uses
  `jj bookmark delete exact:<quoted-name>`. Graph targets are represented as
  `exactly(change_id("<full-change-id>"), 1)`, while status targets remain `@`.
- Deferred behavior: track and untrack remain unexposed because `BookmarkItem` still does not carry
  exact remote or tracking metadata. Rendered labels such as `@origin` or `main@origin` are not used
  to infer tracking state.
- Verification: `cargo check`; focused `cargo test bookmark`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; disposable-repo proof
  under `/tmp/jk-packet24-proof.ZCshiQ` for create, set, move, delete, undo, and duplicate-name
  failure preservation; `just md-check`
- Review repair validation: `cargo test remote_bookmark_rows_do_not_advance_local_metadata`;
  `cargo test file_list_x_is_not_bookmark_delete`;
  `cargo test file_list_status_hints_do_not_advertise_delete`; `cargo test bookmark`; full
  `cargo test`; `cargo check`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt   --check`; `just md-check`
- Manual proof: disposable repo `/tmp/jk-packet24-proof.ZCshiQ` was initialized with
  `jj --no-pager git init`. From that repo's cwd, create and set used
  `jj --no-pager bookmark create|set --revision 'exactly(change_id("<id>"), 1)' <name>`, move used
  `jj --no-pager bookmark move --to 'exactly(change_id("<id>"), 1)' 'exact:"packet24-move"'`, delete
  used `jj --no-pager bookmark delete 'exact:"packet24-delete"'`, and `jj --no-pager undo` restored
  the deleted bookmark.
- Manual proof: the duplicate-name failure path was checked from the same repo cwd with
  `jj --no-pager bookmark create --revision <exact-base-revset> packet24-create`; jj returned
  `Bookmark already exists: packet24-create`, and the bookmark row was unchanged before and after.
- Validation note: `just md-check` initially found Panache formatting diffs in
  `docs/plan/progress.md` and `docs/plan/fragility-register.md`; `just md-fmt` reformatted those
  files and the rerun passed.
- Validation note: `just check` was attempted after Packet 24 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused bookmark tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Remaining risk: create/set/move from status intentionally target jj's dynamic `@` at execution
  time. Bookmark list rows still depend on row-order pairing between rendered output and a
  machine-template metadata stream; delete is disabled whenever that stream does not prove an empty
  remote field, and remote/tracking flows remain deferred until explicit metadata is modeled beyond
  local delete gating.
- Next slice: Packet 25: Absorb Preview Flow

## Packet 25: Absorb Preview Flow

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/graph.rs`, `src/jj.rs`, `src/tui.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: graph action menus now expose a bounded preview-first `absorb` action only when the
  current graph row has an exact change id and at least one selected exact graph row remains after
  excluding the current row. The current row is the single source revision. Explicitly selected rows
  are candidate destinations, and the preview states that jj only considers selected revisions that
  are ancestors of the source.
- Command shape: one `jj absorb` invocation with a single exact `--from` revset,
  `exactly(change_id("<source>"), 1)`, and repeated exact `--into` revsets,
  `exactly(change_id("<candidate>"), 1)`. The flow does not expose bare `jj absorb`,
  status/current-`@` absorb, implicit `mutable()`, filesets, patch selection, multi-source absorb,
  `--ignore-immutable`, or `--no-integrate-operation`.
- Preview/result behavior: the preview lists the exact source id, candidate destination ids, exact
  command, line-level placement semantics, ambiguity behavior, source emptying/abandonment caveat,
  and the `jj undo` and `jj op show -p` review paths. Confirmation refreshes the current view and
  keeps `jj undo | jj op show -p` visible in the completed scrollable result output.
- Verification: `cargo check`; focused `cargo test absorb`; focused `cargo test action_menu`;
  focused `cargo test app::tests::absorb -- --test-threads=1`; focused
  `cargo test jj::tests::absorb -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-check`
- Validation note: `just check` was attempted after Packet 25 validation but failed immediately at
  `cargo +nightly fmt` with `no such command: +nightly`. Equivalent checks were run separately:
  `cargo check`, focused absorb/action-menu tests, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repo `/tmp/jk-absorb-proof.ADHs9w` was initialized with
  `jj --no-pager git init`. From that repo's cwd, a base line was tracked, change A edited the line,
  and change B edited the same line. `jj --no-pager absorb --from @ --into @-` absorbed the source
  changes into one revision, rebased the descendant, and left the source working copy empty.
  `jj --no-pager op show -p --color never` showed the changed commits and rendered patch, and
  `jj --no-pager undo` restored the pre-absorb graph.
- Final 5.5 review `019e44cf-4ec5-7bf2-a20d-0a8f83315480` (`gpt-5.5`, high) reported no findings and
  accepted Packet 25.
- Remaining risk: `jk` does not simulate line-level placement, candidate ancestry filtering, source
  emptying, source abandonment, or final graph shape. Those remain jj semantics visible through the
  preview text, result output, `jj undo`, and rendered `jj op show -p` review path.
- Next slice: Packet 26: Rebase Polish And Before/After Graph

## Packet 26: Rebase Polish And Before/After Graph

- Files changed: `src/app.rs`, `src/jj.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: the rebase preview is now explicitly a command summary, not a simulated graph preview.
  It lists the exact `jj rebase -r <source> ... -o <destination>` command, recaps source and
  destination roles, names the current graph context, and states the expected `--revision`/`--onto`
  semantics without running `jj` or reconstructing the final graph.
- Command shape: runtime still uses `jj rebase -r <source> [-r <source>...] -o <destination>`. The
  flow does not use `--no-integrate-operation`, `--source`, `--branch`, `--insert-after`,
  `--insert-before`, filesets, or alternate rebase variants.
- Preview/result behavior: the preview states that only listed `-r` sources are rebased,
  dependencies among listed sources are preserved, descendants outside the selected set may be
  rebased to fill holes, and destination descendants are not inserted or rebased by `-o`. Successful
  rebase results still reveal the primary source after refresh and now keep
  `jj undo | jj op show -p` visible in both the status line and scrollable result output.
- Verification: `cargo check`; focused `cargo test jj::tests::rebase -- --test-threads=1`; focused
  `cargo test app::tests::rebase -- --test-threads=1`; focused
  `cargo test action_menu -- --test-threads=1`; full `cargo test`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `just md-check`
- Validation note: `just check` was attempted after Packet 26 validation but failed immediately at
  the known `cargo +nightly fmt` wrapper step. Equivalent checks were run separately: `cargo check`,
  focused rebase/action-menu tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repo `/tmp/jk-rebase-proof.4HPKSi` was initialized with
  `jj --no-pager git init`. From that repo's cwd, a base change, sibling destination, and sibling
  source were created. `jj --no-pager rebase -r vwvwtwqwtypx -o txkwxxok` moved the source onto the
  destination, `jj --no-pager op show -p --color never` showed the rebase operation patch, and
  `jj --no-pager undo` restored the sibling graph.
- Remaining risk: `jk` still does not know or preview the final graph before execution. It delegates
  graph truth to `jj`, preserves the raw command/result path, and points users to `jj op show -p`
  and `jj undo` after execution.
- Final 5.5 repair note: the preview summary text was flagged as clipping in normal terminal widths
  because the command-effect semantics were one long line. Spark repaired this by splitting the
  rebase effect section into short, readable lines.
- Final 5.5 re-review accepted Packet 26 after the clipping repair, with no remaining findings.
- The re-review ran `cargo test rebase -- --test-threads=1`.
- Main-thread follow-up validation after the repair used a full `cargo test` and `just md-check`.
- Next slice: Packet 27: Restore/Revert Guided Flows

## Packet 27: Restore/Revert Guided Flows

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/command.rs`, `src/diff.rs`,
  `src/file_list.rs`, `src/file_show.rs`, `src/graph.rs`, `src/jj.rs`, `src/show.rs`, `src/tui.rs`,
  `src/view_state.rs`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`
- Behavior: preview-first restore and revert flows now appear only from exact supported contexts.
  Graph action menus add whole-revision `restore` and `revert` for the current exact row. Show and
  diff views expose only revision-scoped restore/revert when the view carries a graph-derived exact
  revision target. File-list and file-show add path-scoped restore ahead of revision-scoped
  restore/revert when they already own both the exact graph-derived revision target and the exact
  selected path.
- Command shape: restore always uses `jj restore --changes-in exactly(change_id("<id>"), 1)` with an
  optional exact `root-file:"<path>"` fileset. Revert always uses
  `jj revert -r exactly(change_id("<id>"), 1) -o @`. The flow does not expose path-scoped revert,
  arbitrary detail-view revsets, parsed sticky headings as mutation-grade paths, status file
  actions, multi-select restore/revert, operation restore/revert, patch selection, or
  `--no-integrate-operation`.
- Preview/result behavior: restore previews show the exact revision, optional exact path and
  `root-file:` fileset, the exact restore command, the forward `jj diff` that restore removes, the
  Enter confirmation, and `jj undo`. Revert previews show the exact revision, the exact revert
  command, the forward `jj diff` that revert reverse-applies into `@`, the Enter confirmation, and
  `jj undo`. Successful confirmation refreshes the active view and keeps the completed result output
  scrollable. Failures preserve the full multiline command output in the same overlay.
- Verification: `cargo check`; focused `cargo test restore -- --test-threads=1`; focused
  `cargo test revert -- --test-threads=1`; focused `cargo test action_menu -- --test-threads=1`;
  full `cargo test`; `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`
- Validation note: `just check` was attempted after Packet 27 validation but failed immediately at
  the known `cargo +nightly fmt` wrapper step. Equivalent checks were run separately: `cargo check`,
  focused restore/revert/action-menu tests, full `cargo test`, `rustup run nightly cargo fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repo `/tmp/jk-packet27-proof.1FRehG` was initialized with
  `jj --no-pager git init`. From that repo's cwd, a base change, a mutable source change touching
  `path with spaces.txt` and `extra.txt`, and a revert-target working copy were created. Running
  `jj --no-pager restore --changes-in 'exactly(change_id("<source>"), 1)' 'root-file:"path with spaces.txt"'`
  removed only the selected path from the source change and rebased the descendant.
  `jj --no-pager undo` restored the original two-file source diff. Running
  `jj --no-pager revert -r 'exactly(change_id("<source>"), 1)' -o @` succeeded, and
  `jj --no-pager op show -p --color never` showed the generated revert change and both reversed file
  hunks. `jj --no-pager undo` restored the pre-revert operation state.
- Remaining risk: path-scoped restore still depends on jj fileset string-literal semantics and on
  `jj restore`'s descendant-rewrite behavior, while preview honesty still depends on users reading
  the forward `jj diff` as command input rather than as a simulated final graph.
- Final 5.5 re-review accepted Packet 27 after the bookmark provenance repair, with only
  non-blocking cleanup items fixed.
- Next slice: Packet 28: Resolve Screen And Conflict Flow

## Packet 28: Resolve Screen And Conflict Flow

- Files changed: `Cargo.toml`, `src/app.rs`, `src/command.rs`, `src/jj.rs`, `src/main.rs`,
  `src/resolve.rs`, `src/tui.rs`, `src/view_state.rs`, `docs/plan/screens/resolve.md`,
  `docs/plan/progress.md`, `docs/plan/fragility-register.md`, and `docs/process-observations.md`
- Behavior: `jk resolve` now opens a focused conflict list screen, and global `R` opens the same
  screen from other views. The list is read-only, path-first, and uses a narrow
  `self.conflicted_files()` template contract instead of rendered `jj resolve --list`. Each row
  shows the conflicted path, `file_type`, and `side_count`. Search, copy, refresh, help, item
  counts, and back behavior follow the existing selectable-list pattern. `Enter` and `l` inspect the
  selected conflict with `jj file show -r <resolve-target-or-@> <path>` when an exact path is known,
  and otherwise report a clear status message.
- Command shape: conflict listing uses
  `jj log --no-graph -r <target-or-@> --color=never -T   'self.conflicted_files()...'` and parses
  one JSON object per conflicted file. The first pass does not run `jj resolve <path>`, launch
  external merge tools, mark conflicts resolved, infer paths from rendered headings, or mutate
  files.
- Refresh/result behavior: clean repos open as `0 conflicts` instead of a failure state. Refresh
  preserves selection by exact path when possible and clamps cleanly when rows disappear. Copy
  offers the exact path when known and always offers displayed row text. Malformed JSON lines
  degrade into readable non-inspectable rows instead of panicking.
- Verification: `cargo check`; focused `cargo test resolve -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`
- Validation note: `just check` was attempted after Packet 28 validation but still failed
  immediately at the known `cargo +nightly fmt` wrapper step. Equivalent checks were run separately:
  `cargo check`, focused resolve tests, full `cargo test`, `rustup run nightly cargo   fmt`,
  `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repos `/tmp/jk-packet28-clean.VYKte2` and `/tmp/jk-packet28-proof.Ice7He`
  were initialized with `jj --no-pager git init`. In the clean repo, running the chosen listing
  command against `@` produced no output and exited successfully. In the conflicted repo, two
  sibling edits to `file.txt` were merged into a conflicted working copy and the same listing
  command produced `{"path":"file.txt","file_type":"conflict","side_count":2}`. A rerun after an
  initial over-escaped shell proof confirmed the newline-sensitive one-object-per-line contract.
- Remaining risk: Packet 28 depends on `jj 0.41` template method names and on `entry.path()`
  continuing to expose an exact path string suitable for read-only inspection. Guided resolve
  actions still need a stronger contract before `jk` can safely launch `jj resolve <path>` or build
  exact conflict-scoped filesets.
- Final 5.5 re-review accepted Packet 28 after the default `@` target normalization, with no
  findings.
- Next slice: Packet 29: Day-To-Day Tutorial Set

## Packet 29: Day-To-Day Tutorial Set

- Files changed: `README.md`, `docs/tutorials/README.md`, `docs/tutorials/daily-loop.md`,
  `docs/tutorials/rewrite-and-recovery.md`, `docs/tutorials/bookmarks-and-conflicts.md`,
  `docs/plan/progress.md`, `docs/process-observations.md`
- Behavior: added a concise tutorial index and three walkthroughs for the shipped daily loop. The
  tutorials cover inspect, show/diff, status, fetch, push, create new work, describe/commit,
  abandon, squash/rebase/absorb, restore/revert, operation recovery, bookmarks, and the read-only
  resolve screen. The examples reuse the tracked demo repos where that keeps the setup concrete and
  say clearly when a flow needs a repo with exact targets or conflicts.
- Final 5.5 repair: tutorial keybinding and scope language was corrected for graph show entry,
  action-menu vs abandon entry, restore/revert scope visibility, and bookmark command scopes.
- Final 5.5 re-review accepted Packet 29 after the final daily-loop wording repair, with no
  findings.
- Validation: `just demo-setup`; `vhs validate docs/demo/*.tape`; `just md-check`
- Validation note: render commands such as `just demo-static-log` and `just demo-operation-recovery`
  were not run so the repo did not generate or commit media.
- Remaining risk: the tutorials intentionally stay concise, so future packets that add bindings or
  broaden exact-target workflows will need a refresh to keep the walkthroughs aligned with shipped
  behavior.
- Next slice: Packet 30: Edit/Next/Prev Navigation Flows

## Packet 30: Edit/Next/Prev Navigation Flows

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/command.rs`, `src/graph.rs`, `src/jj.rs`,
  `src/tui.rs`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`
- Behavior: graph view now offers preview-first working-copy navigation for `edit`, `next`, and
  `prev`. Direct graph bindings use `e` for exact-row `edit`, `]` for `next --edit`, and `[` for
  `prev --edit`. The graph action menu now adds `edit selected revision ...` only when the current
  row itself is an exact single-row graph target. `next` and `prev` stay out of the action menu so
  the UI does not imply they target the highlighted row.
- Command shape: `edit` always runs `jj edit exactly(change_id("<change-id>"), 1)`. `next` always
  runs `jj next --edit`. `prev` always runs `jj prev --edit`. `jk` does not pass graph-row targets
  to `next`/`prev`, does not use bare `jj next`/`jj prev`, and does not parse or choose between
  ambiguity candidates.
- Preview/result behavior: `edit` previews show the exact selected graph revision and say the
  command moves `@` to edit that revision directly. `next`/`prev` previews say the highlighted row
  is not an argument, that movement is relative to current `@`, and that ambiguity stays a normal
  `jj` failure path. Confirmation runs exactly the previewed command. Successful confirmation
  refreshes the graph, reveals the edited/current `@` change with the existing recent-work fallback,
  and keeps `jj undo` visible in status and completed output. Failures preserve full multiline
  command output in `ActionOutput`.
- Final 5.5 repair: Packet 30’s command-boundary miss was accepted and fixed by adding `--no-graph`
  to the `resolve_exact_change_id` command path, so `next --edit` and `prev --edit` refresh/reveal
  cannot be broken by graph-line contamination on `@`.
- Final 5.5 re-review: packet `019e4553-4e86-7e53-adaf-30baaa0651fe` accepted Packet 30 after the
  `--no-graph` repair with no findings.
- Residual validation gap: only `prev` currently exercises the shared next/prev success flow in app
  behavior, while `next` shares the same success branch and should be covered directly at the same
  level.
- Verification: `cargo check`; focused command tests
  `cargo test edit_plan_uses_exact_change_id_revset -- --test-threads=1`,
  `cargo test next_plan_uses_explicit_edit_flag_and_ignores_selection -- --test-threads=1`,
  `cargo test prev_plan_uses_explicit_edit_flag_and_mentions_ambiguity -- --test-threads=1`; focused
  graph/help/app tests
  `cargo test project_help_exposes_graph_edit_next_and_prev_only_in_graph -- --test-threads=1`,
  `cargo test graph_bindings_expose_edit_next_and_prev_keys -- --test-threads=1`,
  `cargo test open_action_menu_prefers_single_row_context -- --test-threads=1`,
  `cargo test edit_action_menu_enter_opens_preview_with_exact_target -- --test-threads=1`,
  `cargo test edit_direct_key_requires_exact_selected_graph_row -- --test-threads=1`,
  `cargo test next_direct_key_opens_preview_without_selected_row_targeting -- --test-threads=1`,
  `cargo test working_copy_navigation_preview_cancel_restores_normal_mode -- --test-threads=1`,
  `cargo test edit_confirm_success_refreshes_and_reveals_target -- --test-threads=1`,
  `cargo test prev_confirm_success_resolves_current_working_copy_and_reveals_recent -- --test-threads=1`,
  `cargo test working_copy_navigation_failure_keeps_output_readable -- --test-threads=1`; full
  `cargo test`; focused
  `cargo test resolve_exact_change_id_command_uses_no_graph_contract   -- --test-threads=1`,
  `cargo test parse_exact_change_id_rejects_graph_like_output -- --test-threads=1`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `cargo check`
- Validation note: `just check` was attempted after Packet 30 validation but still failed
  immediately at the known `cargo +nightly fmt` wrapper step. Equivalent checks were run separately,
  including `cargo check`, the focused Packet 30 tests above, full `cargo test`,
  `rustup run nightly cargo fmt`, `rustup run nightly cargo fmt --check`, and `just md-check`.
- Manual proof: disposable repo `/tmp/jk-packet30-proof.uYVEee` was initialized with
  `jj --no-pager git init`. From that repo's cwd, a base change, `child a`, `child b`, and a sibling
  child were created. `jj --no-pager edit 'exactly(change_id("<base>"), 1)'` moved `@` directly to
  the base change and `jj --no-pager undo` restored the sibling working copy.
  `jj --no-pager edit 'exactly(change_id("<child-a>"), 1)'` followed by `jj --no-pager next --edit`
  moved `@` from `child a` to `child b`, and `jj --no-pager undo` restored `child a`. From
  `child a`, `jj --no-pager prev --edit` moved `@` back to the base change, and `jj --no-pager undo`
  restored `child a`. With `@` edited back to the base change and two editable children available,
  `jj --no-pager next --edit` failed non-interactively with the raw `jj` ambiguity prompt/output and
  `Error: Cannot prompt for input since the output is not   connected to a terminal`; `jk` preserves
  that failure as command output instead of interpreting it.
- Remaining risk: `next --edit` and `prev --edit` still depend on installed `jj` topology semantics
  and can fail with interactive ambiguity prompts when multiple editable successors/predecessors
  exist. `jk` now keeps `--edit` explicit and preserves those failures readably, but it still does
  not preview the final graph or resolve ambiguity on the user's behalf.
- Next slice: Packet 31: Command Coverage Audit And Passthrough Policy

## Packet 31: Command Coverage Audit And Passthrough Policy

- Files changed: `docs/plan/command-inventory.md`, `docs/plan/workflows.md`,
  `docs/plan/workflows/inspect.md`, `docs/plan/workflows/recover.md`,
  `docs/plan/workflows/refs-and-workspaces.md`, `docs/plan/workflows/rewrite.md`,
  `docs/plan/workflows/sync.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Behavior: the command inventory now separates shipped native screens, utility screens, guided
  flows, direct actions, planned follow-ups, passthrough commands, and deferred commands. The
  workflow docs now say which loops are already shipped and which ones still need dedicated homes or
  stronger command-policy decisions.
- Docs/fragility updates: `docs/plan/fragility-register.md` unchanged; the audit introduced no new
  parsed or inferred command contracts.
- Validation: `just md-check`; manual consistency check against `src/command.rs`, `src/app.rs`,
  `src/jj.rs`, `docs/plan/progress.md`, and the shipped tutorial docs
- Validation note: no Rust validation was run because Packet 31 is docs-only.
- 5.5 follow-up repair notes: passthrough wording no longer implies command-mode support;
  `jj git fetch` launch context was corrected to global/direct wording (not limited to status/log),
  bookmark `set/create/move` contexts were corrected to graph-exact-or-status-`@` targets and
  `delete` to local bookmark rows in bookmarks view; and `operation integrate` is now documented as
  passthrough/specialized. A final 5.5 follow-up also reclassified `metaedit`, `parallelize`,
  `simplify-parents`, and `bookmark advance` as passthrough in workflow docs to match
  `command-inventory.md`. A final Packet 31 repair also moved `gerrit` and `util` from passthrough
  to deferred in `docs/plan/workflows.md`.
- Final 5.5 acceptance check found no findings after the fetch wording cleanup and passthrough/
  classification repairs.
- Remaining risk: the audit does not implement any new command home, so the planned entries for
  command families such as `bookmark track/untrack`, `file track/untrack/chmod`,
  `operation restore/revert`, `workspace`, `tag`, and editor-centric passthrough commands still need
  future implementation packets.
- Next slice: Packet 32: Strong Command-Coverage Follow-Through

## Packet 32: Strong Command-Coverage Follow-Through

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`,
  `docs/process-observations.md`
- Behavior: Packet 31's command coverage audit is now translated into bounded follow-through packets
  instead of a broad parity backlog. The plan schedules Packets 33-46: operation restore/revert,
  split, duplicate, bookmark metadata, bookmark rename, bookmark forget, bookmark track/untrack,
  file hygiene actions, workspace/root, a read-only tag list surface, file search, file annotate,
  evolog, and a later low-value parking-lot review as separate packets with explicit owners, write
  sets, non-goals, acceptance criteria, validation, docs/fragility expectations, model routing, and
  review prompts.
- Product boundary: the plan keeps shipped daily flows in maintenance mode and explicitly avoids a
  generic command mode or full `jj` CLI clone. Low-frequency or poor-fit commands remain
  passthrough/deferred unless a later packet proves concrete `jk` value. The 5.5 boundedness repair
  keeps tag work read-only/list-first and defers tag set/delete to a future packet or parking-lot
  review; bookmark rename and bookmark forget now have separate target contracts.
- Validation: `just md-check`; manual consistency check against `docs/plan/command-inventory.md`,
  `docs/plan/workflows.md`, `docs/plan/workflows/inspect.md`, `docs/plan/workflows/recover.md`,
  `docs/plan/workflows/refs-and-workspaces.md`, `docs/plan/workflows/rewrite.md`,
  `docs/plan/workflows/sync.md`, and this progress file.
- Validation note: no Rust validation was run because Packet 32 is docs-only.
- Docs/fragility updates: `docs/plan/fragility-register.md` unchanged because the packet only plans
  future soft contracts; each future packet is required to add or update fragility entries if it
  parses rendered `jj` output, infers semantic state, or duplicates command behavior.
- Remaining risk: the packets are implementation-ready prompts, but command semantics for `split`,
  bookmark tracking and forget, the read-only tag list metadata contract, and workspace support
  still require per-packet exploration before code is written.
- Next slice: Packet 33: Operation Restore/Revert From Operation Log

## Packet 33: Operation Restore/Revert From Operation Log

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/command.rs`, `src/jj.rs`,
  `src/operation_log.rs`, `src/tui.rs`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`,
  `docs/plan/workflows/recover.md`, and `docs/process-observations.md`
- Behavior: operation-log rows with paired exact operation ids now expose a preview-first action
  menu for `jj operation restore <op-id>` and `jj operation revert <op-id>`. Rows without an
  operation id keep the actions disabled with a clear status message. Previews show the exact
  operation id, exact command, effect wording, confirmation requirement, and `jj undo` recovery
  path. Confirmed success and failure stay in scrollable `ActionOutput`; success refreshes the
  operation log and refreshes stacked repo views where practical.
- Product boundary: global `jj undo`/`jj redo` remain separate recovery actions whose preview and
  help text say the selected operation-log row is not an argument.
- Validation: `cargo check`; focused `cargo test operation_ -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-check`;
  disposable `/tmp/jk-packet33-proof.0vmUMi` proof for `jj operation restore <op-id>`, `jj undo`
  recovery, `jj operation revert <op-id>`, `jj undo` recovery, and invalid operation-id failure with
  every mutating proof command run after `cd` into that disposable repo.
- Validation note: `just check` was attempted and still stopped at the known local
  `cargo +nightly fmt` wrapper issue with `no such command: +nightly`; the equivalent direct Rust
  format check, full tests, and Markdown check passed.
- Remaining risk: operation ids still depend on row-order pairing between rendered operation-log
  rows and the separate `self.id()` template stream; restore/revert command semantics are covered by
  installed `jj` behavior and command-construction tests, but no transaction graph simulation is
  attempted.
- Next slice: Pre-Packet-34 Interruption Packet A: App Decomposition And Screen Contracts

## Pre-Packet-34 Planning Interruption: Maintainability And UI Repair Wave

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`,
  `docs/process-observations.md`
- Behavior: Packet 34 Split Guided Flow is postponed behind a maintainability and UI repair
  interruption. The plan now inserts interruption packets after accepted Packet 33 and before Packet
  34, with the first packet routed to `gpt-5.5 high` for decomposing `app.rs` and defining explicit
  screen/action contracts.
- Inserted packets/backlog: the interruption wave covers app decomposition and screen contracts;
  navigation and view entry contracts, including `S` status selection, bookmarks/oplog entry,
  view-menu access, multi-character command grammar, and `h`/`l` expand/collapse behavior; a
  leader-style help menu; keyboard-driven action menus, popovers, selection highlighting, and theme
  coherence; status file selection/actions; fetch remote selection; file viewing and no-wrap modes;
  and validation/commit-message discipline.
- Product boundary: the interruption does not rewrite Packet 33 implementation notes, does not edit
  Rust, and does not remove Packet 34. Split Guided Flow remains planned but waits until the
  maintainability/UI work lands or is explicitly reprioritized.
- Validation: `just md-check`
- Validation note: no Rust validation was run because this is a docs-only planning update.
- Docs/fragility updates: `docs/plan/fragility-register.md` unchanged because this planning update
  introduces no new parser, rendered-output, or command semantic contract; future implementation
  packets must update it when they add such assumptions.
- Remaining risk: the interruption packets are bounded prompts, not completed implementation. Packet
  A must land first so later UI repair packets can avoid defaulting every change back through
  `src/app.rs`.
- Next slice: Pre-Packet-34 Interruption Packet A: App Decomposition And Screen Contracts

## Pre-Packet-34 Interruption Packet A: App Decomposition And Screen Contracts

- Files changed: `src/action_output.rs`, `src/app.rs`, `src/app_screen.rs`, `src/app_status.rs`,
  `src/main.rs`, `src/tui.rs`, `docs/agent/architecture.md`,
  `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`
- Behavior: app dispatch remains the orchestration owner, but modal/screen state and overlay/status
  projection now live in `app_screen.rs`; status-line construction and per-view count wording now
  live in `app_status.rs`; action preview/result scroll-key handling now lives in
  `action_output.rs`. No new user-visible commands, key remapping, parser behavior, or `jj` command
  semantics were introduced.
- Architecture contract: `docs/agent/architecture.md` now names owners for keys, screen state,
  overlay projection, status projection, command execution, and view behavior so later interruption
  packets can route work to a narrower owner instead of defaulting to `src/app.rs`.
- Verification: focused `cargo test app_`; focused `cargo test action_output`; full `cargo test`;
  `cargo check`; `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`;
  `just md-fmt`; `just md-check`.
- Validation note: `cargo check` still reports pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  `cargo clippy -- -D warnings` remains blocked by those dead-code warnings plus pre-existing
  collapsible-if warnings in `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`.
  `just check` was attempted and still stopped at the known local `cargo +nightly fmt` wrapper issue
  with `no such command: +nightly`; equivalent direct Rust formatting, full tests, cargo check, and
  Markdown checks were run. Interactive `cargo run` smoke was not run because the TUI blocks for
  input and the current warning state already prevents a no-warning smoke.
- Docs/fragility updates: `docs/plan/fragility-register.md` unchanged because the extraction did not
  change parser, rendered-output, or command semantic assumptions.
- Remaining risk: this is the first coherent Packet A extraction, not full app decomposition.
  Command-runner injection, startup parsing, and the large app test module remain candidates for
  later Packet A follow-up once this screen/status/action-output ownership boundary is reviewed.
  Other large modules may carry similar concept-mixing pressure and should be audited separately
  after the current extraction is accepted.
- Next slice: accept or repair the current Packet A extraction, then run the Packet A follow-up
  module-coherence audit before starting broad refactors in other large files.

## Packet A Follow-Up Planning: Module Coherence Audit

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`
- Behavior: planning docs now record a bounded follow-up audit for large or concept-mixed modules
  after the current `app.rs` extraction is reviewed. The audit starts with `src/jj.rs` as the most
  obvious candidate by current size, then inspects `src/tui.rs`, `src/graph.rs`, `src/command.rs`,
  `src/action_menu.rs`, and `src/view_state.rs` as secondary candidates.
- Product boundary: this does not claim implementation work, require immediate broad refactors, or
  block acceptance of the first Packet A extraction. The audit must identify owning concepts, split
  candidates, reasons not to split, non-goals, acceptance criteria, validation, and subagent-ready
  follow-up packets before any Rust code is changed.
- Validation: `just md-check`
- Validation note: no Rust validation was run because this is a docs-only planning update.
- Docs/fragility updates: `docs/plan/fragility-register.md` unchanged because this update introduces
  no parser, rendered-output, or command semantic assumptions.
- Remaining risk: the candidate list is size-informed, not proof that a split is needed. The audit
  must distinguish files that are merely large from files where mixed ownership increases cognitive
  load or blocks future packets.
- Next slice: run the module-coherence audit with `gpt-5.5 high` design/review after current Packet
  A acceptance, or continue with the next interruption packet if the audit finds no promotable
  split.

## Packet A Follow-Up: Module Coherence Audit

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`
- Behavior: audited the largest and most likely concept-mixed modules after the Packet A `app.rs`
  extraction: `src/jj.rs`, `src/tui.rs`, `src/graph.rs`, `src/command.rs`, `src/action_menu.rs`, and
  `src/view_state.rs`, with `src/rendered_jj.rs` checked as context for row/rendered-output
  ownership. The audit promotes `src/jj.rs` as the only immediate high-value split candidate and
  records two bounded follow-up packets: Packet A1 for `jj` action-plan extraction and Packet A2 for
  rendered row loading/parser extraction after A1 lands.
- Findings: `src/jj.rs` mixes command-plan contracts, `ViewSpec` and view-mode command shape, direct
  process execution, selectable rendered row models, metadata pairing, row grouping, and parsers in
  one file, so future command packets and parser packets force reviewers through unrelated concepts.
  `src/tui.rs`, `src/graph.rs`, `src/command.rs`, `src/action_menu.rs`, and `src/view_state.rs` are
  large or repetitive in places but still coherent enough to leave intact until a concrete
  UI/navigation/action packet changes their owning concepts.
- Product boundary: this was a design/audit step only. No Rust files were edited, no behavior was
  changed, Packet 34 remains postponed, and no fragility-register entry was added because the audit
  did not discover a new undocumented parser, rendered-output, or command semantic assumption.
- Validation: `just md-check`
- Validation note: no Rust validation was run because this is a docs-only audit.
- Docs/fragility updates: `docs/plan/next-implementation-slices.md` now contains the audit findings
  and subagent-ready Packet A1/A2 prompts. `docs/plan/fragility-register.md` remains unchanged.
- Remaining risk: Packet A1 is behavior-preserving but will still be a broad Rust move across many
  command-plan tests. Packet A2 should wait until A1 lands so parser and row-loading ownership can
  be reviewed without simultaneous command-plan churn.
- Next slice: Interruption Packet A1: Extract `jj` Action Plans, unless Packet A review requests
  repair of the existing app screen/status/action-output extraction first.

## Interruption Packet A1: Extract `jj` Action Plans

- Files changed: `src/jj_actions.rs`, `src/jj.rs`, `src/main.rs`, `docs/agent/architecture.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: preview-first action and mutation plans moved from `src/jj.rs` into `src/jj_actions.rs`.
  The new owner holds operation recovery/target actions, git push, new, describe, commit,
  edit/next/prev, restore, revert, bookmark mutations, rebase, squash, absorb, abandon, exact
  change-id revset quoting, exact bookmark patterns, root-file fileset quoting, and abandon preview
  text. `src/jj.rs` keeps `ViewSpec`, rendered row models, metadata loaders, parsers, direct process
  helpers, and compatibility re-exports for existing `crate::jj::...` imports.
- Product boundary: this is a behavior-preserving extraction. Command argv, command labels, preview
  summaries, fallback messages, exact quoting helpers, and abandon preview behavior were moved with
  their tests rather than redesigned. `docs/plan/fragility-register.md` is unchanged because no
  parser, rendered-output assumption, or command semantic contract changed.
- Verification: focused `cargo test jj_actions -- --test-threads=1`; full `cargo test`;
  `cargo check`; `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; attempted
  `cargo clippy -- -D warnings`; `just md-fmt`; `just md-check`; attempted `just check`.
- Validation note: `cargo check` still reports the pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text`. Clippy remains blocked
  by those warnings plus pre-existing `collapsible_if` findings in `src/bookmarks.rs`,
  `src/graph.rs`, and `src/operation_log.rs`. `just check` remains blocked by the known local
  wrapper issue: `cargo +nightly fmt` exits with `no such command: +nightly`; the direct equivalent
  `rustup run nightly cargo fmt --check`, full tests, cargo check, and Markdown checks passed.
- Remaining risk: `src/jj.rs` still owns rendered row loading, metadata pairing, and parser tests
  until Packet A2. The compatibility re-exports keep call-site churn low, but future work should
  import directly from `jj_actions.rs` once the module boundary is established.
- Next slice: Interruption Packet A2: Extract `jj` Rendered Row Loading, after Packet A1 review
  confirms command-plan behavior was preserved.

## Interruption Packet A2: Extract `jj` Rendered Row Loading

- Files changed: `src/jj_rows.rs`, `src/jj.rs`, `src/main.rs`, `docs/agent/architecture.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`
- Behavior: selectable rendered row models, row loaders, metadata-template pairing, row grouping,
  resolve JSON parsing, file-list path parsing, and row parser tests moved from `src/jj.rs` to
  `src/jj_rows.rs`. `src/jj.rs` keeps `ViewSpec`, command identity, navigation target provenance,
  diff-format arguments, direct process helpers, and compatibility re-exports for existing imports.
- Product boundary: this is a behavior-preserving extraction. Row grouping, bookmark metadata
  pairing, operation-id pairing, resolve JSON degradation, file-list path preservation, and
  ANSI-to-Ratatui conversion behavior were moved with their tests rather than redesigned.
  `docs/plan/fragility-register.md` remains unchanged because no parser assumption or
  rendered-output contract changed.
- Verification: focused `cargo test jj_rows`; focused `cargo test jj::tests`; full `cargo test`;
  `cargo check`; `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; attempted
  `cargo clippy -- -D warnings`; `just md-check`.
- Validation note: `cargo check` still reports pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text` after the move. Clippy
  remains blocked by those warnings plus pre-existing `collapsible_if` findings in
  `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`. Direct nightly formatting reports
  the existing local rustfmt configuration warnings about unstable options but exits successfully.
- Remaining risk: row APIs are still compatibility re-exported through `src/jj.rs`, so call sites
  can continue compiling without a broad import rewrite. Future cleanup can switch imports to
  `jj_rows.rs` directly if the module boundary remains accepted.
- Final 5.5 acceptance found no findings for Packet A2.
- Next slice: Interruption Packet B: Navigation And View Entry Contracts.

## Interruption Packet B: Navigation And View Entry Contracts

- Files changed: `src/app.rs`, `src/app_screen.rs`, `src/bookmarks.rs`, `src/command.rs`,
  `src/diff.rs`, `src/operation_log.rs`, `src/show.rs`, `src/status.rs`, `src/tui.rs`, `README.md`,
  `docs/plan/command-inventory.md`, `docs/plan/screens/bookmarks.md`, `docs/plan/screens/diff.md`,
  `docs/plan/screens/files.md`, `docs/plan/screens/help-keymap.md`,
  `docs/plan/screens/operation-log.md`, `docs/plan/screens/resolve.md`, `docs/plan/screens/show.md`,
  `docs/plan/screens/status.md`, `docs/plan/screens/tags.md`, `docs/plan/screens/workspaces.md`,
  `docs/tutorials/bookmarks-and-conflicts.md`, `docs/tutorials/daily-loop.md`,
  `docs/tutorials/rewrite-and-recovery.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Behavior: direct `S`, `B`, and `O` view-entry bindings remain the existing dispatch path and now
  have screen-level regression coverage. `v` now opens a real view menu for shipped top-level views
  (`log`, default `jj`, status, resolve, bookmarks, and operation log) plus the existing diff-format
  choices. Bookmarks and operation log now treat `l`/Right like Enter for selected-row detail, and
  show, diff, and status treat Right like `l` for file-list expansion. `h`/Left continue to pop the
  view stack.
- Command grammar: `command.rs` now supports single-key and multi-key static bindings through the
  same metadata used by generated help. `bc` dispatches bookmark create. `gf` dispatches fetch from
  graph only, so non-graph `g` bindings keep immediate top navigation. Bare `b` and graph `g` remain
  timed fallbacks while those prefixes are ambiguous.
- Test coverage: command metadata tests cover exact sequence completion and exact fallback from a
  prefix. App-level tests cover direct `S`/`B`/`O`, generated help rows for `b, bc`, graph-only
  `gf`, and `v` view menu, view-menu selection, prefix completion, key-arrival timeout fallback,
  idle timeout status refresh, Esc cancellation, non-graph immediate `g`, and `l`/Right detail
  expansion with `h`/Left back-out.
- Documentation: README, tutorials, command inventory, and current screen notes now describe shipped
  view-menu, `bc`/`gf`, Right expansion, and `h`/Left back behavior. Stale `Esc`-as-back and status
  `Enter` file-open claims were removed from the touched docs.
- Verification: `cargo check`; focused command and app tests; focused single-test checks for
  generated help and view-menu selection; full `cargo test`; nightly Rust formatting and formatting
  check; `just md-fmt`; `just md-check`; attempted `cargo clippy -- -D warnings`; attempted
  `just check`.
- Validation note: `cargo check` still reports the pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text`. Clippy remains blocked
  by those warnings plus the pre-existing `collapsible_if` findings in `src/bookmarks.rs`,
  `src/graph.rs`, and `src/operation_log.rs`. `just check` remains blocked by the known local
  wrapper issue: `cargo +nightly fmt` exits with `no such command: +nightly`; the direct nightly
  format check, full tests, cargo check, and Markdown checks passed. Direct nightly formatting still
  prints the existing local rustfmt configuration warnings about unstable options but exits
  successfully.
- Review repair: fixed timeout expiry when the next key arrives after the deadline, fixed idle
  timeout status refresh, narrowed `gf` from global to graph-only, changed generated help metadata
  for `v` from `view format` to `view menu`, and made diff-format view-menu labels/status explicitly
  name their show/diff scope.
- Review repair validation: focused app, command, and view-menu option tests; `cargo check`; full
  `cargo test`; nightly Rust formatting and formatting check; `just md-fmt`; `just md-check`;
  attempted `cargo clippy -- -D warnings`; attempted `just check`.
- Final 5.5 acceptance evidence: timeout now checks `PendingCommand::deadline` before consuming the
  pending key, idle timeout fallback uses the same status-refresh helper as key-arrival fallback,
  `gf` is graph-local, help now shows `v` as `view menu`, and diff-format labels/status say
  `show/diff format`.
- Final 5.5 verification: `cargo check` passed with existing dead-code warnings; full `cargo test`
  passed with 356 tests; `rustup run nightly cargo fmt --check` passed with existing rustfmt config
  warnings; `just md-check` passed; `cargo clippy -- -D warnings` remains blocked by six known
  issues (three dead-code, three `collapsible_if`).
- Remaining risk: the prefix grammar intentionally introduces a short delay for ambiguous bare `b`
  and graph `g` fallback behavior. This is covered by timeout tests and documented as a transition
  contract, but a future help-leader or command-menu packet should decide whether the bare fallback
  keys stay long-term.
- Next slice: Interruption Packet C: Help Leader Menu, with attention to the new multi-key prefix
  contract and avoiding another broad shortcut redesign.

## Interruption Packet C: Help Leader Menu

- Files changed: `src/command.rs`, `src/app.rs`, `src/tui.rs`, `README.md`,
  `docs/plan/screens/help-keymap.md`, `docs/tutorials/daily-loop.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Behavior: `?` now opens a keyboard-driven command menu instead of a passive key listing. The menu
  renders an explicit `Esc, q, ?` close option, and visible command entries execute through the
  existing `execute_binding` dispatch path before the menu closes.
- Command metadata: generated help remains the source of truth for visible command rows. Help
  sections are grouped by user operation: navigation, view switching, search/copy, repository
  actions, action previews, recovery, and app commands. Commands hidden by the active `HelpContext`
  are also ignored by help-menu dispatch, so a hidden shortcut cannot run from the menu.
- Prefix behavior: help mode reuses the Packet B multi-key grammar for visible help commands. Graph
  `gf` can be typed from the menu, while close keys and hidden commands are handled by the help
  screen contract instead of normal command dispatch.
- Test coverage: command metadata tests cover help-visible binding matching, grouping, hidden
  command filtering, and operation recovery grouping. App-level tests cover execute-and-close,
  close-only, hidden command behavior, multi-key help dispatch, and screen-level section grouping.
  TUI tests cover the rendered command-menu close option and grouped command rows.
- Documentation: README, the help/keymap screen note, and the daily-loop tutorial now refer to the
  generated command menu, execute-and-close behavior, and explicit close keys.
- Verification: `cargo check`; focused `cargo test command::tests`; focused `cargo test help_menu`;
  focused `cargo test tui::tests::help_overlay_text_renders_generated_sections`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-fmt`;
  `just md-check`; attempted `cargo clippy -- -D warnings`.
- Validation note: `cargo check` still reports the pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text`. Clippy remains blocked
  by those warnings plus pre-existing `collapsible_if` findings in `src/bookmarks.rs`,
  `src/graph.rs`, and `src/operation_log.rs`.
- Remaining risk: help-menu dispatch still uses the same short timeout for ambiguous multi-key
  fallbacks as normal mode. That keeps Packet B behavior coherent, but a future key-design slice may
  want a visibly selected menu row or a non-timeout prefix state.
- Next slice: Interruption Packet D: Action Menu, Popovers, And Selection Presentation.

### Packet C Review Repair

- Files changed: `src/app.rs`, `docs/plan/progress.md`, and `docs/process-observations.md`.
- Review findings: expired help prefixes routed the next key through normal dispatch even when the
  fallback opened a prompt, and nonmatching help-prefix suffixes reported an unknown prefix instead
  of running fallback semantics.
- Repair: help pending-prefix handling now runs the pending fallback through the same Help close and
  dispatch path as exact help commands, then routes the suffix through the existing mode-aware
  `handle_key_after_prefix_fallback` helper. This mirrors normal Packet B fallback behavior without
  adding a separate Help-only dispatch path.
- Test coverage: added app-level regressions for `?`, `b`, expired deadline, `x` opening the
  bookmark prompt with input `x`, and `?`, `g`, `j` running graph `g` fallback before routing `j` to
  move down. Existing exact help `gf` coverage still passes.
- Verification: `cargo test help_prefix -- --test-threads=1`;
  `cargo test help_menu -- --test-threads=1`; `cargo test prefix -- --test-threads=1`;
  `cargo test command::tests -- --test-threads=1`; `cargo check`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-fmt`;
  `just md-check`; attempted `cargo clippy -- -D warnings`.
- Validation note: `cargo check` still reports the pre-existing dead-code warnings for
  `FileShowView::new`, `ViewSpec::bookmarks`, and `FileListItem::row_text`. Full `cargo test` passed
  with 364 tests. Clippy remains blocked by those warnings plus pre-existing `collapsible_if`
  findings in `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`.
- Final 5.5 acceptance: Packet C accepted as-is after repair with no blocking findings.
- Final review checks confirm:
  - expired pending Help prefixes run pending fallback then route the suffix through
    `handle_key_after_prefix_fallback`;
  - nonmatching suffixes execute the visible fallback and route through the same helper;
  - the helper dispatches to normal or active modal handling based on opened mode; and
  - idle Help-prefix expiry shares the fallback path.
- Validation:
  - `cargo check` passed with existing dead-code warnings.
  - `cargo test help_prefix -- --test-threads=1` passed.
  - full `cargo test` passed with 364 tests.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by known issues.
- Non-blocking follow-up: direct idle Help-prefix timeout coverage for no-suffix path.
- Next slice: Interruption Packet D: Action Menu, Popovers, And Selection Presentation.

## Interruption Packet D: Action Menu, Popovers, And Selection Presentation

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/tui.rs`, `src/theme.rs`, `src/main.rs`,
  `src/graph.rs`, `src/bookmarks.rs`, `src/operation_log.rs`, `src/file_list.rs`, `src/resolve.rs`,
  `docs/plan/progress.md`, and `docs/process-observations.md`.
- Behavior: action-menu items now advertise direct single-key shortcuts and app dispatch can execute
  a visible action by pressing that key without moving the menu cursor first. Existing `j`/`k`,
  arrows, `Enter`, `Esc`, and `q` behavior remains available.
- Presentation: app-owned list selection, popovers, and action-output borders now use the shared
  `theme` helper. The active-row style uses background plus bold/reversed modifiers without setting
  a foreground, so jj-backed selected rows keep rendered ANSI foreground colors while remaining
  visible in low-color or no-color terminals. Graph multi-selected rows use bold without forcing a
  foreground.
- Popovers: action menus show compact key-prefixed rows and a shorter preview-required title so
  narrow terminals keep shortcuts and labels visible. Role prompts render the preview-required hint
  as its own row instead of flattening multiline status text into one item.
- Test coverage: action-menu model tests cover shortcut metadata and path-restore shortcut
  disambiguation. App tests cover direct shortcut execution, close/cancel behavior, selected context
  preservation, and path restore by shortcut. TUI and graph tests cover action-menu snapshots,
  narrow terminal rendering, selected-row fallback styling, jj-backed foreground preservation,
  role-prompt popover rendering, and theme fallback modifiers.
- Verification:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test action_menu -- --test-threads=1` passed.
  - `cargo test theme::tests -- --test-threads=1` passed.
  - `cargo test tui::tests -- --test-threads=1` passed.
  - full `cargo test` passed with 375 tests after the review repair.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

### Packet D Review Repair

- Files changed: `src/theme.rs`, `src/graph.rs`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Review finding: `theme::active_row_style()` set `fg(Color::White)`, and jj-backed list views used
  that style as `List::highlight_style`, which could override rendered jj ANSI foreground colors on
  the current row.
- Repair: active-row fallback styling no longer sets a foreground color. It keeps the background,
  bold, and reversed modifiers for visibility. Graph marked-row styling also no longer sets a
  foreground, so explicit graph selections keep rendered span foregrounds while still gaining a bold
  marker.
- Test coverage: theme tests now assert active and marked row styles have no forced foreground.
  Graph rendering tests prove current-row highlighting preserves a rendered foreground color while
  applying background plus bold/reversed modifiers, and explicit graph selection preserves rendered
  foreground while applying bold.
- Validation:
  - `cargo test theme::tests -- --test-threads=1` passed.
  - `cargo test foreground -- --test-threads=1` passed.
  - `cargo test tui::tests::action_menu_selected_row_has_visible_fallback_style -- --test-threads=1`
    passed.
  - `cargo test tui::tests -- --test-threads=1` passed.
  - `cargo check` passed with existing dead-code warnings.
  - full `cargo test` passed with 375 tests.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.
- Final 5.5 acceptance:
  - Packet D accepted as-is after repair with no blocking findings.
  - Acceptance evidence: `theme::active_row_style()` leaves foreground unset;
    `theme::marked_row_style()` is BOLD-only; graph tests verify rendered current-row foreground
    survives `List` highlighting while adding background and bold/reversed, and explicit graph
    selection preserves rendered foreground while adding bold.
  - Current inference boundary: foreground preservation is render-tested on graph, and inferred for
    other jj-backed lists through the same shared style path.
- Next slice: `Interruption Packet E: Status File Actions`.

## Interruption Packet E: Status File Actions

- Files changed: `src/status.rs`, `src/view_state.rs`, `src/action_menu.rs`, `src/app.rs`,
  `src/jj_actions.rs`, `src/command.rs`, `src/tui.rs`, `docs/plan/progress.md`,
  `docs/process-observations.md`, and `docs/plan/fragility-register.md`.
- Behavior: status is now a row-selectable view. Each rendered status line remains the presentation
  source, while `StatusRow` records whether the line confidently owns one exact repo-relative path.
  `M`, `A`, and `D` rows with clean relative paths enable file actions; headers, conflicts, renamed
  rows, untracked-looking rows, absolute paths, parent-relative paths, and multi-path rows remain
  selectable but report a disabled reason instead of guessing.
- Action routing: status exact paths flow through `ViewState::exact_restore_revert_context()` into
  `ExactActionContext::status_path()`. The status action menu exposes only working-copy path
  restore, which opens the existing preview/result surface with `jj restore root-file:"<path>"`.
  Path-scoped revert remains unavailable because installed `jj revert` has no fileset argument.
- Refresh and output behavior: status refresh preserves selection by exact path when possible,
  otherwise by prior row text, then clamps by index. Restore confirmation continues to preserve raw
  command output in the action-output result screen and keeps `jj undo` visible.
- Test coverage: parser tests cover modified, added, deleted, renamed, conflict, untracked-looking,
  absolute, parent-relative, and multi-path rows. Status view tests cover movement, search, exact
  selected path, disabled header rows, refresh preservation, and clamp. Action-menu, view-state,
  command-construction, and app tests cover status path routing and preview/result behavior.
- Mutation proof: disposable `/tmp` jj repos proved `jj restore root-file:"modified.txt"`,
  `jj restore root-file:"added.txt"`, and `jj restore root-file:"deleted.txt"` remove only the
  selected working-copy file changes, and `jj undo` restores the prior status. A follow-up exact
  argv proof in `/tmp/jk-status-actions-exact3.5c8PvE` used the app-equivalent single argument
  `root-file:"file.txt"` and verified restore plus undo recovery.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test status` passed.
  - full `cargo test` passed with 387 tests.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.
- Remaining risk: the exact-path parser intentionally recognizes only the narrow default single-path
  `jj status` row shape. If jj status output becomes configurable or gains structured output
  suitable for status files, this should move to a stronger contract instead of expanding
  rendered-text inference.
- Next slice: `Interruption Packet F: Fetch Remote Selection`.

## App Decomposition Slice 1

- Files changed: `src/app.rs`, `src/app/tests.rs`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Structure: the inline `#[cfg(test)] mod tests` from `src/app.rs` moved into the child module
  `src/app/tests.rs`. `src/app.rs` now declares `#[cfg(test)] mod tests;`.
- Scope control: production behavior was not refactored, and no visibility changes were needed
  because the child test module can continue to use parent-private app items through `super::*`.
- Size evidence: `src/app.rs` dropped from 7,779 lines to 3,854 lines; `src/app/tests.rs` contains
  the extracted 3,899-line app behavior test module after rustfmt.
- Validation:
  - `cargo test app -- --test-threads=1` passed with 142 tests.
  - `cargo check` passed with existing dead-code warnings.
  - full `cargo test` passed with 387 tests.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.

## App Decomposition Slice 2

- Files changed: `src/app.rs`, `src/app/services.rs`, `src/app/tests.rs`, `docs/plan/progress.md`,
  and `docs/process-observations.md`.
- Structure: `src/app/services.rs` now owns `AppServices`, the app side-effect boundary for jj
  mutation plans, previews, revision resolution, fetch/push helpers, view loading, view refresh, and
  graph reveal. `App` owns one `services: AppServices` field instead of the previous set of
  individual test-only runner fields.
- Scope control: command argv and preview semantics stayed in the existing jj plan types.
  `AppServices` only invokes those plans and returns their messages. `App` still decides when to
  invoke side effects and how status, mode, stack, and view transitions follow.
- Test surface: `src/app/tests.rs` uses a local `test_services()` helper for the standard app test
  doubles and overrides individual service functions through `app.services.*` only where a test
  needs a special failure, load, remote, or reveal behavior.
- Review repair: `test_services()` now overrides `new_trunk_run` so graph new-from-trunk app tests
  cannot fall through to the production default. A focused app test exercises graph `c` through
  `ViewEffect::RunNewTrunk`, mocked trunk/current revision resolution, mocked reveal, and the
  service call counter.
- Rework: after moving the production helpers, the shared workspace briefly failed `cargo check`
  because `App::load` did not initialize `services` and the old production wrappers still referenced
  moved imports (`resolve_exact_change_id`, `git_fetch`, `git_remotes`, and `new_trunk`). The repair
  routed those wrappers through `AppServices` and confirmed compilation before continuing cleanup.
- Size evidence: before Slice 2, `wc -l` reported 3,854 lines in `src/app.rs` and 3,899 lines in
  `src/app/tests.rs`. After Slice 2 and rustfmt, `src/app.rs` is 3,434 lines, `src/app/services.rs`
  is 332 lines, and `src/app/tests.rs` is 3,887 lines.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test app -- --test-threads=1` passed with 143 tests after the review repair.
  - full `cargo test` passed with 387 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed after `just md-fmt` applied Panache wrapping to this entry.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

## App Decomposition Slice 3

- Files changed: `src/app.rs`, `src/app/navigation.rs`, `src/app/tests.rs`, `docs/plan/progress.md`,
  and `docs/process-observations.md`.
- Structure: `src/app/navigation.rs` now owns startup argument parsing, `App::load`, detail view
  spec construction, direct top-level view entry, push/pop back-stack transitions, and log/default
  switching. `src/app.rs` still owns the event loop, dispatch, modal/action behavior, refresh, and
  status decisions that are not navigation-specific.
- Service boundary: initial app loading now creates `AppServices::default()` before loading the
  first view and calls `services.load_view(initial_spec)`. Pushed and switched views continue to
  load through `App::load_view_state`, which delegates to `AppServices`.
- Scope control: existing app call sites still call `self.push_detail`, `self.open_status`,
  `self.pop_view`, and related inherent methods. The moved methods use `pub(in crate::app)` so the
  boundary stays app-internal without widening crate-level API.
- Rework: the first extracted startup path still called `ViewState::load` directly. This was
  adjusted before final validation so startup view loading also crosses the Slice 2 service
  boundary.
- Review repair: focused app tests now cover direct `L` and `J` dispatch. `L` proves startup
  `jk log ...` args are reused and the back stack is cleared; `J` proves default view switching
  ignores startup log args and clears the stack. The app test loader now preserves graph `ViewSpec`s
  through a small `GraphView::test_with_spec` constructor so those assertions inspect the loaded
  spec instead of only the view variant.
- Size evidence: before Slice 3, `wc -l` reported 3,434 lines in `src/app.rs`, 332 lines in
  `src/app/services.rs`, and 3,919 lines in `src/app/tests.rs`. After Slice 3 and rustfmt,
  `src/app.rs` is 3,264 lines, `src/app/navigation.rs` is 193 lines, `src/app/services.rs` remains
  332 lines, and `src/app/tests.rs` is 3,921 lines.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test app -- --test-threads=1` passed with 145 app-related tests after the review repair.
  - full `cargo test` passed with 388 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

## App Decomposition Slice 4

- Files changed: `src/app.rs`, `src/app/action_flow.rs`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Structure: `src/app/action_flow.rs` now owns the shared action-output-backed preview key flow. It
  translates output keys into stay-open, close-completed, cancel-pending, and confirm-pending events
  for describe, commit, bookmark mutation, new, rebase, restore, revert, squash, absorb, push,
  operation recovery, operation target, and working-copy navigation previews.
- Dispatch shape: `handle_mode_key_event` calls `handle_common_action_preview_key` before the main
  `self.mode` match, so the repeated preview arms are no longer embedded in the modal dispatch.
  Abandon preview and typed abandon confirmation intentionally stay in `src/app.rs` because their
  recheck and exact-text confirmation behavior is action-specific.
- Scope control: existing `confirm_*` methods stayed on `App`, action output scrolling stayed in
  `action_output.rs`, and user-visible command output, cancellation messages, confirmation behavior,
  and result-close behavior were preserved.
- Size evidence: before Slice 4, `wc -l src/app.rs` reported 3,264 lines. After Slice 4 and rustfmt,
  `src/app.rs` is 2,889 lines and `src/app/action_flow.rs` is 345 lines, for 3,234 lines across the
  two files.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test app -- --test-threads=1` passed with 145 app-related tests.
  - full `cargo test` passed with 390 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed after `just md-fmt` applied Panache wrapping to this entry.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

## App Decomposition Slice 5

- Files changed: `src/app.rs`, `src/app/action_flow.rs`, `src/app/action_lifecycle.rs`,
  `src/app/tests.rs`, `docs/plan/progress.md`, and `docs/process-observations.md`.
- Structure: `src/app/action_lifecycle.rs` now owns action menu follow-up application, action prompt
  opening, preview opening, confirmation, result, stacked-view refresh, and abandon recheck flows
  for the existing preview-first jj actions. Existing dispatch call sites still call inherent `App`
  methods, but those methods now have an action-lifecycle home.
- Action flow boundary: `src/app/action_flow.rs` stays focused on common action-output preview key
  handling and confirmation dispatch. `src/app.rs` keeps modal/key dispatch, `execute_view`,
  `apply_view_effect`, view-menu handling, `run_new_trunk`, and `apply_diff_format`.
- Test boundary: `src/app/tests.rs` now imports action-output and action target types directly
  instead of receiving them through broad `src/app.rs` imports. Assertions and mocked service
  behavior were not weakened.
- Size evidence: before Slice 5, `wc -l src/app.rs src/app/action_flow.rs` reported 2,889 lines in
  `src/app.rs` and 345 lines in `src/app/action_flow.rs`. After Slice 5 and rustfmt, `src/app.rs` is
  1,355 lines, `src/app/action_flow.rs` remains 345 lines, and `src/app/action_lifecycle.rs` is
  1,563 lines.
- Rework: the first mechanical move put the lifecycle methods into `action_flow.rs`, which made the
  module too broad. The moved lifecycle code was split into `action_lifecycle.rs`, leaving
  `action_flow.rs` as the common preview-key mapper from Slice 4.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test app -- --test-threads=1` passed with 145 app-related tests.
  - full `cargo test` passed with 390 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed after `just md-fmt` applied Panache wrapping to this entry.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

## App Decomposition Slice 6

- Files changed: `src/app.rs`, `src/app/mode_input.rs`, `src/app/action_lifecycle.rs`,
  `src/app/tests.rs`, `docs/plan/progress.md`, and `docs/process-observations.md`.
- Structure: `src/app/mode_input.rs` now owns app modal input reduction for help, search, custom log
  revsets, copy/view/action menus, role prompts, text prompts, abandon confirmation, push remote
  selection, and handoff to common action-preview handling. `src/app.rs` keeps the terminal event
  loop, pending normal command prefix handling, normal binding dispatch, view-effect application,
  view-menu actions, refresh/fetch, and view-format reload behavior.
- Boundary: action preview scrolling and confirmation dispatch still live in
  `src/app/action_flow.rs`; selected action follow-ups, preview opening, result handling, and
  stacked repo-view refresh still live in `src/app/action_lifecycle.rs`. The moved prompt-plan
  helpers are visible only inside `crate::app` for existing behavior tests.
- Coverage: operation target restore/revert tests now cover a non-empty repo-view stack refresh and
  a stacked-refresh failure after the active operation log refresh succeeds. The tests assert the
  refresh call count and keep the result output/status inspectable.
- Size evidence: before Slice 6, `wc -l src/app.rs` reported 1,355 lines. After Slice 6 and rustfmt,
  `src/app.rs` is 781 lines, `src/app/mode_input.rs` is 603 lines, `src/app/action_lifecycle.rs`
  remains 1,563 lines, and `src/app/tests.rs` is 4,050 lines.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test app -- --test-threads=1` passed with 147 app-related tests.
  - full `cargo test` passed with 392 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code and `collapsible_if`
    findings in `src/file_show.rs`, `src/jj.rs`, `src/jj_rows.rs`, `src/bookmarks.rs`,
    `src/graph.rs`, and `src/operation_log.rs`.

## Interruption Packet F: Fetch Remote Selection

- Files changed: `src/jj_actions.rs`, `src/jj.rs`, `src/app/services.rs`, `src/app.rs`,
  `src/app/action_lifecycle.rs`, `src/app/action_flow.rs`, `src/app/mode_input.rs`,
  `src/app_screen.rs`, `src/tui.rs`, `src/command.rs`, `src/graph.rs`, `src/app/tests.rs`,
  `docs/plan/progress.md`, `docs/process-observations.md`, and `docs/plan/fragility-register.md`.
- Behavior: default fetch remains a direct `jj git fetch` on `f` and graph `gf`, but the raw result
  now stays inspectable in the shared action-output result overlay after the command runs. Explicit
  remote fetch is available through global `F` and graph `gr`; one remote opens the fetch preview
  directly, multiple remotes open a keyboard remote picker, and no-remotes or remote-list failures
  produce readable status plus a completed output overlay.
- Command contract: `JjGitFetch` owns default and remote-specific fetch argv construction. Remote
  fetch passes `--remote exact:<remote>` so selected names from `jj git remote list` are not treated
  as implicit glob patterns, and the preview/result context shows both the selected remote and exact
  pattern.
- Coverage: focused app tests cover direct default fetch result output, graph `gf`, graph `gr`,
  global remote-fetch help metadata, one-remote skip, multi-remote selection, no-remote and
  remote-list errors, remote preview confirmation, fetch failure output, refresh-failure output, and
  push remote prompt regressions. Command tests cover default and remote fetch argv plus the
  remote-list parser.
- Disposable proof: `/tmp/jk-fetch-proof.gmwDVS` created bare `origin` and `upstream` remotes,
  cloned them through `jj git clone`, added the second remote with `jj git remote add`, and proved
  `jj --no-pager git fetch`, `jj --no-pager git fetch --remote exact:origin`, and
  `jj --no-pager git fetch --remote exact:upstream`. A no-remote `/tmp` repo proved
  `jj --no-pager git remote list` emits no rows and `jj --no-pager git fetch --remote exact:origin`
  preserves the warning and error text: `No matching remotes for names: origin` and
  `No git remotes to fetch from`.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `FileShowView::new`,
    `ViewSpec::bookmarks`, and `FileListItem::row_text`.
  - `cargo test fetch -- --nocapture` passed with 14 focused tests.
  - `cargo test remote -- --nocapture` passed with 18 focused tests.
  - `cargo test git_fetch -- --nocapture` passed with 3 focused tests.
  - `cargo test app::tests -- --nocapture` passed with 141 app tests.
  - `cargo test git_fetch_remote_uses_exact_remote_pattern -- --nocapture` passed.
  - `cargo test parses_git_remotes_from_jj_remote_list_output -- --nocapture` passed.
  - full `cargo test` passed with 403 tests.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed after `just md-fmt` applied Panache wrapping to this entry.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code findings in
    `src/file_show.rs`, `src/jj.rs`, and `src/jj_rows.rs`, plus known `collapsible_if` findings in
    `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`.
- Rework: an attempted `cargo test` invocation used two test-name filters in one command, which
  Cargo rejected. The focused `JjGitFetch` argv and remote parser tests were rerun as separate
  commands and passed.
- Remaining risk: remote names still come from the current `jj git remote list` text format and are
  parsed as the first whitespace-delimited field. The selected remote is passed as an exact
  `jj git fetch` string pattern and recorded in `docs/plan/fragility-register.md`, but a stronger
  structured remote API would be preferable if sync flows expand.
- Next recommended slice: Packet G: File Viewing And Wrap Modes, after review checks the fetch
  target wording, exact remote pattern contract, output preservation, and push remote selection
  regression coverage.

## Interruption Packet H

- Files changed: `Justfile`, `docs/agent/workflow.md`, `docs/agent/testing.md`,
  `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, `docs/process-observations.md`
- Verification: `just md-check`; `just check`
- Validation note: no `cargo run` smoke was run because this packet changed docs and validation
  tooling only, not runtime behavior.
- Residual risk: the repo still has known Rust warnings and clippy blockers in the current baseline,
  so future handoffs must continue to list exact blockers and whether they are unchanged when a
  truly warning-free proof is not available.
- Next recommended slice: Packet 17: Undo/Redo From Operation Log.

## Interruption Packet G: File Viewing And Wrap Modes

- Files changed: `src/sticky_file_view.rs`, `src/file_show.rs`, `src/show.rs`, `src/diff.rs`,
  `src/command.rs`, `src/view_state.rs`, `src/app.rs`, `src/app/action_lifecycle.rs`,
  `src/app/tests.rs`, list-style view modules updated for exhaustive ignored command handling,
  `docs/plan/progress.md`, and `docs/process-observations.md`.
- Behavior: show, diff, and file-show document views now default to the existing wrapped
  `Wrap { trim: false }` rendering, expose `zw` to toggle no-wrap mode, and expose `zh` / `zl` for
  horizontal movement in no-wrap mode. No-wrap mode clips long rendered lines instead of reflowing
  them and clamps horizontal offset to the rendered document width for the current viewport width. A
  follow-up review repair made the app/view clamp path width-aware so refreshes and terminal resizes
  also clamp no-wrap horizontal offsets.
- Ownership: `src/sticky_file_view.rs` owns the shared `DocumentDisplayMode` and `DocumentViewport`
  policy, including Ratatui wrapping and horizontal scroll offset. File-show owns its
  single-document viewport state directly; show and diff store viewport state inside
  `StickyFileDocument`. `src/tui.rs` was not changed.
- Coverage: `sticky_file_view` rendering tests cover wrapped Markdown-like long lines, no-wrap
  clipping, horizontal offset revealing later columns, and sticky fixed/body rendering under
  no-wrap. `file_show` tests cover toggle behavior, horizontal clamping, vertical scroll stability,
  source-line search, refresh clamping, and exact-path copy. Show/diff tests cover copy/file labels
  and file navigation under horizontal scroll. Follow-up clamp tests cover shared
  `StickyFileDocument` content shrink, file-show refresh/content shrink, and show/diff viewport
  width revalidation. `command.rs` tests prove generated help exposes wrap commands only in document
  contexts.
- Validation:
  - `cargo check` passed with the existing dead-code warnings for `ViewSpec::bookmarks` and
    `FileListItem::row_text`.
  - `cargo test sticky_file_view` passed with 5 focused tests.
  - `cargo test file_show` passed with 13 focused tests.
  - `cargo test horizontal_scroll` passed with 4 focused tests.
  - `cargo test document_help` passed with 1 focused test.
  - The follow-up repair first attempted multiple Cargo test-name filters in one command; Cargo
    rejected that invocation, and the focused clamp tests were rerun with the `horizontal_offset`
    filter.
  - `cargo test horizontal_offset` passed with 5 focused tests after the follow-up clamp repair.
  - Plain `cargo test` passed with 417 tests. The earlier note about
    `app::tests::operation_restore_confirm_refreshes_non_empty_repo_stack` was stale after the
    app-test refresh counters were split; the parallel shared-counter race is no longer present in
    this tree.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code findings in `src/jj.rs` and
    `src/jj_rows.rs`, plus known `collapsible_if` findings in `src/bookmarks.rs`, `src/graph.rs`,
    and `src/operation_log.rs`.
- Fragility: no rendered-output parser or ANSI assumptions changed, so
  `docs/plan/fragility-register.md` was not updated.
- Remaining risk: render-time viewport clamping is defensive, but persisted state is now also
  clamped through refresh and resize paths. Review should still check terminal resize behavior in a
  live TUI because unit tests cover the state contract rather than an end-to-end terminal session.
- Next recommended slice: review Packet G for no-wrap ergonomics and consider whether document
  status hints should advertise `zw` after the first user-facing pass.

## Packet 34a: Split Process-Boundary Spike

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`,
  `docs/plan/fragility-register.md`, and `docs/process-observations.md`.
- Behavior: Packet 34a is now inserted before Packet 34 as a docs-only prerequisite. It records that
  split command planning is clear, but interactive editor handoff through the current
  captured-process runner is not proven.
- Command contract: the planned command shapes are preserved exactly: bare `jj split` for the
  visible/current `@`, and `jj split --revision exactly(change_id("<id>"), 1)` for exact graph
  targets.
- Process boundary: no-fileset split delegates patch selection to `jj`'s diff editor and may also
  invoke description editing. `jk` must not present Packet 34 as an in-app patch editor or imply it
  can choose hunks without handing control to `jj`.
- Packet 34 dependency: implementation must either add and prove an interactive process or
  terminal-suspension runner for real editor handoff, or explicitly ship only preview/readable
  failure semantics with raw output preserved.
- Evidence: this spike cites the Packet 34 exploration finding from the gpt-5.5 high explorer. No
  new mutation proof was run, and no command was executed in this repository to prove `jj split`.
- Validation: `just md-check`.
- Review outcome: `gpt-5.5` high review `019e470b-9aaf-7981-9204-5db8eedc4fd5` found no findings,
  checked command shapes against `jj --no-pager split --help`, and passed `just md-check`
  successfully.
- Remaining risk: Packet 34 still needs an implementation decision and proof for terminal/editor
  lifecycle before it can execute split interactively. The docs now make that risk blocking instead
  of letting implementation infer an unproven runner capability.
- Next recommended slice: choose and prove the Packet 34 implementation boundary, preferably a
  bounded interactive process or terminal-suspension runner spike before the product split flow, or
  an explicit preview/readable-failure boundary if that is the cleaner path.

## Packet 34b: Split Process-Boundary Spike

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.
- Behavior: Packet 34b resolves the Packet 34a open decision in favor of a dedicated implementation
  packet before the product split flow. Packet 34c now owns the terminal-suspension and inherited
  stdio runner; Packet 34 must not execute no-fileset interactive split through the captured
  `Command::output()` runner.
- Code inspection evidence: `src/jj.rs` direct runners (`run_direct_args`, `run_direct_args_stdout`,
  and related helpers) use `Command::output()`, so stdout and stderr are captured pipes and the
  child process does not inherit the app TTY. `src/app.rs` enters the TUI with `ratatui::run`, and
  Ratatui 0.30's `run()` initializes raw mode plus alternate screen before invoking the app closure
  and restores only after the closure returns.
- Process-boundary decision: real `jj split` editor handoff needs a mid-run terminal suspension that
  leaves raw mode and alternate screen, spawns `jj` with inherited stdin/stdout/stderr, waits for
  the child, and restores the Ratatui terminal before the event loop resumes.
- `/tmp` proof: disposable repo `/tmp/jk-packet34b-proof.upUcu2` was initialized with
  `jj --no-pager git init`, then `split.txt` was added from that repo's cwd. From that same proof
  repo cwd, `jj --no-pager split --tool false` failed with `Error: Failed to edit diff`, and bare
  `jj --no-pager split` failed under the captured non-TTY process with
  `failed to set up terminal: Device not configured (os error 6)`.
- `/tmp` process-only success proof: from the same proof repo cwd,
  `jj --no-pager split split.txt -m selected` succeeded and printed selected/remaining change
  summaries. This proves that noninteractive fileset split can run as a normal captured process, but
  it is not the planned product path because no-fileset Packet 34 split delegates patch selection to
  jj's diff editor.
- Validation: `just md-check`.
- Review outcome: gpt-5.5 high review `019e4717-5e19-7c20-8a26-db2d1c312b06` found no findings,
  verified the existing `Command::output()` / `ratatui::run` boundary and Packet 34c / 34 gating,
  and ran `jj --no-pager split --help` but did not rerun `just md-check`.
- Remaining risk: the dedicated runner still needs Rust implementation and a live-terminal manual
  proof for a real inherited-stdio editor handoff. Unit tests can cover runner intent and
  restoration branches, but they cannot by themselves prove a human diff editor session behaves
  correctly in every terminal.
- Next recommended slice: Packet 34c, the interactive split process runner, before Packet 34 Split
  Guided Flow.

## Packet 34c: Interactive Split Process Runner

- Files changed: `src/interactive_process.rs`, `src/jj.rs`, `src/main.rs`, `docs/plan/progress.md`,
  and `docs/process-observations.md`.

- Behavior: added a focused `interactive_process` boundary for commands that must inherit the app's
  stdin, stdout, and stderr instead of using `Command::output()`. The runner suspends the Ratatui
  terminal by showing the cursor and leaving raw mode / the alternate screen, spawns and waits for
  the child with inherited stdio, then re-enables raw mode, re-enters the alternate screen, and
  clears the Ratatui terminal so the event loop can draw the next frame.

- Command contract: `jj::interactive_jj_command` builds the future Packet 34 command path as
  `jj --no-pager <args...>` with inherited stdio. Existing captured command helpers and action plans
  still use their current `Command::output()` paths.

- Failure behavior: spawn, wait, nonzero exit, terminal suspension failure, and terminal restore
  failure all return readable errors that name the command label. A restore guard attempts to put
  the app terminal back even if the spawner panics while the command is suspended. Because child
  stdout/stderr are inherited instead of captured, runner errors preserve the child status but not
  child stderr text.

- Coverage: unit tests cover inherited-stdio and cwd intent, restoration after spawn errors, nonzero
  statuses, suspension failures, restore failures, and panic-adjacent guard restoration. `jj`
  command tests cover the interactive `jj --no-pager split --revision <exact-revset>` argv and
  inherited-stdio marker.

- Disposable proof: `/tmp/jk-packet34c-proof.Grlzej` was initialized with `jj --no-pager git init`.
  From that repo's cwd, `split.txt` was created and tracked with
  `jj --no-pager file track split.txt`. No `jj` mutation command was run from
  `/Users/joshka/local/jk`.

- Runner proof commands:

  ```sh
  JK_INTERACTIVE_PROOF_REPO=/tmp/jk-packet34c-proof.Grlzej \
    cargo test real_runner_reports_jj_failure_from_tmp_repo -- --ignored --test-threads=1

  JK_INTERACTIVE_PROOF_REPO=/tmp/jk-packet34c-proof.Grlzej \
    cargo test real_ratatui_runner_reports_jj_failure_from_tmp_repo \
      -- --ignored --nocapture --test-threads=1
  ```

  The first proof used the real process spawner with a fake lifecycle. The second ran in a PTY and
  used the real Ratatui terminal lifecycle. Both executed `jj --no-pager split --tool false` with
  the child cwd set to the `/tmp` proof repo and returned a clean nonzero child status. The
  `Error: Failed to edit diff` text was observed as inherited child terminal output while the app
  terminal was suspended, not as captured runner result text.

- Live-editor proof note: a default diff-editor split cancellation/completion proof was not
  attempted because the automated PTY cannot safely drive an arbitrary user-configured diff editor
  without risking a blocked session. The PTY proof still exercised the terminal suspension,
  inherited-stdio spawn, wait, and restore path.

- Validation:
  - `cargo check` passed with the existing dead-code warnings for `ViewSpec::bookmarks` and
    `FileListItem::row_text`.
  - `cargo test interactive_process -- --test-threads=1` passed with 7 tests and 2 ignored manual
    proofs.
  - `cargo test interactive_jj_command_inherits_stdio_and_keeps_no_pager -- --test-threads=1`
    passed.
  - full `cargo test` passed with 425 tests and 2 ignored manual proofs.
  - `rustup run nightly cargo fmt` completed with existing rustfmt config warnings.
  - `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings.
  - `just md-check` passed after `just md-fmt` applied Panache wrapping to this entry.
  - `cargo clippy -- -D warnings` remains blocked by the known dead-code findings in `src/jj.rs` and
    `src/jj_rows.rs`, plus known `collapsible_if` findings in `src/bookmarks.rs`, `src/graph.rs`,
    and `src/operation_log.rs`.
  - `cargo run` was smoke-tested in a PTY and quit with `q`; it rendered the TUI but was not
    warning-free because of the existing two `cargo check` warnings.

- Review prompt: review Packet 34c for terminal lifecycle correctness, inherited stdio, restoration
  on success/failure/panic-adjacent paths, exact `/tmp` proof cwd discipline, tests, docs, and
  whether it is small enough for Packet 34 to depend on without absorbing split UI behavior.

- Remaining risk: the runner is now proven for controlled noninteractive failure and PTY terminal
  restoration, but Packet 34 still needs explicit post-command status/result visibility plus a
  human/editor-level proof for real default diff-editor cancellation or completion once product
  split wiring exists.

- Next recommended slice: Packet 34 Split Guided Flow can depend on this runner for no-fileset
  `jj split` editor handoff without adding an in-app patch editor, but it must not rely on inherited
  child output alone for result visibility after the alternate screen is restored.

## Packet 34c Review Repair

- Files changed: `src/interactive_process.rs`, `docs/plan/next-implementation-slices.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`.
- Review findings addressed: Packet 34c docs now distinguish inherited-stdio terminal output from
  captured runner result text, and future Packet 34 is required to design explicit post-command
  status/result visibility. The proof notes no longer claim that `Error: Failed to edit diff` was
  captured in the runner error; it was observed child terminal output while the app terminal was
  suspended.
- Test repair: ignored proof tests now canonicalize both `/tmp` and `JK_INTERACTIVE_PROOF_REPO`
  before accepting the proof repo, so paths such as `/tmp/../Users/...` or symlinks resolving
  outside canonical `/tmp` are rejected. The Ratatui ignored proof now asserts a clean child nonzero
  status separately from terminal restore failure wording, and the cleanup `ratatui::try_restore()`
  call runs after the runner result is captured without masking that assertion.
- Final 5.5 re-review: `019e473c-f2f0-7ab2-b936-9d0261910255` (`gpt-5.5`, high) reported no findings
  and verified the prior three review findings were fixed: output visibility wording, canonical
  `/tmp` proof path validation, and the clean restore-status assertion.
- Validation: `cargo test interactive_process -- --test-threads=1` passed with 7 passed, 2 ignored;
  `rustup run nightly cargo fmt --check` passed with existing rustfmt config warnings;
  `just md-check` passed; `cargo check` passed with existing dead-code warnings for
  `ViewSpec::bookmarks` and `FileListItem::row_text`. Optional note: ignored live-terminal proofs
  were not rerun; the repair tightened their checks only.

## Packet 34: Split Guided Flow

- Files changed: `src/action_menu.rs`, `src/app.rs`, `src/app/action_flow.rs`,
  `src/app/action_lifecycle.rs`, `src/app/mode_input.rs`, `src/app/services.rs`,
  `src/app_screen.rs`, `src/graph.rs`, `src/jj.rs`, `src/jj_actions.rs`, `src/jj_rows.rs`,
  `src/tui.rs`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.

- Behavior: graph action menus now open a preview-first split flow. A visible current `@` row uses
  bare `jj split`; an exact non-current graph row uses
  `jj split --revision exactly(change_id("<id>"), 1)`. Multi-source graph menus and ambiguous rows
  do not expose split. The preview says no fileset is passed, `jj` owns diff-editor patch selection
  and any description editing, and `jk` is not an in-app patch editor.

- Execution boundary: confirmation uses the Packet 34c inherited-stdio runner through
  `JjSplitPlan::run_interactive`. App tests use the action service seam, but the default product
  service requires a live Ratatui terminal and runs `jj --no-pager split ...` with inherited stdin,
  stdout, and stderr instead of `Command::output()`.

- Result visibility: success and failure return to an app-owned result overlay. The result names the
  command, child/runner status, live-output caveat, `jj undo`, and `jj op show -p` without claiming
  captured child stderr. Success refreshes the active view and reveals the exact target or current
  `@` in recent work when possible; failures keep readable app-owned status because inherited child
  output may disappear after alternate-screen restore.

- Disposable proof: `/tmp/jk-packet34-proof.6Kx9Pw` was initialized with `jj --no-pager git init`.
  From that repo's cwd, `split.txt` was created and `jj --no-pager file track split.txt` was run. No
  product split flow mutated `/Users/joshka/local/jk`.

- Runner proof commands:

  ```sh
  JK_INTERACTIVE_PROOF_REPO=/tmp/jk-packet34-proof.6Kx9Pw \
    cargo test real_runner_reports_jj_failure_from_tmp_repo -- --ignored --test-threads=1

  JK_INTERACTIVE_PROOF_REPO=/tmp/jk-packet34-proof.6Kx9Pw \
    cargo test real_ratatui_runner_reports_jj_failure_from_tmp_repo \
      -- --ignored --nocapture --test-threads=1
  ```

  Both proof tests executed `jj --no-pager split --tool false` with the child cwd forced to the
  `/tmp` proof repo. The first used the real process spawner with a fake lifecycle; the second used
  the real Ratatui lifecycle in a PTY. Both returned clean nonzero child status. The
  `Error: Failed to edit diff` text was inherited live terminal output, not captured result text.

- Validation so far: `cargo check`; `cargo test split -- --test-threads=1`;
  `cargo test action_menu -- --test-threads=1`; `cargo test app::tests::split -- --test-threads=1`;
  `cargo test jj_actions::tests::split -- --test-threads=1`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; `just md-check`.

- Scope repair: initial 5.5 review `019e4750-9d7f-7be2-a0ce-8ab112a771f7` found no split behavior
  blockers, but it did flag a medium scope/atomicity issue because the Post-Packet-34 App Module
  Coherence Gate docs were mixed into Packet 34. Main orchestration created child jj change
  `plzltlkx Plan app coherence gate`, edited parent `sxolvtpo`, mini worker
  `019e4753-c94b-7ca0-a304-d0664acbabf3` removed the gate hunks from the Packet 34 parent, and mini
  worker `019e4755-aecd-7631-b475-1424c1c9deb6` restored the gate docs in the child. Both docs
  workers ran `just md-check`; the second also ran `just md-fmt`.

- Final 5.5 review `019e4757-73ed-7ca3-a3c4-0980e50f1daf` reported no blocking findings: the Packet
  34 parent no longer contains gate text, and the child is docs-only with the gate.

- Post-repair validation: `cargo check` passed with the known `ViewSpec::bookmarks` and
  `FileListItem::row_text` warnings; `cargo test split -- --test-threads=1`;
  `cargo test   action_menu -- --test-threads=1`;
  `cargo test app::tests::split -- --test-threads=1`;
  `cargo   test jj_actions::tests::split -- --test-threads=1`; full `cargo test` passed with 437
  passed / 2 ignored; `rustup run nightly cargo fmt --check` passed with existing rustfmt config
  warnings; `just md-check` passed; `cargo clippy -- -D warnings` still failed on the known
  dead-code warnings plus `collapsible_if` in `src/bookmarks.rs`, `src/graph.rs`, and
  `src/operation_log.rs`.

- Warning / blocker status: `cargo check` and `cargo run` still report the existing dead-code
  warnings for `ViewSpec::bookmarks` and `FileListItem::row_text`. `cargo clippy -- -D warnings`
  remains blocked by those two dead-code warnings plus the known `collapsible_if` findings in
  `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`. PTY `cargo run` rendered the TUI
  and quit with `q`, but was not warning-free because of the same two `cargo check` warnings.

- Review prompt: review Packet 34 for honest split/editor semantics, exact target handling,
  inherited-stdio runner use, post-command app-owned status/result visibility, refresh/reveal
  behavior, noninteractive failure behavior, `/tmp` proof cwd discipline, tests, docs, and evidence
  that the flow does not pretend to be an in-app patch editor.

- Remaining risk: a live default diff-editor cancel/complete proof was not attempted because Codex
  cannot safely drive an arbitrary user-configured editor in the PTY without risking a blocked
  session. The controlled failure proof covers terminal suspension, inherited stdio, wait, and
  restore; human/editor-level behavior should still be reviewed manually.

- Next recommended slice: Post-Packet-34 App Module Coherence Gate before Packet 35 or another
  rewrite/action packet adds more app dispatch. Packet 34 left `src/app.rs` at about 841 LOC after
  substantial decomposition into app submodules; that is acceptable temporarily but should not be
  treated as completion until a gpt-5.5 high coherence pass confirms the file is only terminal-loop
  and thin app-level routing, or extracts any remaining modal/action/view-menu policy to a clearer
  owner.

## 2026-05-20 App Refactor Completion Gate Planning

- Files changed: `docs/plan/next-implementation-slices.md`, `docs/plan/progress.md`, and
  `docs/process-observations.md`.

- Planning outcome: added a required Post-Packet-34 App Module Coherence Gate before Packet 35. The
  gate uses concept ownership and reader cognitive load as acceptance criteria, with line count only
  as a soft review signal.

- Evidence: current app-module line counts are:

  ```text
       841 src/app.rs
       389 src/app/action_flow.rs
      1813 src/app/action_lifecycle.rs
       642 src/app/mode_input.rs
       193 src/app/navigation.rs
       362 src/app/services.rs
      4521 src/app/tests.rs
  ```

- Target rationale: the plan now treats 500-700 LOC as a reasonable post-pass target band for
  `src/app.rs`, with about 750 LOC as a soft review trigger. The threshold is not a hard target; a
  larger file is acceptable only when the remaining methods are a coherent terminal loop and thin
  app-level routing.

- Remaining risk: no Rust refactor was performed in this planning pass. The app refactor should not
  be marked complete until the gate is implemented or a gpt-5.5 high review explicitly records why
  no further extraction is warranted.

## 2026-05-20 Post-Packet-34 App Module Coherence Gate

- Files changed: `src/app.rs`, `src/app/action_lifecycle.rs`, `src/app/mode_input.rs`,
  `src/app/navigation.rs`, `src/app/services.rs`, `src/app/tests.rs`, `docs/agent/architecture.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`.

- Gate outcome: the app refactor is complete for the current Packet 34 boundary. `src/app.rs` now
  reads as terminal-loop and app-level routing: drawing, key-prefix dispatch, normal command
  routing, refresh, view execution, and `ViewEffect` application. No named follow-up packet is
  required before Packet 35 on app-module coherence grounds.

- Ownership decisions:

  - `src/app/action_lifecycle.rs` now owns action-menu opening, default fetch execution, and
    new-from-trunk result handling, keeping action preview/result lifecycle outside `src/app.rs`.
  - `src/app/mode_input.rs` now owns copy-menu opening beside modal key reducers.
  - `src/app/navigation.rs` now owns log revset prompts, custom log revset application, view-menu
    selection, and diff-format application as global view-selection policy.
  - `src/app/services.rs` now owns App service-forwarding methods. The forwarding remains a thin
    test seam, but it no longer forces `src/app.rs` to import every action-plan type.
  - Focused app tests remain in `src/app/tests.rs` because they exercise cross-owner app state and
    service seams; imports are now explicit instead of relying on `src/app.rs` re-exports.

- Line-count evidence:

  ```text
  Before:
       841 src/app.rs
       389 src/app/action_flow.rs
      1813 src/app/action_lifecycle.rs
       642 src/app/mode_input.rs
       193 src/app/navigation.rs
       362 src/app/services.rs
      4521 src/app/tests.rs

  After:
       505 src/app.rs
       389 src/app/action_flow.rs
      1929 src/app/action_lifecycle.rs
       657 src/app/mode_input.rs
       281 src/app/navigation.rs
       490 src/app/services.rs
      4526 src/app/tests.rs
  ```

- Validation so far: `cargo test app::tests::view_menu -- --test-threads=1`;
  `cargo test app::tests::fetch -- --test-threads=1`;
  `cargo test app::tests::split -- --test-threads=1`; `cargo check`; full `cargo test`;
  `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`; attempted
  `cargo clippy -- -D warnings`; `just md-fmt`; `just md-check`.

- Warning / blocker status: `cargo check` passes with the known `ViewSpec::bookmarks` and
  `FileListItem::row_text` warnings. `cargo clippy -- -D warnings` remains blocked only by the known
  baseline findings: those two dead-code warnings plus `collapsible_if` in `src/bookmarks.rs`,
  `src/graph.rs`, and `src/operation_log.rs`.

- Remaining risk: `src/app/action_lifecycle.rs` is still large, but it is large around one concept:
  app-owned mutation/action preview lifecycle. Future rewrite packets should add behavior there, or
  split a narrower action owner when a concrete sub-lifecycle becomes independently readable, rather
  than putting action policy back into `src/app.rs`.

## Packet 35: Duplicate Guided Flow

- Files changed: `src/action_menu.rs`, `src/app/action_flow.rs`, `src/app/action_lifecycle.rs`,
  `src/app/mode_input.rs`, `src/app/services.rs`, `src/app/tests.rs`, `src/app_screen.rs`,
  `src/graph.rs`, `src/jj.rs`, `src/jj_actions.rs`, `src/tui.rs`, `docs/plan/fragility-register.md`,
  `docs/plan/progress.md`, and `docs/process-observations.md`.

- Behavior: action menus now expose duplicate only for one exact selected graph or detail-view
  revision target. Multi-source graph menus intentionally exclude duplicate; Packet 35 does not
  implement bulk/range duplicate, duplicate-and-rebase, bookmark movement, or target guessing from
  rendered labels.

- Command contract: `JjDuplicatePlan` runs one positional exact revset:
  `jj duplicate exactly(change_id("<id>"), 1)`. This shape was checked against
  `jj --no-pager duplicate --help`, which documents `[REVSETS]...` positional duplicate sources.

- Result visibility: confirmation uses the normal captured-output runner, so stdout/stderr from
  `jj duplicate` are preserved in the action output surface on both success and failure. Success
  refreshes the active view, then reveals the original source in recent work when the active view
  can actually run graph reveal logic. Detail views refresh without claiming a graph reveal that
  never ran. The result and status keep `jj undo` and `jj op show -p` visible.

- Disposable proof: `/tmp/jk-packet35-proof.oveqWn` was initialized with `jj --no-pager git init`.
  From that repo's cwd, `file.txt` was created, tracked, and described. From the same proof repo
  cwd, `jj --no-pager duplicate 'exactly(change_id("qwzxmoltolmpywqvnzouwlpprynkryqk"), 1)'`
  succeeded and printed `Duplicated fb0ff0e682c1 as qqrwmtko d5ff99ef source change`; a follow-up
  `jj --no-pager undo` restored the pre-duplicate operation. No duplicate or undo command was run
  from `/Users/joshka/local/jk`.

- Validation: `jj --no-pager duplicate --help`; disposable `/tmp` duplicate and undo proof;
  `cargo check` passed with the known `ViewSpec::bookmarks` and `FileListItem::row_text` warnings;
  `cargo test duplicate -- --test-threads=1`; `cargo test action_menu -- --test-threads=1`;
  `cargo test app::tests::duplicate -- --test-threads=1`;
  `cargo test app::tests::split -- --test-threads=1`;
  `cargo test jj_actions::tests::duplicate -- --test-threads=1`;
  `cargo test jj_actions::tests::split -- --test-threads=1`; full `cargo test` passed with 443
  passed / 2 ignored; `rustup run nightly cargo fmt`; `rustup run nightly cargo fmt --check`;
  `just md-fmt`; `just md-check`; and `just check`.

- Warning / blocker status: `cargo clippy -- -D warnings` remains blocked by the known baseline
  findings: dead code for `ViewSpec::bookmarks` and `FileListItem::row_text`, plus `collapsible_if`
  in `src/bookmarks.rs`, `src/graph.rs`, and `src/operation_log.rs`.

- Remaining risk: exact reveal of the duplicated change is not implemented because that would parse
  current human `jj duplicate` output. The bounded fallback is visible in preview, result text, and
  the fragility register. A future packet can add a narrow parser or structured result contract if
  exact duplicate reveal becomes important.

- Next recommended slice: review Packet 35 for exact source targeting, single-source menu policy,
  command shape, output preservation, source fallback wording, docs accuracy, and whether a future
  structured result contract is worth a separate packet before adding more rewrite flows.

## Packet 36: Bookmark Tracking Metadata Contract

- Files changed: `src/jj_rows.rs`, `src/bookmarks.rs`, `src/jj.rs`,
  `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and `docs/process-observations.md`.

- Metadata/template shape: the bookmark metadata side channel now emits JSONL with `name`, `remote`,
  `tracked`, `tracking_present`, `synced`, `target_change_id`, and `target_commit_id` from
  `jj bookmark list -T`. Rendered bookmark labels are still preserved for presentation and fallback
  row text.

- Behavior: `BookmarkItem` now carries a typed row state. Local rows distinguish local-only,
  tracked, untracked-remote-present, and ambiguous local tracking state. Remote rows carry the
  remote name, tracked/untracked state, synced state, whether tracked local state is present, and
  whether a local peer row is proven present, absent, or unknown.

- Conservative policy: missing, malformed, or row-count-mismatched metadata becomes
  `BookmarkRowState::Unknown`. Default bookmark output is treated as only visible-row coverage, so a
  local row stays ambiguous unless `--all-remotes` output proves local-only/tracked state. Tracking
  and forget mutations remain unimplemented and disabled; existing local delete remains gated on an
  explicit local state rather than rendered labels.

- Validation: `jj --no-pager bookmark list --help`; `jj --no-pager help -k templates`; manual
  current-repo template sample with `--all-remotes`; `cargo test jj_rows -- --test-threads=1`;
  `cargo test bookmark -- --test-threads=1`; `cargo check`; full `cargo test` passed with 447 passed
  / 2 ignored after the review repair; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo   fmt --check`; `just md-fmt`; and `just md-check`.
  `cargo clippy -- -D warnings` was attempted separately and failed on the known baseline findings.

- Warning / blocker status: `cargo check` passes with the known `ViewSpec::bookmarks` and
  `FileListItem::row_text` warnings. `cargo clippy -- -D warnings` remains blocked by the known
  baseline findings: those two dead-code warnings plus `collapsible_if` in `src/bookmarks.rs`,
  `src/graph.rs`, and `src/operation_log.rs`.

- Remaining risk: row-order pairing is still a side-channel contract between two `jj bookmark list`
  invocations. The packet now fails closed on count mismatch, but exact tracking actions still need
  a dedicated gating pass before Packet 38 or Packet 39 enables forget/track/untrack mutations.

- Next recommended slice: review Packet 36 for metadata truthfulness, row-order mismatch behavior,
  default-output ambiguity, and preservation of existing local bookmark delete behavior before
  Packet 37 rename or Packet 38/39 remote-tracking mutations.

## 2026-05-20 Post-Packet-36 App Module Refactor Gate

- Files changed: `src/app/action_lifecycle.rs`, `src/app/action_lifecycle/*.rs`, `src/app/tests.rs`,
  `src/app/tests/*.rs`, `docs/plan/progress.md`, and `docs/process-observations.md`.

- Gate outcome: the post-Packet-36 refactor gate is implemented without changing user-visible
  behavior. `src/app.rs` stays at 505 LOC and remains terminal-loop and app-level routing only.
  `src/app/action_lifecycle.rs` is now a 10-line module root, with lifecycle ownership split into
  action entry/prompt setup, preview opening, general completions, rewrite completions, and small
  shared wording helpers.

- Test outcome: `src/app/tests.rs` is now a 15-line module root. App tests are split by behavior
  group, with shared fixtures and service seams isolated in `src/app/tests/support.rs`.

- Line-count evidence:

  ```text
  Before:
       505 src/app.rs
      2045 src/app/action_lifecycle.rs
      4744 src/app/tests.rs

  After:
       505 src/app.rs
        10 src/app/action_lifecycle.rs
       584 src/app/action_lifecycle/completion.rs
       362 src/app/action_lifecycle/entry.rs
       694 src/app/action_lifecycle/preview.rs
       417 src/app/action_lifecycle/rewrite_completion.rs
        70 src/app/action_lifecycle/shared.rs
        15 src/app/tests.rs
       303 src/app/tests/abandon_actions.rs
       392 src/app/tests/bookmark_actions.rs
       532 src/app/tests/command_navigation.rs
       351 src/app/tests/describe_commit_actions.rs
       533 src/app/tests/detail_restore_actions.rs
       349 src/app/tests/operation_actions.rs
       529 src/app/tests/rewrite_actions.rs
       539 src/app/tests/support.rs
       479 src/app/tests/sync_actions.rs
       778 src/app/tests/working_copy_actions.rs
  ```

- Validation: `cargo check`; `cargo test app:: -- --test-threads=1`; focused Packet 34/35/36
  coverage with `cargo test split -- --test-threads=1`, `cargo test duplicate -- --test-threads=1`,
  and `cargo test bookmark -- --test-threads=1`; full `cargo test`; `rustup run nightly cargo fmt`;
  `rustup run nightly cargo fmt --check`; `just md-check`; and attempted
  `cargo clippy -- -D warnings`.

- Warning / blocker status: `cargo check` still reports the known `ViewSpec::bookmarks` and
  `FileListItem::row_text` warnings. `cargo clippy -- -D warnings` remains blocked by known baseline
  findings outside this refactor unless they are fixed separately.

- Remaining risk: this is a structural refactor over app action lifecycle and tests. The proof is
  compiler and test coverage rather than manual TUI driving, because no behavior or terminal
  interaction contract intentionally changed.

## Packet 37: Bookmark Rename Flow

- Files changed: `src/app.rs`, `src/app_screen.rs`, `src/app/action_lifecycle/entry.rs`,
  `src/app/action_lifecycle/shared.rs`, `src/app/mode_input.rs`,
  `src/app/tests/bookmark_actions.rs`, `src/app/tests/support.rs`, `src/command.rs`, `src/jj.rs`,
  `src/jj_actions.rs`, `src/tui.rs`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`, and
  `docs/plan/screens/bookmarks.md`, `docs/process-observations.md`, and
  `docs/tutorials/bookmarks-and-conflicts.md`.

- Behavior: the bookmarks view now exposes bookmark rename through `br`. Rename starts only when the
  selected row has Packet 36 metadata proving one exact local bookmark name. Remote, unknown, and
  non-bookmark contexts do not open a rename prompt. The prompt collects one new bookmark name,
  rejects empty, unchanged, and obvious invalid names before preview, then opens the existing
  preview-first ActionOutput flow.

- Command shape: rename uses `jj bookmark rename <old> <new>` with old and new names passed as argv
  values, never through a shell string, and never with `--overwrite-existing`. The preview shows the
  old name, new name, exact command label, duplicate-name failure expectation, confirmation prompt,
  and `jj undo` recovery.

- Validation boundary: `jk` does not duplicate the complete jj bookmark grammar. It rejects a
  conservative subset before execution: empty or unchanged names, whitespace/control characters,
  option-like names, remote-syntax `@`, empty path components, component-leading dots, trailing dot
  or `.lock`, `..`, and common Git-ref reserved characters. Deeper grammar and duplicate-name
  failures are left to jj so raw CLI output remains authoritative.

- Proof: disposable repo `/tmp/jk-packet37-rename-proof.vn476I` verified successful rename,
  duplicate-name failure, and undo. Commands were run with cwd set to that proof repo:
  `jj --no-pager bookmark rename packet37-old packet37-new` changed the visible row to
  `packet37-new`; `jj --no-pager bookmark rename packet37-new packet37-existing` failed with
  `Error: Bookmark already exists: packet37-existing`; `jj --no-pager undo` restored `packet37-old`.

- Validation: `cargo test bookmark -- --test-threads=1`;
  `cargo test app::tests::bookmark_actions -- --test-threads=1`;
  `cargo test bookmark_rename -- --test-threads=1`; full `cargo test` passed with 453 passed / 2
  ignored; `cargo check`; `cargo clippy -- -D warnings`; `rustup run nightly cargo fmt --check`;
  `just md-check`; and `just check`.

- Review repair: bookmark rename prompt input now reaches validation unchanged, the app-level
  nonlocal rename rejection covers an explicit `BookmarkRowState::Remote`, and the duplicate-name
  failure path preserves `Bookmark already exists` through the test service mock and action output.

- Remaining risk: the early validator is intentionally conservative and may reject a jj-supported
  exotic bookmark name. That is preferable for this mutation path until `jk` has structured jj
  ref-name validation; accepted names and all command execution still use argv values rather than
  display-label inference.

## Packet 38: Bookmark Forget Flow

- Files changed: `src/app.rs`, `src/app/action_lifecycle/entry.rs`, `src/app/mode_input.rs`,
  `src/app/tests/bookmark_actions.rs`, `src/bookmarks.rs`, `src/command.rs`, `src/jj.rs`,
  `src/jj_actions.rs`, `src/jj_rows.rs`, `src/tui.rs`, `src/view_state.rs`,
  `docs/plan/command-inventory.md`, `docs/plan/fragility-register.md`, `docs/plan/progress.md`,
  `docs/plan/screens/bookmarks.md`, `docs/plan/workflows/refs-and-workspaces.md`, and
  `docs/process-observations.md`.

- Behavior: the bookmarks view now exposes preview-first bookmark forget through `bf`. The action
  starts only from typed Packet 36 bookmark metadata. Local tracked rows and local rows with an
  untracked remote peer open a local forget preview. Remote-only rows open a remote forget preview
  only when the selected row metadata and same-name sibling scan prove exactly one remote peer and
  no local peer.

- Disabled states: local-only, ambiguous, unknown, remote-with-local-peer,
  remote-local-peer-unknown, and non-unique same-name remote rows stay in normal mode and report a
  specific status message. The flow never infers target state from rendered labels and never targets
  `name@remote`.

- Command shape: local forget uses `jj bookmark forget exact:"<name>"`. Remote-only forget uses
  `jj bookmark forget --include-remotes exact:"<name>"` only after metadata loaded from an
  unfiltered all-remotes view proves that the base name has exactly one remote peer and no local
  peer. Filtered views such as `--all-remotes --remote origin`, `--tracked`, `--conflicted`,
  `-r <revset>`, or positional/name filters are classified as visible-only and fail closed for
  remote-only forget. Exact string-pattern escaping is shared with bookmark move/delete and covered
  for quote/backslash names.

- Output behavior: preview names the exact bookmark, visible tracking/remote state, command, effect,
  confirmation, and recovery/review path. Confirmation preserves success and failure output in
  `ActionOutput`; cancel returns to normal mode with `bookmark forget cancelled`.

- Proof: disposable repo `/tmp/jk-p38-proof` with remote `/tmp/jk-p38-remote.git` verified local
  tracked forget, remote-only include-remotes forget, and undo. Commands were run with cwd set to
  `/tmp/jk-p38-proof`: `jj --no-pager bookmark forget 'exact:"feature/name"'` removed the local
  tracked row and left `feature/name@origin`; `jj --no-pager undo` restored the local tracked row;
  then `jj --no-pager bookmark forget --include-remotes 'exact:"feature/name"'` removed the
  remote-only row and `jj --no-pager undo` restored `feature/name@origin`.

- Validation: `cargo check`; `cargo test bookmark_forget -- --test-threads=1`;
  `cargo test bookmark -- --test-threads=1`; `cargo test jj_rows -- --test-threads=1`; full
  `cargo test` passed with 470 passed / 2 ignored; `cargo clippy -- -D warnings`;
  `rustup run nightly cargo fmt --check`; `just md-check`; `just check`; disposable `/tmp` jj proof
  above.

- Repair note: this packet repaired a partial non-compiling `@` state from an earlier worker. The
  missing `open_bookmark_forget_preview` entrypoint and exhaustive `JjBookmarkMutationKind::Forget`
  prompt match were added while preserving the partial command-builder work that already produced
  exact local and remote-only argv shapes. A later 5.5 review found that `--all-remotes` was treated
  as globally complete even when additional bookmark-list filters hid same-name peers; this repair
  now treats only bare `--all-remotes`/`-a` as complete enough for remote-only exactness.

- Remaining risk: remote-only forget is exact only when the bookmark view was loaded from an
  unfiltered all-remotes metadata stream with matching rendered rows. Filtered bookmark views,
  row/metadata drift, and unknown peer metadata fail closed instead of guessing. Track/untrack
  remain non-goals and disabled.

- Next recommended slice: review Packet 38 for metadata-gated target exactness, delete-versus-forget
  clarity, command construction, disabled states, output preservation, docs/fragility updates,
  isolated `/tmp` proof coverage, and compile/test health restoration.

## Packet 38 UI/Keybinding Follow-Up

- Status: planned follow-up work, not shipped behavior.
- Scope: after Packet 38 and before broad Packet 39+ work, address the current log-screen and
  keybinding bugs: visible space selection on the log, clearer current-row highlighting, PageUp and
  PageDown scrolling, help-popup arrow-key handling, two-column help, shifted-capital handling for
  `S` and related keys, status-bar shortcut prioritization, command-menu background/key-label
  readability, and next-key discovery for multi-key prefixes like `g`.
- Validation expectation: use focused view-level tests and rendering snapshots for the log, help,
  status, and command-menu surfaces, plus keybinding dispatch tests for shifted capitals and prefix
  hints when the implementation lands.
- Next turn: promote this follow-up before Packet 39 or broader bookmark-tracking work.

## App Refactor Audit Follow-Up

- Files changed: `docs/plan/progress.md`, `docs/process-observations.md`
- Audit summary: a gpt-5.5 high read-only audit found no blocking refactor slice. `src/app.rs` is
  now 511 LOC and owns only app orchestration, key dispatch, and `ViewEffect` routing. The remaining
  app module sizes are acceptable because ownership is coherent, with watch items at
  `src/app/action_lifecycle/preview.rs` about 694 LOC, `src/app/mode_input.rs` about 695 LOC, and
  `src/app/tests/support.rs` about 550 LOC.
- Target bands: `src/app.rs` is healthy in the 450-650 LOC range, should be reviewed around 750-800
  LOC, and is suspect at 900-1000 LOC. Extracted dispatch/lifecycle modules can be 600-750 LOC only
  when they have a clear owner.
- Next potential refactor: if `src/app/action_lifecycle/preview.rs` grows, the next candidate is
  naming and ownership around the immediate action paths rather than another broad app split.
- Validation: `just md-check`
