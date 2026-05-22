# File Screens

## Purpose

File-oriented screens support inspection and targeted file actions around a revision or working-copy
state.

## Source Commands

- shipped screens: `jj file list`, `jj file show`
- related guided flows: `track`, `untrack`, `chmod`
- CLI-first neighbors: `jj file search`, `jj file annotate`

## Screen Family

This is a shipped pair plus CLI-first neighbors:

- file list
- file show
- file search
- file annotate

## View Model

- file list stays path-first
- file show is the dedicated detail screen
- no permanent file preview pane under file list
- annotate or file-search promotion should use the same path-first exact-context rules as file show

## Priority

Priority 2. File screens should arrive after the core show/diff/status surfaces, because they depend
on exact path identity and benefit from established document navigation behavior.

## Primary Interactions

- navigate file list
- open selected file into file show
- search within file show
- copy file path
- launch file actions when the current context supports them

## Entry Points

- from show, diff, or status when an exact path is available
- direct startup via `jk file list` or `jk file show`
- from file list into file show on the selected exact path

## Selection Model

- file list selection unit: exact path item
- file show selection unit: scroll offset in one file
- annotate selection unit: line or hunk if that command is later promoted from CLI-first use
- file actions require exact path identity and the revision or working-copy context that owns it

## Interaction Details

- File list: list paths for a revision, diff, status group, or working-copy context. Movement is by
  path item.
- File show: open one path in a dedicated document view with search, copy path, and back.
- Working-copy actions: exact working-copy file list/show paths offer guided untrack and chmod
  previews. Untrack previews state that jj requires the path to already be ignored.
- Exact revision actions: graph-derived file list/show paths offer exact-revision chmod previews
  using the selected change id. Direct arbitrary revsets do not enable chmod.
- File search and annotate remain CLI-first. Native promotion should require exact path and revision
  context rather than rendered labels.
- Actions: track is shipped from status `?` rows; untrack, chmod, and restore-like actions launch
  guided flows with previews when exact path and revision or working-copy ownership are known.
- Refresh: preserve the selected exact path when possible.

## Non-Goals

- no mutation-capable path inference from rendered labels
- no annotate or file-search screen until the contract is exact enough to keep path and revision
  identity stable
- no broad file dashboard detached from show, diff, or status context

## Shipped Bindings

- file list: `j`/`k`, arrows, `g`/`G`, `Home`, `End` to move by path item
- file list: `Enter`, `l`, `Right` to open the selected file
- file list and file show: `a` to open the action menu when the current context is exact enough
- file show: `j`/`k`, arrows, `PageUp`, `PageDown`, `g`/`G`, `Home`, `End` to scroll
- file show: `z w` toggles wrap; `z h` and `z l` scroll horizontally
- search, copy, refresh, and back come from shared app bindings

## Integration Notes

File inspection can start from rendered output. Shipped track, untrack, chmod, and restore-like
actions do not pass raw paths as filesets; they use exact paths carried by status, file list, or
file show state and construct one `root-file:"..."` fileset argument. Prefer structured output,
narrow templates, or `jj_lib` before mutation flows expand.

The preferred contract exposes exact path, file status, revision context, display label, and
renderable styled row or document spans together. File list should carry the exact path separately
from any display label, and file show should preserve the selected exact path in view state so copy
and refresh stay anchored to the same file.

## Acceptance Criteria

- file inspection complements show/diff instead of duplicating them
- file actions attach to meaningful context
- mutation-capable file actions use exact path and fileset semantics
