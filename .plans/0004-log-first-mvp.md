# Log-First MVP

## Why

The reset exists to avoid reviewing a broad prototype all at once. The first useful release should
be a terminal window that can sit beside an editor or coding agent and keep the `jj` log current.
Manual refresh is enough for the first cut; auto-refresh is the first upgrade once the view is
stable.

## Scope

Build only enough TUI to support the first log-focused loop:

1. Start in a full-screen log view.
1. Refresh manually with one key.
1. Auto-refresh when repository state changes, once manual refresh is solid.
1. Move selection by change/log item with vimish navigation.
1. Expand the selected item inline to show details such as the full commit message.
1. Preserve selection and scroll position across refresh when the selected change still exists.

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

## Done When

- The app opens a log-first TUI and refreshes in place with one key.
- Vimish movement works for the visible log list.
- The selected item can expand inline to show the full commit message.
- Auto-refresh has a tested debounce or event-coalescing policy, or is explicitly deferred behind
  manual refresh.
- Diff refresh and collapsible diff sections are documented as the next slice if not included.
- Tests cover state transitions, refresh behavior, selection preservation, and rendered output.
- Any unavoidable `jj` output parsing is narrow, named, and documented as temporary.
