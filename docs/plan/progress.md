# Slice Progress

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
