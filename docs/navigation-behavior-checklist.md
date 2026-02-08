# Navigation Behavior Checklist

Use this checklist when reviewing navigation and interaction UX in `jk`.
It targets expectations from people who use readline, vim, and terminal pagers daily.

## Global Session Behavior

- [x] `q` quits immediately from normal mode.
- [x] `Ctrl+c` exits immediately from any mode.
- [x] Header always shows current mode and active command context.
- [x] Footer always shows actionable mode-specific hints.

## Normal-Mode Movement (Vim + Arrow Parity)

- [x] `j` and `Down` move to the next selectable item.
- [x] `k` and `Up` move to the previous selectable item.
- [x] `g` and `Home` jump to the top.
- [x] `G` and `End` jump to the bottom.
- [x] `PageDown` pages forward by viewport.
- [x] `PageUp` pages backward by viewport.
- [x] `Ctrl+d` and `Ctrl+f` page forward.
- [x] `Ctrl+u` and `Ctrl+b` page backward.
- [x] Movement keeps cursor visible without manual scrolling.

## Log Selection Semantics

- [x] In log-like views, movement is item-based (revision to revision), not raw line-based.
- [x] Multi-line log entries stay grouped under one selectable revision.
- [x] `Enter` and `d` actions resolve the currently selected revision reliably.

## Screen Switching and Discovery

- [x] Common screens are one keypress away in normal mode (`l`, `s`, `o`, `L`, `v`, `f`, `t`, `w`).
- [x] `:` opens command mode for exact command-driven screen changes.
- [x] `?` opens command help.
- [x] `Left`/`Right` and `Ctrl+o`/`Ctrl+i` navigate back/forward screen history.
- [x] Help lists common screens before full command coverage.
- [x] Help includes explicit navigation keys and paging keys.

## Readline-Like Command Entry

- [x] `Esc` cancels command mode.
- [x] `Enter` submits command mode input.
- [x] `Up` and `Down` navigate command history.
- [x] Command history restores draft input when returning from history traversal.

## Prompt and Confirmation Modes

- [x] `Esc` cancels prompts.
- [x] `Esc` or `n` rejects confirmations.
- [x] Confirmation state clearly indicates the command being approved.
- [x] Prompt state clearly indicates label and current input.

## Visual Contract

- [x] ANSI colors from `jj` output remain visible in the content body.
- [x] Selected row has a clear marker and highlight.
- [x] Header/footer palette is stable and mode-aware.
