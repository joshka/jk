# Bookmarks And Conflicts

## Prerequisites

- A `jj` repository with at least one local bookmark if you want to try bookmark mutations.
- A conflicted repository if you want to try the resolve screen.

## Bookmarks

- Press `B` to open the bookmarks view.
- Use `bc` from graph or status to create a local bookmark. A bare `b` remains a timed fallback for
  now, but generated help shows the explicit multi-key form.
- Use `=` on graph or status to set a bookmark and `m` to move one.
- Use `br` in the bookmarks view to rename a local bookmark row.
- Use `x` in the bookmarks view to delete a local bookmark row.
- Use `bf` in the bookmarks view to forget a tracked local bookmark or a single exact remote-only
  bookmark row when metadata proves that target safely.
- Use `bt` and `bu` in the bookmarks view to track or untrack the exact selected remote bookmark, or
  a local bookmark row with exactly one eligible remote peer.
- The bookmark list is useful for inspection and for selecting exact bookmark targets.
- Remote and tracking actions stay guided and exact-targeted rather than becoming a broad remote
  dashboard.

## Resolve

- Press `R` or run `jk resolve` to open the resolve screen.
- The screen is currently read-only.
- It lists conflicted paths with their file type and side count.
- Use `Enter`, `l`, or `Right` when the row has an exact path and you want to inspect the file.
- Search, copy, refresh, back, and help behave like the other selectable list screens.
- Clean repositories open as `0 conflicts` instead of a failure state.

## Help

- Press `?` on any of these screens to see the current bindings.
