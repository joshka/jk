# jk-cli

`jj` process integration for `jk`.

Log inspection combines configured `jj` rendering with JSON records for navigation and selection.
Diff and other command modules provide the output and metadata needed by their views. Bookmark lists
and abandon's embedded Git-format patch use fixed formats.

Queries build command specifications for inspection, mutations, workspaces, bookmarks, and remotes.
Runners execute those specifications, with optional cancellation and history recording. External
programs use a separate runner interface and receive arguments without implicit shell interpretation.

Direct `jj-cli` / `jj-lib` integration remains a longer-term option if it can preserve jj's configured
rendering and command semantics without duplicating its display logic.
