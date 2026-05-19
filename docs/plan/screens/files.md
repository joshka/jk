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

- file list should remain list-first
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

## Selection Model

- file list selection unit: exact path item
- file show selection unit: scroll offset in one file
- annotate selection unit: line or hunk, if annotate becomes native
- file actions require exact path identity and the revision or working-copy context that owns it

## Interaction Details

- File list: list paths for a revision, diff, status group, or working-copy context. Movement is by
  path item.
- File show: open one path in a dedicated document view with search, copy path, and back.
- File search: if native, search should be scoped to a revision or path set and return a list of
  matches that can open file show at a line.
- Annotate: later screen for line provenance; should use semantic line/revision data before it
  becomes native.
- Actions: track, untrack, chmod, restore-like actions should launch guided flows with previews when
  they can change state.
- Refresh: preserve selected path when possible.

## Shortcut Candidates

- `j`/`k`, arrows: move path or scroll file
- `Enter`: open selected file
- `/`, `n`, `N`: search
- `y`: copy path or selected text
- `t`: track/untrack flow where valid
- `x`: chmod flow where valid
- `r`: refresh
- `h`, `Esc`: back

## Integration Notes

File inspection can start from rendered output. Track, untrack, chmod, and restore-like actions
should not rely on loosely parsed file labels when exact paths or fileset behavior matter. Prefer
structured output, narrow templates, or `jj_lib` before mutation flows expand.

The preferred contract exposes exact path, file status, revision context, display label, and
renderable styled row or document spans together.

## Acceptance Criteria

- file inspection complements show/diff instead of duplicating them
- file actions attach to meaningful context
- mutation-capable file actions use exact path and fileset semantics
