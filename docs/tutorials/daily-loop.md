# Daily Loop

## Prerequisites

- A `jj` repository.
- For a reproducible history walk, run `just demo-setup` and `cd target/demo-repos/static-log`.

## Read History

1. Start `jk` in the repository.
2. Stay in the log view to scan history.
3. Use `j` and `k` to move.
4. Open `show` with `s`, `l`, or `Right`.
5. Open `diff` with `d`.
6. Use `[` and `]` in `show` or `diff` to move between files.
7. Use `l` in `show` or `diff` to open the file list.
8. Return with `h` or `Left`.
9. Press `Esc` to leave `jk` from normal mode.

The log, show, diff, and status surfaces are inspection-first. In some contexts, action menus expose
previewed mutation paths, so check help/previews before confirming.

## Check Status

- Press `S` to open status.
- Press `r` after changing files in another terminal.
- Press `?` whenever you need the exact keys for the current screen.

## Fetch And Push

- Press `f` for `jj git fetch`. From the graph view, `gf` is also available; other views keep `g` as
  immediate top navigation.
- Press `p` for the guided push flow.
- Fetch is direct and low risk.
- Push is previewed before it runs, and the preview makes the target choice explicit when `jj` will
  pick it at execution time.

## Create New Work

- Press `c` to create a new working-copy change from trunk.
- Use this when you want a fresh editable change without leaving `jk`.
- If the new change was not what you wanted, `jj undo` is the recovery path.

## Utility Views

- Press `B` for bookmarks, `R` for resolve, `X` for workspaces, and `O` for operation log.
- These views stay focused: they expose search, copy, and exact-target actions where the current
  metadata is strong enough, but they do not turn into broad dashboards.

## When To Use Help

- Press `?` whenever the screen changes or you forget the current bindings.
- The generated command menu is the quickest way to confirm and run the current key surface without
  reading the source.
