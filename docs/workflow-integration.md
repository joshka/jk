# Safe workflow integration

This integration brings rebase, whole-change squash, all-path restore, and workspace lifecycle
actions onto the action-menu baseline. It does not publish or rewrite the original task changes.

## Recovered inputs

| Workflow            | Original commit        | Integration treatment                                               |
| ------------------- | ---------------------- | ------------------------------------------------------------------- |
| Baseline            | `16e39f01`             | Local `main`, including the contextual action menu                  |
| Squash              | `01977d5e`             | Duplicated; exact commit roles and destination reselection retained |
| Restore             | `3dd4d3b7`             | Duplicated; shared confirmation reconciled with squash              |
| Workspace lifecycle | `e8274097`             | Duplicated; confirmation and history coverage strengthened          |
| Rebase              | `8ce008ba`, `03b8811e` | Recovered CLI model; incomplete app wiring rebuilt                  |

The earlier divergence came from insertion rebases onto `main`. `jj rebase -A main` also rewrites
existing descendants of `main`, including other tasks' working-copy changes. Separate filesystem
workspaces do not isolate that shared graph. The originals and `joshka/rebase-recovery-*` bookmarks
remain available; integration uses new change IDs and does not insert before unrelated descendants.

## Interaction contracts

- `R` opens rebase selection. One source is resolved to a full commit ID; multiple marks are rejected.
- Rebase defaults to revision-only onto. Broader branch, descendant, and insertion modes are explicit.
- `a s` uses ordered source marks and a distinct cursor destination for whole-change squash.
- `a r` previews all-path copying from the selected exact commit into the working copy.
- `W`, then `a`, offers workspace add, rename, forget metadata, and update-stale.
- Each new mutation requires a separate confirmation; cancel leaves the graph unchanged.
- Commands run through the recorded runner, capture resulting operation IDs, and refresh on success.
- Failure stays visible without refreshing away useful context. No immutable override is offered.
- `C` opens Command History; `o` there follows a successful command's resulting operation.
- Existing immediate New/Edit/Undo/Redo and empty-abandon behavior is unchanged.

The dialogs use slate backgrounds and colored regions rather than borders. Long exact-command
previews scroll while keeping confirmation controls visible. Repeated Enter events cannot submit
the preview that a held key just opened.

## Reproduce visual evidence

Run `just betamax-workflows` from the repository root. It builds the local binary and records
`tapes/workflows-integration.tape` against unique disposable fixtures, not the development graph.
The tape writes its GIF and PNG checkpoints under `target/dogfood-artifacts/betamax/` and checks
the rebase graph before and after confirmation. Generated media is intentionally not tracked.

The combined tape covers cancellation, successful mutations, history, operation links, and a real
workspace-name collision. Fixture paths remain visible in exact commands so the preview matches argv.
The complete run passed on 2026-09-20 and produced `workflows-integration.gif` plus PNG checkpoints.
Rebase cancellation/success and forgotten-directory retention also have shell assertions; other
cancellation paths have visual checkpoints and unit coverage. The successful workspace-add history
record opens its captured operation; metadata-only commands can fall back to the operation log.

## Validation and landing order

The dedicated integration workspace is `/Users/joshka/local/jk/work/workflows-integration`.
Its three changes sit directly above the action-menu baseline, in this order:

1. `vkvnvpvl`: whole-change squash, with ordered roles and an exact-command preview.
1. `mzrsuzxm`: restore and the unified borderless, scrollable confirmation renderer.
1. `szmsyuyq`: rebase, workspace lifecycle, confirmation safety, and the combined demo.

The lower layers independently pass their workspace tests: 504 for squash and 516 for restore.
The integrated tip passes 543 tests and `just release-check`, covering formatting, compilation,
Clippy, unused dependencies, Rustdoc, packaging, install smoke, and Markdown lint. Focused tests cover
ambiguous/divergent marks, immutable rejection, spawn/nonzero failures, cancellation, refresh,
operation linkage, narrow layouts, and repeated-key/unrendered confirmation protection.

Land in the listed order, not by inserting the original task changes under existing workspaces.
The final layer intentionally reconciles rebase and workspace lifecycle together because they share
input dispatch and confirmation safety. No remote publication is performed by this integration.

## Remaining scope

Rebase searches the currently visible log, not an arbitrary repository-wide revset. Multi-parent
destinations and ghost previews remain deferred. Squash and restore do not select files or hunks.
Concurrent external jj operations can still stale a preview or workspace; jj diagnostics and
operation history remain the recovery boundary. Website/media publication is a separate handoff.
The companion website was checked: its homepage still lists rebase, squash, and restore as planned.
Update that copy and regenerate release media after this stack lands; this integration does not edit
or publish the separate website or screenshot repositories.
