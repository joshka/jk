# Log-First MVP

## Why

The reset exists to avoid reviewing a broad prototype all at once. The first useful release should
be a terminal window that can sit beside an editor or coding agent and keep the `jj` log current.
Manual refresh is enough for the first cut. Auto-refresh is explicitly deferred until the manual
log view and the selected-change inspection loop are stable enough to justify a file-watching
policy.

## Scope

Build only enough TUI to support the first log-focused loop:

1. Start in a full-screen log view.
1. Refresh manually with one key.
1. Move selection by change/log item with vimish navigation.
1. Expand the selected item inline to show details such as the full commit message.
1. Preserve selection and scroll position across refresh when the selected change still exists.

The first MVP should intentionally stop at manual refresh. Auto-refresh should be added only after
there is a tested debounce or event-coalescing policy for repository writes.

The log view should preserve the user's normal `jj` expectations:

1. Running `jk` with no subcommand follows `jj`'s configured default command.
1. Running `jk log` opens the log view explicitly.
1. Rendered log output keeps the same template, graph, revset, and colors that `jj` would show.
1. The TUI does not frame the log in borders; title and status bars are allowed because they explain
   the current view and available commands without changing the log body.

The next slice is the same refresh model for diff inspection:

1. Open a diff for the selected change.
1. Refresh the diff manually and then automatically.
1. Collapse and expand file or hunk sections.
1. Return to the log without losing context.

Avoid mutation commands, command launchers, broad dashboards, and multi-pane policy until the log
and diff inspection loops are reviewed and tested.

## Integration Questions

- Can `jk` get semantic log records and CLI-equivalent rendered output through `jj-cli` / `jj-lib`
  without parsing `jj log` stdout?
- If not, what exact upstream or local contract is missing?
- Which output should stay opaque rendered text, and which fields need structured state for
  navigation?
- What file-watching signal is reliable enough for auto-refresh without thrashing while tools are
  writing the working copy?
- Which vimish commands are mandatory for the first release, and which can wait behind help/keymap
  discovery?

## Temporary Integration Contract

The MVP shells out to `jj` twice:

1. A rendered pass captures the user's configured default command or explicit `jj log` output with
   color forced on, so the log body stays visually equivalent to `jj`.
1. A semantic pass asks `jj log` for one JSON record per change using a narrow template, so the TUI
   can preserve selection, move by change, and expand the full description.

This stdout/template bridge is temporary. Keep it isolated in `jk-cli`, keep the parser fixtures
realistic, and prefer a direct `jj-cli` / `jj-lib` integration once there is a stable way to ask for
both CLI-equivalent rendered output and semantic change records.

Because rendered output and semantic records do not have identical line shapes, selection must map
semantic changes back to rendered commit rows deliberately. Do not assume JSON record order is the
same thing as rendered line number.

## Current State

Completed in the working copy:

- Bare `jk` follows `jj`'s configured default command, with fallback to `jj log`.
- `jk log` opens the explicit log command path.
- The rendered log body keeps `jj` graph/template/color output, including when the parent
  environment sets `NO_COLOR`.
- The view has no log border, but includes a title bar for command context and a status bar for
  available keys.
- Selection uses a full-width gradient row background while preserving the foreground color and text
  modifiers from `jj`.
- `j`/`k`, arrow movement, manual refresh, quit, inline expansion, and collapse are covered by unit
  tests.
- Inline expansion toggles with Enter, Right, and space. Left collapses expanded details. When an
  expanded row moves, expansion follows the new selected row.
- A Betamax tape records initial render, movement, Enter expansion, refresh, and explicit `jk log`.
- Local dogfood installation is documented in `AGENTS.override.md` for this checkout and has been
  verified with `cargo install --path crates/jk`.

Moved to later numbered plans:

- MVP review, split decisions, and hardening live in `0005-log-mvp-review-hardening.md`.
- Selected-change diff inspection and collapsible diff sections live in
  `0006-selected-change-diff.md`.
- Auto-refresh and its debounce/coalescing policy live in `0007-auto-refresh-policy.md`.
- Replacing or narrowing the temporary `jj` stdout/template bridge lives in
  `0008-jj-integration-cleanup.md`.

## Done When

- The app opens a log-first TUI and refreshes in place with one key. Done.
- Vimish movement works for the visible log list. Done.
- The selected item can expand inline to show the full commit message. Done.
- Auto-refresh is explicitly deferred behind manual refresh. Done in
  `0007-auto-refresh-policy.md`.
- Diff refresh and collapsible diff sections are documented as the next slice. Done in
  `0006-selected-change-diff.md`.
- Tests cover state transitions, refresh behavior, selection preservation, and rendered output.
  Done.
- Any unavoidable `jj` output parsing is narrow, named, and documented as temporary. Done for the
  MVP in `jk-cli`, with cleanup tracked in `0008-jj-integration-cleanup.md`.
