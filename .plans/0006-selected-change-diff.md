# Selected-Change Diff

## Why

After the log refresh loop, the next useful product slice is diff inspection for the selected
change. It should reuse the same mental model: open the current target, refresh manually, preserve
context, and return to the log.

## Scope

Build only enough diff view to support:

1. Open a diff for the selected log change.
1. Render the `jj`-equivalent diff output without inventing a different presentation.
1. Refresh manually with one key.
1. Return to the log without losing selection.
1. Collapse and expand file sections first; hunk-level collapse can follow if file collapse is
   solid.

Avoid mutation actions, staging-like concepts, or a broad command launcher in this slice.

## Design Questions

- Should the diff view shell out first, or can it share a structured `jj` integration boundary with
  the log view?
- What is the minimum structure needed for file/hunk collapse without parsing more diff text than
  necessary?
- Does refresh preserve collapsed sections by file path, hunk identity, or visible order?
- Which vimish movement commands must be shared with the log view?

## Done When

- The selected log row can open a diff view and return.
- Manual refresh works in the diff view.
- Collapsed file sections survive refresh when the file still exists.
- Tests cover target selection, refresh, return navigation, and collapse state.
