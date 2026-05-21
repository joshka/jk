# Source Maintainability Ledger

This ledger records the current bounded follow-up packets after the completed bookmark and rewrite
refactoring slices. It is not a standing "split the biggest files" queue. Reassess with fresh
measurements before starting another source-shape packet.

Current evidence comes from the 2026-05-21 reassessment packet: `just largest-rust-files`, cheap
visibility and control-flow scans, and direct reads of the cited source files together with
[`architecture.md`](architecture.md) and [`rust-style.md`](rust-style.md).

## Quality Bar

- Favor reader locality and low cognitive burden over generic file splitting.
- Keep ownership vertical: move rules, data, and wording toward the concept that changes for the
  same reason.
- Preserve rendered `jj` output, argv shape, labels, refresh behavior, selection behavior, and
  app-level routing unless the slice explicitly owns that contract.
- Use docs and tests as proof of ownership. Each packet should say what moved, what did not move,
  and what focused validation preserved the contract.

## Reassessment Snapshot

### Recent Completed Slices

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

Those packets closed the previous bookmark- and rewrite-heavy queue. The next work should start from
the current hotspots instead of replaying the old priority order.

### Current Size Snapshot

Current largest production files from `just largest-rust-files` and a direct source scan:

```text
2056 src/jj_actions.rs
1440 src/jj.rs
1299 src/jj_rows.rs
1255 src/command.rs
1246 src/action_menu.rs
1218 src/graph.rs
1134 src/tui.rs
 973 src/bookmarks.rs
 876 src/jj_rows/bookmarks.rs
 833 src/jj_actions/bookmarks.rs
```

Large files alone do not justify a packet. They only nominate surfaces to inspect for mixed concepts
and high live context.

### Current Visibility Snapshot

Cheap `rg`-style counts found these visibility totals:

- unrestricted `pub` lines: 768
- restricted visibility lines (`pub(crate)`, `pub(super)`, `pub(in ...)`): 393

Top production files by visibility count:

- `src/jj_actions.rs`: 152 unrestricted `pub`
- `src/jj_rows.rs`: 51 unrestricted `pub`
- `src/action_menu.rs`: 46 unrestricted `pub`
- `src/sticky_file_view.rs`: 44 unrestricted `pub`
- `src/command.rs`: 39 unrestricted `pub`
- `src/jj.rs`: 37 unrestricted `pub`

These counts are measurement only. They are useful when they line up with a concept boundary, not as
a goal by themselves.

### Current Control-Flow Snapshot

Cheap token scans over production files found these current hotspots:

- `src/jj_actions.rs`: 62
- `src/app/mode_input.rs`: 58
- `src/command.rs`: 55
- `src/jj.rs`: 50
- `src/action_menu.rs`: 43
- `src/app/action_lifecycle/completion.rs`: 34
- `src/jj_rows.rs`: 30
- `src/app.rs`: 30
- `src/tui.rs`: 29

The current next slices come from where these counts overlap with mixed ownership during direct
reads. `src/graph.rs`, `src/jj.rs`, and `src/tui.rs` are still large, but the current read did not
show a sharper bounded packet there than the ones below.

## Current Next Slices

### 1. Help Projection Policy Packet

- Owner: `src/command.rs`
- Purpose: isolate the context-conditioned help policy currently concentrated in `help_metadata`,
  `view_help_metadata`, `collect_help_rows`, and `project_help`. This is a sharp vertical slice
  because it changes when bindings are documented, not when key matching or dispatch changes.
- Non-goals: no key binding changes, no dispatch changes, no `ViewEffect` changes, and no TUI help
  layout redesign in `src/tui.rs`.
- Proof: focused command/help projection tests, especially context-specific visibility cases, plus
  `cargo check` and `just md-check`.

### 2. File And Status Path Action-Menu Policy

- Owner: `src/action_menu.rs`
- Purpose: isolate the path-scoped action-menu rules currently mixed across `ExactActionContext`,
  `FileActionContext`, `status_path_action_menu`, `file_action_menu_items`, and the path
  restore/track/untrack/chmod follow-ups. This packet is narrower than a whole-file split and
  matches one user-visible concept: path actions vary by working-copy versus exact revision scope.
- Non-goals: no graph multi-revision menu redesign, no role-prompt redesign, no mutation preview
  execution changes, and no new action vocabulary.
- Proof: focused action-menu tests for status paths, file lists, and file-show surfaces, plus
  `cargo check` and `just md-check`.

## Not The Next Packet

- `src/jj.rs`: still large, but the current read is mostly one cohesive owner around `ViewSpec`,
  diff-format handling, process helpers, and navigation provenance. Wait for a narrower contract
  break before slicing it again.
- `src/graph.rs`: still large, but the current remaining density is mostly one view contract:
  bindings, selection, search, refresh, and action-menu opening. The previous selection helper work
  already removed the clearest cross-view packet.
- `src/tui.rs`: still large, but current density comes from shared overlay rendering. A safe slice
  needs a concrete overlay owner such as help projection or action-output chrome, not size alone.
