# Log-First MVP

## Why

The reset exists to avoid reviewing a broad prototype all at once. The first product slice should
prove the core loop: keep a `jj` log-like screen open, refresh it, inspect a selected change, and
return to the log.

## Scope

Build only enough TUI to support:

1. Start in a full-screen log view.
1. Refresh manually with one key.
1. Move selection by change/log item.
1. Open `show` for the selected change.
1. Open `diff` for the selected change.
1. Return to the log.

Avoid mutation commands, command launchers, broad dashboards, and multi-pane policy until the
inspection loop is reviewed and tested.

## Integration Questions

- Can `jk` get semantic log records and CLI-equivalent rendered output through `jj-cli` / `jj-lib`
  without parsing `jj log` stdout?
- If not, what exact upstream or local contract is missing?
- Which output should stay opaque rendered text, and which fields need structured state for
  navigation?

## Done When

- The app opens a log-first TUI and refreshes in place.
- The selected item can open `show` and `diff`.
- Tests cover state transitions and rendered output for the loop.
- Any unavoidable `jj` output parsing is narrow, named, and documented as temporary.
