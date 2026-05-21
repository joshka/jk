# Source Cleanup Audit

This audit records mechanical measurements for the maintainability cleanup wave. Measurements are
evidence for choosing what to read next; they are not a refactoring order by themselves. A packet
should still prove that the chosen move lowers reader burden, keeps behavior local to the owning
concept, and preserves rendered `jj` behavior.

Captured: `2026-05-21 16:21:25 PDT`.

## Current Measurements

- Rust source total from `find src -name '*.rs' -maxdepth 4 -print0 | xargs -0 wc -l`: `34,452`
  lines.
- Visibility entries from `rg "^pub(\(| |$)|pub\(crate\)|pub\(super\)" src -n | wc -l`: `471`.
- Inline `#[cfg(test)] mod tests { ... }` blocks from `rg -U`: `19`.
- Match expressions in `src/app/mode_input.rs`, `src/app.rs`, and `src/app/action_lifecycle/*` from
  a simple `rg "match .*\{|match$"` count: `108`.

## Largest Files

The largest files are currently dominated by app tests and feature/action modules. The list below is
a prompt for reading, not a command to split every file.

```text
1196 src/app/tests/bookmark_actions.rs
 778 src/app/tests/working_copy_actions.rs
 648 src/app/mode_input.rs
 643 src/bookmarks/tests.rs
 613 src/graph/tests.rs
 605 src/graph.rs
 596 src/app/tests/command_navigation.rs
 592 src/tui.rs
 588 src/bookmarks/actions.rs
 585 src/app/tests/support.rs
 583 src/status.rs
 575 src/operation_log.rs
 571 src/app/action_lifecycle/entry.rs
 569 src/app.rs
 568 src/sticky_file_view.rs
 564 src/bookmarks/action_targets.rs
 562 src/interactive_process.rs
 553 src/command.rs
 541 src/app/action_lifecycle/preview.rs
 535 src/app/tests/detail_restore_actions.rs
 531 src/app/services.rs
 529 src/app/tests/rewrite_actions.rs
 528 src/view_state.rs
 498 src/graph/rows.rs
```

## Inline Test Blocks

Remaining inline test blocks fall into two groups:

- Small shared helpers where inline tests may still be fine: `search`, `selection`, `theme`,
  `jj_syntax`, `files/list/rows`, `resolve/rows`, `tui/status_hints`.
- Larger feature or shared modules worth reading before deciding: `operation_log`,
  `operation_log/rows`, `operation_log/actions`, `workspaces/rows`, `graph/rows`, `action_output`,
  `view_state`, `interactive_process`, `action_menu/path_actions`, `jj_actions/git_sync`, and
  `jj_actions/rewrite`.

Do not move a test block only because it appears in this list. Move it when the production module is
harder to scan and the sibling test module would still preserve reader locality.

## Candidate Next Targets

- `src/app/mode_input.rs`: after menu, text-prompt, and abandon extraction, the dispatch table is
  mostly named modal handlers. Further work should be based on reading the remaining helper order
  and tests, not on line count alone.
- `src/app/action_lifecycle/entry.rs` and `src/app/action_lifecycle/preview.rs`: read for repeated
  preview/completion setup before extracting any helper. Preserve status wording, output panes,
  refresh/reveal behavior, and command execution contracts.
- `src/status.rs` and `src/operation_log.rs`: feature-view modules still carry view behavior and
  tests inline. Any split should keep row model, action availability, copy behavior, and tests easy
  to find from the feature root.
- `src/graph.rs` and `src/graph/rows.rs`: graph selection and rendered-row assumptions are central
  product behavior. Prefer contract comments and focused tests before moving code.
- `src/tui.rs`: shared chrome already has useful ownership docs. Future cleanup should avoid turning
  `tui` into a feature-policy bucket.

## Process Guidance

- Start with a fresh measurement before a major source-shape packet.
- Read the owning module and nearby tests before choosing a move.
- Prefer documentation contracts when the module is readable but ownership assumptions are implicit.
- Prefer extraction only when a name removes live context or puts behavior with the product concept
  that owns it.
- Keep behavior-preserving validation specific to the touched surface, then run `cargo check` and
  formatting or Markdown checks as appropriate.
