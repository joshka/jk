# File Screens

## Purpose

File-oriented screens support inspection and targeted file actions around a revision or working-copy
state.

## Source Commands

- `jj file list`
- `jj file show`
- `jj file search`
- `jj file annotate`
- related guided flows: `track`, `untrack`, `chmod`

## Screen Family

This is a family rather than one screen:

- file list
- file show
- file search
- file annotate

## View Model

- file list should stay path-first
- file show and annotate should be dedicated detail screens
- inline file preview under file list may be worth exploring later

## Priority

Priority 2. File screens should arrive after the core show/diff/status surfaces, because they depend
on exact path identity and benefit from established document navigation behavior.

## Primary Interactions

- navigate file list
- open selected file into file show
- search within file-oriented screens where useful
- copy file path
- launch file actions when the current context supports them

## Entry Points

- from show, diff, or status when an exact path is available
- direct startup via `jk file list` or `jk file show`
- from file list into file show on the selected exact path

## Selection Model

- file list selection unit: exact path item
- file show selection unit: scroll offset in one file
- annotate selection unit: line or hunk, if annotate becomes native
- file actions require exact path identity and the revision or working-copy context that owns it

## Interaction Details

- File list: list paths for a revision, diff, status group, or working-copy context. Movement is by
  path item.
- File show: open one path in a dedicated document view with search, copy path, and back.
- Working-copy actions: exact working-copy file list/show paths offer guided untrack and chmod
  previews. Untrack previews state that jj requires the path to already be ignored.
- Exact revision actions: graph-derived file list/show paths offer exact-revision chmod previews
  using the selected change id. Direct arbitrary revsets do not enable chmod.
- File search: if native, search should be scoped to a revision or path set and return a list of
  matches that can open file show at a line.
- Annotate: later screen for line provenance; should use semantic line/revision data before it
  becomes native.
- Actions: track is shipped from status `?` rows; untrack, chmod, and restore-like actions launch
  guided flows with previews when exact path and revision or working-copy ownership are known.
- Refresh: preserve selected path when possible.

## Shortcut Candidates

- `j`/`k`, arrows: move path or scroll file
- `Enter`, `l`, `Right`: open selected file
- `/`, `n`, `N`: search
- `y`: copy path or selected text
- `a`: action menu where valid
- `t`: track flow in status action menu where valid
- `u`: untrack flow in action menu where valid
- `x`/`n`: chmod executable/normal in action menu where valid
- `r`: refresh
- `h`, `Left`: back

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
