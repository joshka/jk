# Workflow integration foundation

Historical record of the workflow foundation assembled on 2026-09-20, before the broader
`integration-all` pass. It combined rebase, whole-change squash, all-path restore, and workspace
lifecycle actions on the action-menu baseline while preserving the original task changes.

The [workspace coherence record](workspace-coherence.md) tracks the later integration and validation;
[command coverage](command-coverage.md) describes the implemented scope under review. The source
revisions, test counts, and landing order below refer to this earlier foundation.

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

## Foundation interaction contracts

- `R` opens rebase selection. One source is resolved to a full commit ID; multiple marks are rejected.
- Rebase defaults to revision-only onto. Broader branch, descendant, and insertion modes are explicit.
- `a s` uses ordered source marks and a distinct cursor destination for whole-change squash.
- `a r` previews all-path copying from the selected exact commit into the working copy.
- `W`, then `a`, offers workspace add, rename, forget metadata, and update-stale.
- Each new mutation requires a separate confirmation; cancel leaves the graph unchanged.
- Commands run through the recorded runner, capture resulting operation IDs, and refresh on success.
- Failure stays visible without refreshing away useful context. This stage offered no immutable
  override; the later [Run Options](run-options.md) flow adds a command-scoped override.
- `C` opens Command History; `o` there follows a successful command's resulting operation.
- Existing immediate New/Edit/Undo/Redo and empty-abandon behavior is unchanged.

The original dialogs used slate backgrounds and colored regions without borders. The later
integration replaces their separate palettes with shared styles under the [TUI design
contract](tui-design.md). Long exact-command previews retain visible confirmation controls while
scrolling, and repeated Enter events cannot submit a preview opened by the same held key.

## Foundation visual evidence

Run `just betamax-workflows` from the repository root. It builds the local binary and records
`tapes/workflows-integration.tape` against unique disposable fixtures, not the development graph.
The tape writes its GIF and PNG checkpoints under `target/dogfood-artifacts/betamax/` and checks
the rebase graph before and after confirmation. Generated media is intentionally not tracked.

The combined tape covers cancellation, successful mutations, history, operation links, and a real
workspace-name collision. Fixture paths remain visible in exact commands so the preview matches argv.
The foundation run passed on 2026-09-20 and produced `workflows-integration.gif` plus PNG checkpoints.
Rebase cancellation/success and forgotten-directory retention also have shell assertions; other
cancellation paths have visual checkpoints and unit coverage. The successful workspace-add history
record opens its captured operation; metadata-only commands can fall back to the operation log.

## Recorded validation and landing order

The foundation was assembled in `/Users/joshka/local/jk/work/workflows-integration`.
Its three changes were ordered above the action-menu baseline as follows:

1. `vkvnvpvl`: whole-change squash, with ordered roles and an exact-command preview.
1. `mzrsuzxm`: restore and the unified borderless, scrollable confirmation renderer.
1. `szmsyuyq`: rebase, workspace lifecycle, confirmation safety, and the combined demo.

The lower layers independently passed their workspace tests: 504 for squash and 516 for restore.
The foundation tip passed 543 tests and `just release-check`, covering formatting, compilation,
Clippy, unused dependencies, Rustdoc, packaging, install smoke, and Markdown lint. Focused tests cover
ambiguous/divergent marks, immutable rejection, spawn/nonzero failures, cancellation, refresh,
operation linkage, narrow layouts, and repeated-key/unrendered confirmation protection.

The proposed landing order preserved these layers without inserting the original task changes
under unrelated workspaces. The final layer reconciled rebase and workspace lifecycle together
because they share input dispatch and confirmation safety. Publication and review of the later
combined change are tracked in the workspace coherence record.

## Remaining scope

Rebase searches the currently visible log, not an arbitrary repository-wide revset. Multi-parent
destinations and ghost previews remain deferred. Squash and restore do not select files or hunks.
Concurrent external jj operations can still stale a preview or workspace; jj diagnostics and
operation history remain the recovery boundary. Website/media publication is a separate handoff.
At the foundation review, the companion website still listed rebase, squash, and restore as planned.
Website copy and release media must follow the released feature set; the foundation changed neither
the website nor the separate screenshot repository.
