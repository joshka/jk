# Show Screen

## Purpose

The show screen is the detailed inspection surface for one selected change.

## Source Command

- `jj show`

## View Model

- single active document surface
- compact commit context pinned above file content
- sticky file heading projection as the user scrolls

This screen should not require a sibling preview pane. The document itself should carry enough
context.

## Priority

Priority 0. Show is the first detail screen from log and sets the pattern for document scrolling,
sticky context, search, copy, refresh, and switching between related inspection views.

## Core Information

- commit metadata and subject
- body text
- file-level change sections
- rendered `jj` styles and wording

## Primary Interactions

- scroll
- page up/down
- jump to top/bottom
- next file / previous file
- search and search-next/search-previous
- copy available identifiers or file labels
- switch to diff for the same target
- go back
- refresh in place

## Interaction Details

- Scrolling: movement is line-oriented within the document, with page movement preserving enough
  context that the pinned header does not feel like a jump.
- File navigation: next/previous file jumps to semantic or rendered file boundaries. If heading
  detection fails, the document remains scrollable and the file-jump action should be unavailable.
- Sticky context: pinned commit and file context should orient the user without becoming a split
  pane. It should be derived from the document or a stronger contract, not regenerated into a
  different layout.
- Search: search highlights rendered document text while preserving source styles. Search should
  work across commit metadata, body text, file headings, and patch-like sections.
- Copy: copy should expose target revision identity, active file label, and selected rendered text
  when supported.
- Switching: switching to diff should preserve the target revision and, when possible, the active
  file or nearest file context.
- Refresh: refresh should preserve target revision and approximate scroll/file context. If the
  target is no longer available, report the command error and keep the prior view state readable.

## Shortcut Candidates

- `j`/`k`, arrows: scroll
- `Space`, `Ctrl-f`: page down
- `Shift-Space`, `Ctrl-b`: page up
- `g`/`G`: top/bottom
- `[`/`]`: previous/next file
- `/`, `n`, `N`: search
- `d`: switch to diff
- `y`: copy menu
- `l`, `Right`: open file list
- `h`, `Left`: back
- `r`: refresh

## Entry Points

- from log
- direct startup via `jk show`
- from diff

## Selection Model

- selection unit: document scroll position
- active file context is derived from scroll position

## Integration Notes

Use rendered `jj show` output as the document. Sticky file context may parse headings, but body
text, metadata, and diff content should remain opaque unless a specific interaction needs structure.
If additional actions need exact file sets or template semantics, record the soft agreement and
consider a purpose-built template, structured output, or `jj_cli`/`jj_lib`.

The preferred stronger contract would expose commit metadata, body, file sections, file identities,
and renderable styled spans together. That would let `jk` preserve jj-like output while navigating
files and copying identifiers without parsing prose.

## Empty, Error, And Edge States

- no files changed
- unusual rendered headings
- long body text before file sections
- refresh after content shape changes

## Hybrid Information Policy

The show screen can be richer than raw `jj show` in one important sense: it may preserve or pin
context that helps navigation. It should not invent a second repository model or rewrite the content
into a custom prose layout.

## Acceptance Criteria

- preserves `jj` fidelity
- file-to-file navigation feels natural
- pinned context improves orientation without feeling like a pane
