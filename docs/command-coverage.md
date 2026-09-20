# Routine jj command coverage

Source coverage reviewed on 2026-09-20. These workflows describe this checkout; installed releases
may have fewer features.

The [CLI surface addendum](plans/cli-surface-addendum.md) records jj 0.42. The installed `jj help`
used for this review also exposes `converge`, `run`, and remote tag tracking. Check the installed
version's help when adding command forms. The [screen atlas](design/atlas.html) is a design study.

## Implemented and remaining workflows

| Family                 | Native workflow                                                     | Gaps and command-mode alternatives                                           |
| ---------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Log, show, status      | jj-rendered views; log templates and navigation                     | Shared display options beyond log/diff                                       |
| Diff                   | Revision and from/to forms; formats, file/hunk movement, search     | Fileset operands and richer comparisons                                      |
| Evolog, interdiff      | Selected-change evolog                                              | Interdiff is available through command mode                                  |
| New, edit, describe    | Menu actions; ordered new parents; multiline inline description     | Editor handoff and advanced new options                                      |
| Commit                 | None                                                                | Noninteractive command mode; commit operates on the working copy             |
| Rebase                 | Source/destination roles, visible-log search, exact command preview | Multiple destinations and graph preview                                      |
| Squash, restore        | Whole-change squash; all-path restore into the working copy         | File and hunk selection                                                      |
| Split, absorb, resolve | None                                                                | Noninteractive command mode; foreground diff/merge tools unavailable         |
| Abandon                | Empty revisions run immediately; other cases open confirmation      | Multi-revision forms use command mode                                        |
| Duplicate, revert      | None                                                                | Available through command mode                                               |
| Bookmarks              | List, create, move, delete                                          | Configured list templates; other actions use command mode                    |
| Tags                   | None                                                                | Available through command mode                                               |
| Git fetch/push         | Explicit remote fetch; one-bookmark push dry-run                    | Real push is unavailable; other remote workflows remain planned              |
| Undo/redo, operations  | Undo/redo; operation log/show/diff; history links                   | Operation restore/integrate use command mode; display options remain planned |
| Workspaces             | List, scoped inspection, add/rename/forget/update-stale             | Creation parents/message/sparse options                                      |

Command mode (`:`) captures output with stdin closed. Use noninteractive forms, such as supplying a
message to commands that would otherwise open an editor. Interactive editors, diff tools, and merge
tools require leaving jk.

Command mode rejects bookmark create/move/delete and Git fetch/push, including push dry-run, with
instructions to use the bookmark screen. Other bookmark commands remain available. The bookmark
screen's push dry-run may contact the remote but does not publish changes.

## Shared command behavior

- Rebase, squash, and restore use explicit source/destination roles and exact command previews.
  [Run options](run-options.md) changes execution settings for that pending command only.
- Command History retains executed commands, diagnostic output, and resulting operation links for
  the session. See [workflow integration](workflow-integration.md) for operand and refresh behavior.
- External command mode (`!`) runs an executable without adding a shell. Like jj command mode, it
  captures output and closes stdin. Explicit log refresh runs in the background and can be cancelled.
- Bookmark and remote workflows use exact name patterns and an explicit remote. Deleting a bookmark,
  forgetting workspace metadata, and abandoning a revision remain separate actions.

## Next workflows

1. Add foreground jj tool handoff, beginning with editor describe. Restore the terminal, record
   success/failure, and refresh when returning. Reuse it for split, diffedit, and resolve.
1. Add shared fileset selection for commit/split, then extend squash/restore. Show selected and
   remaining content, keep commit scoped to the working copy, and preserve explicit command roles.
1. Complete publication from the existing push dry-run flow, retaining the chosen bookmark and
   remote through final confirmation. Native interdiff is a smaller independent inspection task.

Use fresh disposable repositories for execution tests and Betamax scenarios. Review terminal text
and PNG checkpoints together. Build from source workspaces; do not exercise mutation workflows on
the development repository.
