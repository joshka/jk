# Log Screen

## Purpose

The log screen is the home surface of `jk`. It should make stack inspection, selection, and drill
down cheaper than repeated `jj log`, `jj show`, and `jj diff` shell loops.

## Source Commands

- `jj`
- `jj log`

## View Model

- primary mode: single active list surface
- preferred enrichment: inline expansion of the selected item
- possible later variant: optional under-preview or side-preview mode, local to this screen only

The default should not be a permanent split-pane dashboard.

## Priority

Priority 0. The log screen is the foundation for navigation, read drill-down, refresh behavior,
search, copy, and later graph-attached mutation flows.

## Always-Visible Information

- graph glyphs and structure
- change id signal
- subject line
- enough metadata to orient the user quickly

## Expandable Information

When expanded inline, the selected entry may reveal:

- body text
- file list or file headings
- commit id
- bookmark/tag indicators
- divergence or working-copy signal

The expansion should still feel like one list, not a second screen crammed into the first.

## Entry Points

- startup default
- explicit `log` startup
- default work view from anywhere
- broader revset/view-mode switch from the log screen
- back to home actions
- dedicated log shortcut

## Primary Interactions

- move selection by logical revision item
- jump to first/last item
- search and search-next/search-previous
- open `show`
- open `diff`
- copy change id and commit id
- refresh in place
- optional expand/collapse detail for selected item
- switch between default, trunk-focused, recent, all/repo, and custom revset views
- launch low-friction `jj new trunk` when trunk is known exactly

## Interaction Details

- Movement: `j`/`k` and arrows move by revision item, not by visual line. Elided graph rows and
  decoration should not steal selection.
- Paging: page movement should land on revision items and clamp at the first or last selectable
  item.
- Search: search should match visible rendered text first. When a row has semantic fields or
  expanded text, search should be able to highlight the matching renderable range without changing
  the row identity.
- Copy: copy actions should expose change id, commit id, bookmark/tag names, and file labels when
  present. Copy availability should follow row semantics, not brittle text slicing.
- Expansion: expanded rows replace the compact row inline. Expansion should preserve list position,
  selected revision identity, search highlights, and graph context.
- Action selection: one selected row is enough for navigation and simple actions. Multi-row
  selection is needed before graph-attached flows such as rebase, squash, abandon, and future batch
  operations feel native.
- View mode: default view should stay close to built-in `jj` output for current work. Broader view
  modes should be easy to reach when the user needs repository orientation.
- New trunk: `jj new trunk` should be a low-friction direct action when trunk is known exactly. The
  post-action refresh should make the new working-copy change visible, with `jj undo` as the
  recovery path.
- Refresh: refresh should preserve selection by stable change id or equivalent semantic identity
  when possible. If the item disappears, clamp to the nearest reasonable row and explain errors in
  status chrome rather than mutating selection silently.

## Shortcut Candidates

- `j`/`k`, arrows: move selection
- `g`/`G`: first/last selectable item
- `/`, `n`, `N`: search
- `Enter`, `l`, `s`: open show
- `d`: open diff
- `Space`: toggle row selection for action flows once multi-select exists
- `x`: expand/collapse selected row
- `w`: switch work/revset view mode
- `c`: create a new change from trunk or selected context, if this remains mnemonic after key review
- `y`: copy menu
- `r`: refresh
- `?`: help/keymap

Shortcut names are candidates, not commitments. They should be finalized after command ownership is
clear.

## Selection Model

- selection unit: revision item
- paging should target item boundaries, not raw rendered lines
- refresh should preserve selection when the row can still be identified honestly
- action selection should support one or more revision items for guided flows such as `new`,
  `rebase`, `squash`, or `abandon`

## Row Contract

The ideal log row provides both semantic fields and renderable view pieces:

- change id for navigation and copy actions
- commit id for copy actions and detail display
- stable action identity for single-row and multi-row guided flows
- graph role and glyph information for stable item selection
- parent/child or adjacency information when an action depends on graph relationships
- labels, bookmarks, tags, and working-copy signals where jj would show them
- styled text spans that preserve the user's configured template and colors
- searchable text ranges that can be highlighted without rewriting the row

Inline expansion should be able to replace a compact row with richer detail for the same revision:

- long description or body text
- file list or file headings
- full or shortened commit identity
- additional metadata that jj would normally expose through `show`, `diff`, or templates

This row contract should avoid forcing `jk` to parse terminal text for semantic meaning. If the
contract is not available, the fallback parser should stay narrow and the missing contract should be
recorded as integration evidence.

Guided mutation flows should not infer their inputs from visual row text. Selecting a row for `new`,
`rebase`, or another action should carry the exact revision identity and any required graph
relationship information through the flow.

## Integration Notes

Use rendered `jj log` output as the baseline. Parse only enough row structure to identify selectable
revision items and navigation targets. If inline expansion needs full commit bodies, file lists, or
metadata that is not already reliable in the rendered row, prefer `jj show`, a narrow template, or a
stronger API over expanding the graph parser into a repository model.

The long-term preferred contract is a row model that carries:

- semantic ids and graph relationships;
- renderable styled spans from the user's configured template;
- searchable ranges;
- optional detail hooks for show/diff/file-list expansion;
- action-selection identity for guided flows.

## View Modes

The log screen should support multiple scopes:

- Default work: the normal home view, close to `jj`'s built-in default.
- Trunk work: focused on work from `trunk` or the configured main branch.
- Recent work: changes worked on recently across branches or repos.
- All/repo overview: broader context when the default view is too narrow.
- Custom revset: user-supplied revset, retained where practical.

Switching modes should preserve selected change id when possible and should make the active mode
visible in title/status chrome.

## Empty, Error, And Edge States

- empty repository or trivial history
- elided revisions
- non-selectable graph rows
- graph rows whose metadata cannot be parsed safely

If parsing fails, the row should remain visible and navigation should degrade honestly.

## Out Of Scope For First Wave

- permanent side pane
- broad mutation launcher
- edit-in-place history surgery from the log row itself

## Acceptance Criteria

- feels like the natural home of the app
- list scanning is fast
- drill-down is predictable
- expanded detail adds meaning without turning the screen into a dashboard
