# Routine jj command coverage

Status: integration review on 2026-09-20. The unfinished source workspaces have been combined in
`integration-all`; this is the implemented scope under review, not a released-version claim.

The [product plan](product-plan.md) remains the direction. The CLI surface addendum records jj 0.42;
installed `jj help` now also exposes `converge`, `run`, and remote tag tracking. Recheck installed
help when adding command forms. The screen atlas is a design study, not implemented coverage.

## Implemented and remaining workflows

| Family                 | Current native scope                                                    | Remaining scope                                                  |
| ---------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------- |
| Log, show, status      | Focused jj-rendered views; log templates and navigation                 | Shared display options beyond log/diff                           |
| Diff                   | Revision and from/to forms; formats, file/hunk movement, search         | Fileset operands and richer comparisons                          |
| Evolog, interdiff      | Selected-change evolog                                                  | Native interdiff; typed commands remain available                |
| New, edit, describe    | Menu actions; ordered new parents; multiline inline description         | Editor handoff and advanced new options                          |
| Commit                 | Typed noninteractive commands                                           | Native working-copy commit workflow; commit has no revision flag |
| Rebase                 | Explicit source/destination roles, visible-log search, exact preview    | Multiple destinations and graph preview                          |
| Squash, restore        | Whole-change squash; selected-commit all-path restore into working copy | File and hunk selection                                          |
| Split, absorb, resolve | Typed noninteractive forms only                                         | Native actions; foreground diff/merge tool handoff               |
| Abandon                | Empty-check fast path; destructive confirmation otherwise               | Broader multi-revision forms                                     |
| Duplicate, revert      | Typed commands                                                          | Native P2 actions                                                |
| Bookmarks              | List, create, move, delete                                              | Configured list templates; advanced bookmark actions             |
| Tags                   | Typed commands                                                          | Native list and scoped actions                                   |
| Git fetch/push         | Explicit remote fetch; one-bookmark push dry-run                        | Real push is disabled; advanced remote forms                     |
| Undo/redo, operations  | Undo/redo; operation log/show/diff; history links                       | Native operation restore/integrate and display options           |
| Workspaces             | List, scoped inspection, add/rename/forget/update-stale                 | Creation parents/message/sparse options                          |

Command mode captures output and keeps failures visible. It is not a foreground terminal handoff.
The refs flow restricts bookmark mutation, fetch, and push command-mode entry to its dedicated
flows. In particular, a successful push dry-run does not mean publication is implemented.

## Integrated foundations

- The [safe workflow integration](workflow-integration.md) retains explicit operand roles, exact
  previews, recorded results, operation links, and borderless confirmation surfaces.
- Shared selectors provide a small typed handoff from view selections to command roles. Preserve
  existing navigation and rendering rather than replacing them with a generic selector framework.
- The integration provides captured shell-free external commands and cancellable log refresh.
  Its null-stdin external runner does not support interactive editors or merge tools.
- Refs work provides real bookmark and remote workflows through dry-run, with exact patterns and
  explicit destinations. Preserve its distinction between deletion, forgetting, and abandonment.

## Next coherent slices

1. Add foreground jj tool handoff, beginning with editor describe. Restore the terminal, record
   success/failure, and refresh when returning. Reuse it for split, diffedit, and resolve.
1. Add shared fileset selection for commit/split, then extend squash/restore. Show selected and
   remaining content, keep commit scoped to the working copy, and preserve explicit command roles.
1. Complete publication from the existing push dry-run flow, retaining the chosen bookmark and
   remote through final confirmation. Native interdiff is a smaller independent inspection slice.

Use fresh disposable repositories for execution tests and Betamax scenarios. Review terminal text
and PNG checkpoints together. Build from source workspaces; do not exercise mutation workflows on
the development repository.
