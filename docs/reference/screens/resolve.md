# Resolve Screen

## Purpose

The resolve screen is the conflict-oriented utility surface.

It is currently read-only. It helps users inspect conflicted paths without launching merge tools,
mutating files, or pretending to own `jj`'s conflict semantics.

## Source Command

- Primary list contract:

  ```sh
  jj log --no-graph -r @ --color=never -T 'self.conflicted_files().map(|entry| "{\"path\":"
  ++ json(entry.path()) ++ ",\"file_type\":" ++ json(entry.file_type()) ++ ",\"side_count\":"
  ++ json(entry.conflict_side_count()) ++ "}\n").join("")'
  ```

- Inspect selected path:

  ```sh
  jj file show -r <resolve-target-or-@> <path>
  ```

The screen does not shell out to `jj resolve --list` because clean repos need to open as an empty
list instead of as a command failure.

## View Model

- list-first surface
- path-first selectable items
- empty clean repos render as `0 conflicts`
- detail inspection reuses `jj file show`

## Priority

Priority 2. Resolve should follow status and file screens, because it needs exact conflict and path
semantics before guided resolution can be safe.

## Core Information

- conflicted paths
- `file_type` from `self.conflicted_files()`
- `side_count` from `self.conflicted_files()`
- enough context to inspect the conflict target quickly

## Primary Interactions

- move between conflict items
- inspect the selected conflict path with `jj file show`
- refresh
- go back

## Selection Model

- selection unit: conflict item or conflicted path
- refresh preserves selection by exact conflicted path when possible
- multi-select is not a first-wave need; resolve flows should start with one conflict at a time

## Interaction Details

- Movement: move by conflict item.
- Inspect: `Enter`, `l`, and `Right` open `jj file show -r <resolve-target-or-@> <path>` when the
  selected row carries an exact path.
- Unknown path: rows with malformed or partial metadata stay readable and copyable, but inspect
  shows a clear status error instead of inventing a path.
- Refresh: preserve selected conflicted path when possible.
- Copy: offer the exact path when known and always offer the displayed row text.
- Search: search wraps by conflict item, not by individual line.

## Bindings

- `j`/`k`, arrows: move conflict selection
- `Enter`, `l`, `Right`: inspect selected conflict path
- `n` / `N`: next or previous search match
- `R`: open resolve from other views
- `y`: copy path
- `r`: refresh
- `h`, `Left`: back

## Integration Notes

Conflict state is too semantic to duplicate casually. The current screen uses a narrow
machine-oriented template contract for the list and leaves rendered `jj file show` output as the
inspection surface. Guided resolution actions should move to stronger `jj` APIs or structured
contracts before `jk` tries to mutate conflicted files.

This screen explicitly does not:

- launch `jj resolve <path>`;
- open an external merge tool;
- mark conflicts resolved;
- offer `:ours`, `:theirs`, or automatic resolution;
- infer exact paths from rendered sticky headings or prose;
- turn conflict rows into mutation-capable filesets.

## Acceptance Criteria

- conflict state is visible as a focused list of conflicted paths
- clean repos open as an empty list instead of a failure state
- inspection stays read-only and path-exact when possible
- guided resolution stays out of scope until the contract is strong enough for safe mutation
