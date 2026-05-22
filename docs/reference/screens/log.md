# Log Screen

## Purpose

The log screen is the home surface of `jk`. It should make stack inspection, selection, and drill
down cheaper than repeated `jj log`, `jj show`, and `jj diff` shell loops.

## Source Commands

- `jj`
- `jj log`

## View Model

- single active list surface
- inline expansion of the selected item
- no permanent split-pane dashboard

## Priority

Priority 0. The log screen is the foundation for navigation, read drill-down, refresh behavior,
search, copy, and graph-attached mutation flows.

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
- expand/collapse detail for selected item
- switch between default, trunk-focused, recent, all/repo, and custom revset views
- launch low-friction `jj new trunk` when trunk is known exactly
- open the graph action menu from selected items

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
- Action selection: one selected row is enough for navigation and simple actions. Log screen action
  flows use exact revision id multiselect controlled by `Space` for graph-style guided operations
  (`a` menu, with rebase/squash/abandon-style actions).
- View mode: default view should stay close to built-in `jj` output for current work. Broader view
  modes should be easy to reach when the user needs repository orientation.
- New trunk: `jj new trunk` is a low-friction direct action when trunk is known exactly. The
  post-action refresh should make the new working-copy change visible, with `jj undo` as the
  recovery path.
- Refresh: refresh should preserve selection by stable change id or equivalent semantic identity
  when possible. The log now stores and reuses exact change ids, prunes disappeared selections on
  refresh or mode change, and clamps the cursor to the nearest row when needed. Explain any missing
  selection target in status chrome rather than mutating selection silently.

## Shipped Bindings

- `j`/`k`, arrows, `PageUp`, `PageDown`: move selection by revision item
- `g`/`G`, `Home`, `End`: first or last selectable item
- `l`, `s`, `Right`: open show
- `d`: open diff
- `w`: cycle log view mode
- `c`: run `jj new trunk` when trunk is known exactly
- `e`, `]`, `[`: edit current, next, or previous working-copy change
- `Space`: toggle exact-id row selection for action flows
- `a`: open the log action menu for the selected revision set
- `g f`, `g p`, `g r`: fetch, push, or fetch a named remote through global dispatch
- search, copy, refresh, and help are provided by the shared app bindings

## Selection Model

- selection unit: revision item
- log selection is change-id exact, not row-index based
- multi-select is available on log only
- paging should target item boundaries, not raw rendered lines
- refresh should preserve selection when the row can still be identified honestly using exact change
  ids
- action selection supports one or more revision items for guided flows such as `new`, `rebase`,
  `squash`, or `abandon`
- selected rows are visibly marked in the list so users can tell queued action scope at a glance

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
relationship information through the flow. Action flows should use preview-first menus and explicit
role or prompt preparation where the operation is risky or ambiguous.

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

## Non-Goals

- permanent side pane
- broad mutation launcher
- edit-in-place history surgery from the log row itself

## Acceptance Criteria

- feels like the natural home of the app
- list scanning is fast
- drill-down is predictable
- expanded detail adds meaning without turning the screen into a dashboard
