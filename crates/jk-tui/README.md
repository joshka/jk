# jk-tui

Ratatui views and interaction state for `jk`.

Views cover logs, diffs, workspaces, bookmarks, operations, command history, and command previews.
They consume caller-provided snapshots and manage navigation, selection, and rendering. The calling
application executes commands and supplies refreshed data.

Log and diff views preserve jj-rendered content inside borderless title and status chrome. Cursor
and mark indicators occupy a separate gutter. Shared styles define light and dark dialog surfaces,
keyboard accents, supporting text, and ordinary or destructive action colors.
