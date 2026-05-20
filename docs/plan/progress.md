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
- Next slice: TBD after review of exact-target mutation flows

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
- Next slice: TBD after review of exact-target mutation flows
