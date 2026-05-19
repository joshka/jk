# Diff Screen

## Purpose

The diff screen is the patch-focused inspection surface for one selected target.

## Source Command

- `jj diff`

## View Model

- single active document surface
- sticky file heading projection
- no always-visible secondary pane

## Priority

Priority 0. Diff completes the core inspection loop with patch-focused review, file navigation,
search, copy, diff-format control, and switch-to-show behavior.

## Core Information

- rendered patch text
- file headings
- diff-format-specific output

## Primary Interactions

- scroll
- page up/down
- jump to top/bottom
- next file / previous file
- search and search-next/search-previous
- copy file labels and available identifiers
- switch to show for the same target
- go back
- refresh in place
- toggle diff format if supported by the current app model

## Interaction Details

- Scrolling: line and page movement should keep patch context readable and avoid hiding the active
  file boundary behind sticky chrome.
- File navigation: next/previous file jumps between diff file sections. The action should be
  disabled or degrade gracefully when the output has no recognizable file boundaries.
- Diff format: toggling between default and `--git` diff output reloads the same target and tries to
  preserve active file context. It should not rewrite the patch into an app-specific format.
- Search: search highlights rendered patch text, file headings, and metadata without changing diff
  styles.
- Copy: copy should expose the target revision identity, active file label, and selected text when
  supported.
- Switching: switching to show should preserve target revision and active file context when
  possible.
- Refresh: refresh reloads the same diff command and clamps scroll/file context honestly when output
  changes.

## Shortcut Candidates

- `j`/`k`, arrows: scroll
- `Space`, `Ctrl-f`: page down
- `Shift-Space`, `Ctrl-b`: page up
- `g`/`G`: top/bottom
- `[`/`]`: previous/next file
- `/`, `n`, `N`: search
- `s`: switch to show
- `v`: diff-format menu
- `y`: copy menu
- `h`, `Esc`: back
- `r`: refresh

## Entry Points

- from log
- direct startup via `jk diff`
- from show

## Selection Model

- selection unit: document scroll position
- active file is derived from scroll position

## Integration Notes

Use rendered `jj diff` output as the document. File navigation may depend on heading detection, but
the patch body should remain opaque. Treat default and `--git` heading parsing as soft agreements
and prefer a stronger contract before adding actions that need exact path, mode, or conflict state.

The preferred stronger contract would expose file identities, file status/mode data, diff hunks,
renderable styled spans, and diff-format selection together. Until then, `jk` should not infer
mutation-capable path state from patch text alone.

## Empty, Error, And Edge States

- empty diff
- unusual file heading shapes
- `--git` versus default diff rendering differences

## Acceptance Criteria

- patch review feels cheaper than shelling out repeatedly
- file navigation is obvious
- switching between show and diff preserves orientation
