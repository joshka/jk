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

## First Slice

Use the existing shell-out boundary for this slice. `JjDiff` runs `jj diff -r <change>` with color
preserved, and the TUI treats the body as opaque rendered `jj` output except for the narrow file
headers needed by collapse state.

The log keeps its existing inline expansion on enter/right. The selected-change diff opens from the
log with `d`, or directly with `jk diff [REVISION]`. In the diff view, `r` refreshes, `j/k` scrolls
line by line, space/PageUp/PageDown keep less-style page movement, and `[`/`]` jump between file
sections. File folding uses `h`/left and unfolding uses `l`/right; Ctrl-left folds all files and
Ctrl-right unfolds all files. `H`/`L` return to the log when the diff was opened from a log view,
preserving the same `LogView` instance so selection and scroll are not reloaded. `d` does not close
the diff view.

File-section collapse is intentionally path-based and limited to recognizable `jj diff` file
headers such as `Modified regular file path:`. Refresh keeps collapsed paths that still appear and
drops paths that disappear. Folded file rows append the colored per-file `jj diff --stat` suffix on
the header line, with the stat pipe aligned after the full diff header text rather than only the
path. When a file header scrolls above the viewport, the diff view pins that current file header at
the top of the content area so the active file stays visible while reading large diffs. The diff
view uses a flat, subtle selected-file highlight instead of the log view's graph-row gradient.
Hunk-level collapse remains out of scope.

## Done When

- The selected log row can open a diff view and return.
- Manual refresh works in the diff view.
- Collapsed file sections survive refresh when the file still exists.
- Tests cover target selection, refresh, return navigation, and collapse state.

## Follow-Up Slices

1. Diff search: `/` opens a search prompt, Enter jumps to the first visible line match, and `n`/`N`
   repeat the last search forward or backward with wrapping.
1. Current-file context: when the real file header scrolls offscreen, the pinned header includes
   the same stat suffix and a compact file index such as `[file 1/3]`.
1. Horizontal overflow: `<` and `>` scroll wide diff content horizontally, with the current column
   shown in the status line when shifted.

## Validation

- `cargo test -p jk-cli -p jk-tui -p jk --lib --bins`
- `just fmt-check`
- `just check`
- `just test`
- `just clippy`
